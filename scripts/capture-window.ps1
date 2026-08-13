param(
    [Parameter(Mandatory = $true)][string]$WindowEvidenceJson,
    [Parameter(Mandatory = $true)][string]$OutputPng,
    [Parameter(Mandatory = $true)][string]$ExeSha256,
    [string]$Page = '',
    [int]$DemoId = 0,
    [int]$Round = 0,
    [int]$Tick = 0,
    [int]$Points = 0,
    [string]$CanvasRectJson = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
Add-Type -AssemblyName System.Drawing
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class PreflightCaptureWindow {
  [DllImport("user32.dll", SetLastError=true)] public static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int cx, int cy, uint flags);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hwnd);
}
'@
function Get-Sha256([string]$Path) {
    $stream = [IO.File]::OpenRead($Path)
    try {
        $hash = [Security.Cryptography.SHA256]::Create()
        try { return -join ($hash.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') }) }
        finally { $hash.Dispose() }
    } finally { $stream.Dispose() }
}
$window = Get-Content -LiteralPath $WindowEvidenceJson -Raw | ConvertFrom-Json
$rect = $window.windowRect
$captureRect = if ($window.extendedFrameBounds) { $window.extendedFrameBounds } else { $rect }
$parent = Split-Path -Parent $OutputPng
if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$bitmap = [Drawing.Bitmap]::new([int]$captureRect.width, [int]$captureRect.height, [Drawing.Imaging.PixelFormat]::Format32bppArgb)
$graphics = [Drawing.Graphics]::FromImage($bitmap)
$hwndValue = [IntPtr]([Convert]::ToInt64($window.hwnd.Substring(2), 16))
$SWP_NOSIZE = 0x0001; $SWP_NOMOVE = 0x0002; $SWP_SHOWWINDOW = 0x0040
$HWND_TOPMOST = [IntPtr](-1); $HWND_NOTOPMOST = [IntPtr](-2)
try {
    if (-not [PreflightCaptureWindow]::SetWindowPos($hwndValue, $HWND_TOPMOST, 0, 0, 0, 0, $SWP_NOSIZE -bor $SWP_NOMOVE -bor $SWP_SHOWWINDOW)) { throw 'Unable to bring evidence HWND to the top' }
    [void][PreflightCaptureWindow]::SetForegroundWindow($hwndValue)
    Start-Sleep -Milliseconds 350
    $graphics.CopyFromScreen([int]$captureRect.x, [int]$captureRect.y, 0, 0, $bitmap.Size, [Drawing.CopyPixelOperation]::SourceCopy)
    $bitmap.Save($OutputPng, [Drawing.Imaging.ImageFormat]::Png)
} finally {
    [void][PreflightCaptureWindow]::SetWindowPos($hwndValue, $HWND_NOTOPMOST, 0, 0, 0, 0, $SWP_NOSIZE -bor $SWP_NOMOVE -bor $SWP_SHOWWINDOW)
    $graphics.Dispose()
    $bitmap.Dispose()
}

$image = [Drawing.Bitmap]::FromFile($OutputPng)
try {
    $colors = [Collections.Generic.HashSet[int]]::new()
    $sampleCount = 0; $black = 0; $transparent = 0; $nonUniform = 0
    $first = $image.GetPixel(0, 0).ToArgb()
    for ($y = 0; $y -lt $image.Height; $y += 4) {
        for ($x = 0; $x -lt $image.Width; $x += 4) {
            $color = $image.GetPixel($x, $y)
            [void]$colors.Add($color.ToArgb()); $sampleCount++
            if ($color.A -eq 0) { $transparent++ }
            if ($color.R -lt 8 -and $color.G -lt 8 -and $color.B -lt 8) { $black++ }
            if ($color.ToArgb() -ne $first) { $nonUniform++ }
        }
    }
    $canvasMetrics = $null
    if ($CanvasRectJson) {
        $canvas = $CanvasRectJson | ConvertFrom-Json
        $scale = [double]$window.scale
        $cropX = [Math]::Max(0, [int]([int]$window.clientRect.x * $scale + [int]$canvas.x * $scale - [int]$captureRect.x))
        $cropY = [Math]::Max(0, [int]([int]$window.clientRect.y * $scale + [int]$canvas.y * $scale - [int]$captureRect.y))
        $cropWidth = [Math]::Min([int]([int]$canvas.width * $scale), $image.Width - $cropX)
        $cropHeight = [Math]::Min([int]([int]$canvas.height * $scale), $image.Height - $cropY)
        $canvasColors = [Collections.Generic.HashSet[int]]::new(); $canvasSamples = 0; $canvasBlack = 0
        for ($cy = $cropY; $cy -lt $cropY + $cropHeight; $cy += 3) {
            for ($cx = $cropX; $cx -lt $cropX + $cropWidth; $cx += 3) {
                $pixel = $image.GetPixel($cx, $cy); [void]$canvasColors.Add($pixel.ToArgb()); $canvasSamples++
                if ($pixel.R -lt 8 -and $pixel.G -lt 8 -and $pixel.B -lt 8) { $canvasBlack++ }
            }
        }
        $canvasMetrics = [ordered]@{ x = $cropX; y = $cropY; width = $cropWidth; height = $cropHeight; sampledPixels = $canvasSamples; uniqueSampledColors = $canvasColors.Count; blackRatio = if ($canvasSamples) { [Math]::Round($canvasBlack / [double]$canvasSamples, 6) } else { 1 }; textured = ($canvasSamples -gt 100 -and $canvasColors.Count -gt 64 -and $canvasBlack -lt $canvasSamples * 0.95) }
    }
    $metrics = [ordered]@{
        timestamp = (Get-Date).ToString('o')
        path = [IO.Path]::GetFullPath($OutputPng)
        sha256 = Get-Sha256 $OutputPng
        bytes = (Get-Item -LiteralPath $OutputPng).Length
        width = $image.Width
        height = $image.Height
        pid = $window.pid
        hwnd = $window.hwnd
        exeSha256 = $ExeSha256
        windowRect = $window.windowRect
        clientRect = $window.clientRect
        captureRect = $captureRect
        dpi = $window.dpi
        scale = $window.scale
        page = $Page
        demoId = if ($DemoId) { $DemoId } else { $null }
        round = if ($Round) { $Round } else { $null }
        tick = if ($Tick) { $Tick } else { $null }
        points = if ($Points) { $Points } else { $null }
        sampledPixels = $sampleCount
        uniqueSampledColors = $colors.Count
        blackRatio = [Math]::Round($black / [double]$sampleCount, 6)
        transparentRatio = [Math]::Round($transparent / [double]$sampleCount, 6)
        nonUniformRatio = [Math]::Round($nonUniform / [double]$sampleCount, 6)
        nonBlank = ($colors.Count -gt 32 -and $black -lt $sampleCount * 0.95)
        canvas = $canvasMetrics
    }
} finally { $image.Dispose() }
$metricsPath = [IO.Path]::ChangeExtension($OutputPng, '.json')
$metrics | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $metricsPath -Encoding utf8
$metrics | ConvertTo-Json -Depth 8
