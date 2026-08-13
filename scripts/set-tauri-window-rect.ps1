param(
    [Parameter(Mandatory = $true)][int]$ProcessId,
    [Parameter(Mandatory = $true)][ValidateSet('main', 'scoreboard')][string]$WindowKind,
    [Parameter(Mandatory = $true)][int]$Width,
    [Parameter(Mandatory = $true)][int]$Height,
    [Parameter(Mandatory = $true)][string]$ExpectedExePath,
    [Parameter(Mandatory = $true)][string]$OutputJson,
    [int]$X = 40,
    [int]$Y = 40
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class PreflightWindows {
  public delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr hwnd, StringBuilder text, int maxCount);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool GetWindowRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool GetClientRect(IntPtr hwnd, out RECT rect);
  [DllImport("user32.dll", SetLastError=true)] public static extern bool ClientToScreen(IntPtr hwnd, ref System.Drawing.Point point);
  [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr hwnd);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr hwnd, int attribute, out RECT value, int size);
  public static List<IntPtr> VisibleForProcess(uint pid) {
    var result = new List<IntPtr>();
    EnumWindows((hwnd, state) => { uint candidate; GetWindowThreadProcessId(hwnd, out candidate); if (candidate == pid && IsWindowVisible(hwnd)) result.Add(hwnd); return true; }, IntPtr.Zero);
    return result;
  }
  public static string Title(IntPtr hwnd) { var text = new StringBuilder(512); GetWindowTextW(hwnd, text, text.Capacity); return text.ToString(); }
}
'@ -ReferencedAssemblies System.Drawing

function Convert-Rect($Rect) {
    [ordered]@{ x = $Rect.Left; y = $Rect.Top; width = $Rect.Right - $Rect.Left; height = $Rect.Bottom - $Rect.Top; left = $Rect.Left; top = $Rect.Top; right = $Rect.Right; bottom = $Rect.Bottom }
}

$process = Get-Process -Id $ProcessId -ErrorAction Stop
$actualExe = [System.IO.Path]::GetFullPath($process.Path)
$expectedExe = [System.IO.Path]::GetFullPath($ExpectedExePath)
if (-not [string]::Equals($actualExe, $expectedExe, [StringComparison]::OrdinalIgnoreCase)) { throw "PID $ProcessId executable mismatch: $actualExe" }

$expectedTitle = if ($WindowKind -eq 'main') {
    -join ([char[]]@(0x43, 0x53, 0x32, 0x4EBA, 0x673A, 0x589E, 0x5F3A, 0x52A9, 0x624B))
} else {
    -join ([char[]]@(0x672C, 0x5C40, 0x6218, 0x62A5))
}
$windows = @([PreflightWindows]::VisibleForProcess([uint32]$ProcessId) | Where-Object { [PreflightWindows]::Title($_) -eq $expectedTitle })
if ($windows.Count -ne 1) {
    $found = @([PreflightWindows]::VisibleForProcess([uint32]$ProcessId) | ForEach-Object { "0x$($_.ToInt64().ToString('X')):$([PreflightWindows]::Title($_))" }) -join ', '
    throw "Expected one visible $WindowKind window for PID $ProcessId, found $($windows.Count). Visible: $found"
}
$hwnd = $windows[0]
$SWP_NOZORDER = 0x0004
$SWP_SHOWWINDOW = 0x0040
if (-not [PreflightWindows]::SetWindowPos($hwnd, [IntPtr]::Zero, $X, $Y, $Width, $Height, $SWP_NOZORDER -bor $SWP_SHOWWINDOW)) {
    throw "SetWindowPos failed: $([Runtime.InteropServices.Marshal]::GetLastWin32Error())"
}
[void][PreflightWindows]::SetForegroundWindow($hwnd)
Start-Sleep -Milliseconds 700

$windowRect = New-Object PreflightWindows+RECT
$clientRect = New-Object PreflightWindows+RECT
$dwmRect = New-Object PreflightWindows+RECT
if (-not [PreflightWindows]::GetWindowRect($hwnd, [ref]$windowRect)) { throw 'GetWindowRect failed' }
if (-not [PreflightWindows]::GetClientRect($hwnd, [ref]$clientRect)) { throw 'GetClientRect failed' }
$clientOrigin = [Drawing.Point]::new(0, 0)
if (-not [PreflightWindows]::ClientToScreen($hwnd, [ref]$clientOrigin)) { throw 'ClientToScreen failed' }
$dwmResult = [PreflightWindows]::DwmGetWindowAttribute($hwnd, 9, [ref]$dwmRect, [Runtime.InteropServices.Marshal]::SizeOf([type]'PreflightWindows+RECT'))
$dpi = [PreflightWindows]::GetDpiForWindow($hwnd)
$actualWidth = $windowRect.Right - $windowRect.Left
$actualHeight = $windowRect.Bottom - $windowRect.Top
$result = [ordered]@{
    timestamp = (Get-Date).ToString('o')
    pid = $ProcessId
    processStart = $process.StartTime.ToString('o')
    executablePath = $actualExe
    hwnd = "0x$($hwnd.ToInt64().ToString('X'))"
    title = [PreflightWindows]::Title($hwnd)
    windowKind = $WindowKind
    requested = [ordered]@{ x = $X; y = $Y; width = $Width; height = $Height }
    windowRect = Convert-Rect $windowRect
    clientRect = [ordered]@{ x = $clientOrigin.X; y = $clientOrigin.Y; width = $clientRect.Right; height = $clientRect.Bottom }
    extendedFrameBounds = if ($dwmResult -eq 0) { Convert-Rect $dwmRect } else { $null }
    dpi = $dpi
    scale = [Math]::Round($dpi / 96.0, 3)
    withinTolerance = ([Math]::Abs($actualWidth - $Width) -le 2 -and [Math]::Abs($actualHeight - $Height) -le 2)
}
if (-not $result.withinTolerance) { throw "Window rect is $actualWidth x $actualHeight, expected $Width x $Height" }
$parent = Split-Path -Parent $OutputJson
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $OutputJson -Encoding utf8
$result | ConvertTo-Json -Depth 6
