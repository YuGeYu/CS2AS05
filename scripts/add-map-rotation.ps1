param(
  [string]$ZipPath = 'src-tauri/resources/CS2BotImprover.zip',
  [string]$DllPath = 'third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/bin/Release/net10.0/MapRotation.dll',
  [string]$ConfigPath = 'third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json',
  [string]$ReportDirectory = 'workspace/runtime/map-rotation'
)

$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$zip = (Resolve-Path (Join-Path $workspace $ZipPath)).Path
$dll = (Resolve-Path (Join-Path $workspace $DllPath)).Path
$config = (Resolve-Path (Join-Path $workspace $ConfigPath)).Path
$entryName = 'addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll'
$configEntryName = 'addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json'
$temporary = "$zip.tmp-$([guid]::NewGuid().ToString('N'))"
$backup = "$zip.before-map-rotation-$((Get-Date).ToString('yyyyMMdd-HHmmss')).bak"

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Get-Hash([string]$Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash }
function Get-Manifest([string]$Path) {
  $archive = [IO.Compression.ZipFile]::OpenRead($Path)
  try {
    return @($archive.Entries | ForEach-Object {
      $stream = $_.Open()
      try {
        $hash = [Security.Cryptography.SHA256]::Create()
        try { [pscustomobject]@{ name = $_.FullName; size = $_.Length; sha256 = ([BitConverter]::ToString($hash.ComputeHash($stream))).Replace('-', '') } }
        finally { $hash.Dispose() }
      } finally { $stream.Dispose() }
    })
  } finally { $archive.Dispose() }
}

$before = Get-Manifest $zip
Copy-Item -LiteralPath $zip -Destination $temporary -Force
try {
  $archive = [IO.Compression.ZipFile]::Open($temporary, [IO.Compression.ZipArchiveMode]::Update)
  try {
    $existing = @($archive.Entries | Where-Object FullName -eq $entryName)
    if ($existing.Count -gt 1) { throw "Expected at most one $entryName entry, found $($existing.Count)." }
    $existing | ForEach-Object Delete
    $entry = $archive.CreateEntry($entryName, [IO.Compression.CompressionLevel]::Optimal)
    $entry.LastWriteTime = [DateTimeOffset]::new(2026, 8, 18, 0, 0, 0, [TimeSpan]::Zero)
    $input = [IO.File]::OpenRead($dll); $output = $entry.Open()
    try { $input.CopyTo($output) } finally { $output.Dispose(); $input.Dispose() }
    $configExisting = @($archive.Entries | Where-Object FullName -eq $configEntryName)
    $configExisting | ForEach-Object Delete
    $configEntry = $archive.CreateEntry($configEntryName, [IO.Compression.CompressionLevel]::Optimal)
    $input = [IO.File]::OpenRead($config); $output = $configEntry.Open()
    try { $input.CopyTo($output) } finally { $output.Dispose(); $input.Dispose() }
  } finally { $archive.Dispose() }

  $after = Get-Manifest $temporary
  $expectedCount = $before.Count + $(if ($existing.Count -eq 0) { 1 } else { 0 }) + $(if ($configExisting.Count -eq 0) { 1 } else { 0 })
  if ($after.Count -ne $expectedCount) { throw "Unexpected ZIP entry count change: expected=$expectedCount actual=$($after.Count)." }
  $added = @($after | Where-Object name -eq $entryName)
  if ($added.Count -ne 1 -or $added[0].sha256 -ne (Get-Hash $dll)) { throw 'MapRotation DLL hash verification failed.' }
  [IO.File]::Replace($temporary, $zip, $backup)
  $reportRoot = Join-Path $workspace $ReportDirectory
  New-Item -ItemType Directory -Force -Path $reportRoot | Out-Null
  [pscustomobject]@{ completedAt = (Get-Date).ToString('o'); entry = $entryName; dllSha256 = Get-Hash $dll; zipSha256 = Get-Hash $zip; backupPath = $backup } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $reportRoot 'result.json') -Encoding utf8
  Get-Content -LiteralPath (Join-Path $reportRoot 'result.json')
} finally {
  if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
}

