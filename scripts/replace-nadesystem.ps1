param(
  [string]$ZipPath = 'src-tauri/resources/CS2BotImprover.zip',
  [string]$DllPath = 'third_party/CS2-Bot-Improver-v1.4.4/addons/counterstrikesharp/plugins/NadeSystem/bin/Release/net10.0/NadeSystem.dll',
  [string]$ReportDirectory = 'workspace/runtime/nadesystem-replacement'
)

$ErrorActionPreference = 'Stop'
$entryName = 'addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll'
$panelName = 'Panel v1.4.4.exe'
$panelSha256 = '2797A3FE85E65959CAE9501525B67B3876CEF65152E88DC716F64D5485AC2182'
$panelSize = 5890560

$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$zip = (Resolve-Path (Join-Path $workspace $ZipPath)).Path
$dll = (Resolve-Path (Join-Path $workspace $DllPath)).Path
$reportRoot = Join-Path $workspace $ReportDirectory
$temporary = "$zip.tmp-$([guid]::NewGuid().ToString('N'))"
$backup = "$zip.before-nades-pacing-1.4.4-$((Get-Date).ToString('yyyyMMdd-HHmmss')).bak"

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Get-StreamSha256([System.IO.Stream]$Stream) {
  $hash = [System.Security.Cryptography.SHA256]::Create()
  try {
    return [Convert]::ToHexString($hash.ComputeHash($Stream))
  }
  finally {
    $hash.Dispose()
  }
}

function Get-ZipManifest([string]$Path) {
  $archive = [System.IO.Compression.ZipFile]::OpenRead($Path)
  try {
    return @($archive.Entries | ForEach-Object {
      $stream = $_.Open()
      try {
        [pscustomobject]@{
          name = $_.FullName
          size = $_.Length
          sha256 = Get-StreamSha256 $stream
        }
      }
      finally {
        $stream.Dispose()
      }
    })
  }
  finally {
    $archive.Dispose()
  }
}

New-Item -ItemType Directory -Path $reportRoot -Force | Out-Null
$before = Get-ZipManifest $zip
Copy-Item -LiteralPath $zip -Destination $temporary -Force
(Get-Item -LiteralPath $zip).IsReadOnly = $false
(Get-Item -LiteralPath $temporary).IsReadOnly = $false

try {
  $archive = [System.IO.Compression.ZipFile]::Open($temporary, [System.IO.Compression.ZipArchiveMode]::Update)
  try {
    $matches = @($archive.Entries | Where-Object FullName -eq $entryName)
    if ($matches.Count -ne 1) {
      throw "Expected exactly one $entryName entry, found $($matches.Count)."
    }
    $matches[0].Delete()
    $replacement = $archive.CreateEntry($entryName, [System.IO.Compression.CompressionLevel]::Optimal)
    $replacement.LastWriteTime = [DateTimeOffset]::new(2026, 7, 26, 0, 0, 0, [TimeSpan]::Zero)
    $input = [System.IO.File]::OpenRead($dll)
    $output = $replacement.Open()
    try {
      $input.CopyTo($output)
    }
    finally {
      $output.Dispose()
      $input.Dispose()
    }
  }
  finally {
    $archive.Dispose()
  }

  $after = Get-ZipManifest $temporary
  if ($before.Count -ne $after.Count) {
    throw "ZIP entry count changed from $($before.Count) to $($after.Count)."
  }

  $beforeByName = @{}
  $afterByName = @{}
  $before | ForEach-Object { $beforeByName[$_.name] = $_ }
  $after | ForEach-Object { $afterByName[$_.name] = $_ }
  $changed = @($beforeByName.Keys | Where-Object {
    -not $afterByName.ContainsKey($_) -or $beforeByName[$_].sha256 -ne $afterByName[$_].sha256
  })
  if ($changed.Count -ne 1 -or $changed[0] -ne $entryName) {
    throw "Unexpected changed ZIP entries: $($changed -join ', ')"
  }

  $panel = $afterByName[$panelName]
  if (-not $panel -or $panel.size -ne $panelSize -or $panel.sha256 -ne $panelSha256) {
    throw 'Panel v1.4.3.exe integrity check failed.'
  }
  if ($afterByName[$entryName].sha256 -ne (Get-FileHash -LiteralPath $dll -Algorithm SHA256).Hash) {
    throw 'Replacement DLL hash does not match the ZIP entry.'
  }

  [System.IO.File]::Replace($temporary, $zip, $backup)
  $result = [pscustomobject]@{
    completedAt = (Get-Date).ToString('o')
    zipPath = $zip
    zipSha256 = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
    dllPath = $dll
    dllSha256 = (Get-FileHash -LiteralPath $dll -Algorithm SHA256).Hash
    backupPath = $backup
    entryCount = $after.Count
    changedEntries = $changed
    before = $beforeByName[$entryName]
    after = $afterByName[$entryName]
    panel = $panel
  }
  $resultPath = Join-Path $reportRoot 'replacement-result.json'
  $result | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $resultPath -Encoding utf8
  $result | ConvertTo-Json -Depth 6
}
finally {
  if (Test-Path -LiteralPath $temporary) {
    Remove-Item -LiteralPath $temporary -Force
  }
}
