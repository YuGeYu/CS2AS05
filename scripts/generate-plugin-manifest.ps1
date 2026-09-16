param(
  [string]$ZipPath = 'src-tauri/resources/CS2BotImprover.zip',
  [string]$ReportDirectory = 'workspace/runtime/plugin-manifest'
)

$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$zip = (Resolve-Path (Join-Path $workspace $ZipPath)).Path
$version = (Get-Content (Join-Path $workspace 'package.json') -Raw | ConvertFrom-Json).version
$markerName = 'addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json'
$panelName = 'Panel v1.4.4.exe'
$mapRotationEntry = 'addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll'
$mapRotationConfigEntry = 'addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json'
$rustServicePath = Join-Path $workspace 'src-tauri/src/services/cs2.rs'
$temporary = "$zip.tmp-$([guid]::NewGuid().ToString('N'))"
$backup = "$zip.before-plugin-manifest-$((Get-Date).ToString('yyyyMMdd-HHmmss')).bak"

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Get-Hex([byte[]]$Bytes) { ([BitConverter]::ToString($Bytes)).Replace('-', '') }
function Add-Text([Security.Cryptography.HashAlgorithm]$Hash, [string]$Text) {
  $bytes = [Text.Encoding]::UTF8.GetBytes($Text)
  [void]$Hash.TransformBlock($bytes, 0, $bytes.Length, $bytes, 0)
}
function Get-Payload([System.IO.Compression.ZipArchive]$Archive) {
  $hash = [Security.Cryptography.SHA256]::Create()
  try {
    $entries = @($Archive.Entries | Where-Object {
      -not $_.FullName.EndsWith('/') -and
      $_.FullName -ne $markerName -and
      ($_.FullName.StartsWith('addons/counterstrikesharp/plugins/NadeSystem/') -or $_.FullName.StartsWith('addons/counterstrikesharp/plugins/CS2BotLlmChat/') -or $_.FullName -eq $mapRotationEntry)
    } | Sort-Object FullName)
    foreach ($entry in $entries) {
      Add-Text $hash "$($entry.FullName)`0$($entry.Length)`0"
      $stream = $entry.Open()
      try {
        $buffer = New-Object byte[] 65536
        while (($read = $stream.Read($buffer, 0, $buffer.Length)) -gt 0) { [void]$hash.TransformBlock($buffer, 0, $read, $buffer, 0) }
      } finally { $stream.Dispose() }
    }
    [void]$hash.TransformFinalBlock(@(), 0, 0)
    return [pscustomobject]@{ sha256 = Get-Hex $hash.Hash; entries = @($entries | ForEach-Object FullName) }
  } finally { $hash.Dispose() }
}
function Get-FileSha256([string]$Path) {
  $hash = [Security.Cryptography.SHA256]::Create(); $stream = [IO.File]::OpenRead($Path)
  try { Get-Hex $hash.ComputeHash($stream) } finally { $stream.Dispose(); $hash.Dispose() }
}

Copy-Item -LiteralPath $zip -Destination $temporary -Force
(Get-Item -LiteralPath $zip).IsReadOnly = $false
(Get-Item -LiteralPath $temporary).IsReadOnly = $false
try {
  $archive = [IO.Compression.ZipFile]::Open($temporary, [IO.Compression.ZipArchiveMode]::Update)
  try {
    @($archive.Entries | Where-Object FullName -eq $markerName) | ForEach-Object Delete
    $payload = Get-Payload $archive
    $manifest = [ordered]@{
      schema = 2
      product = 'cs2-bot-improver'
      pluginId = 'cs2as05-custom-package'
      version = $version
      components = @(
        [ordered]@{
          id = 'cs2as05-custom-package'
          source = 'CS2-Bot-Improver-v1.4.4'
        }
      )
      payloadSha256 = $payload.sha256
      payloadEntries = $payload.entries
      mutableConfigEntries = @($mapRotationConfigEntry)
    } | ConvertTo-Json -Depth 4
    $entry = $archive.CreateEntry($markerName, [IO.Compression.CompressionLevel]::Optimal)
    $entry.LastWriteTime = [DateTimeOffset]::new(2026, 7, 26, 0, 0, 0, [TimeSpan]::Zero)
    $writer = [IO.StreamWriter]::new($entry.Open(), [Text.UTF8Encoding]::new($false))
    try { $writer.Write($manifest) } finally { $writer.Dispose() }
  } finally { $archive.Dispose() }
  $check = [IO.Compression.ZipFile]::OpenRead($temporary)
  try {
    $markerEntry = $check.Entries | Where-Object FullName -eq $markerName
    $reader = [IO.StreamReader]::new($markerEntry.Open())
    try { $checkManifest = $reader.ReadToEnd() | ConvertFrom-Json } finally { $reader.Dispose() }
    if ($checkManifest.payloadEntries -contains $mapRotationConfigEntry -or $checkManifest.mutableConfigEntries -notcontains $mapRotationConfigEntry) { throw 'MapRotation.json must be declared only in mutableConfigEntries.' }
  } finally { $check.Dispose() }
  [IO.File]::Replace($temporary, $zip, $backup)
  $zipSha256 = Get-FileSha256 $zip
  $rustService = Get-Content -LiteralPath $rustServicePath -Raw
  $zipConstantPattern = 'const CUSTOM_ZIP_SHA256: &str = "[0-9A-F]{64}";'
  if ([regex]::Matches($rustService, $zipConstantPattern).Count -ne 1) {
    throw 'Expected exactly one CUSTOM_ZIP_SHA256 constant in src-tauri/src/services/cs2.rs.'
  }
  $rustService = [regex]::Replace(
    $rustService,
    $zipConstantPattern,
    "const CUSTOM_ZIP_SHA256: &str = `"$zipSha256`";"
  )
  Set-Content -LiteralPath $rustServicePath -Value $rustService -Encoding utf8NoBOM -NoNewline
  $reportRoot = Join-Path $workspace $ReportDirectory; New-Item -ItemType Directory -Force -Path $reportRoot | Out-Null
  [pscustomobject]@{ completedAt=(Get-Date).ToString('o'); version=$version; marker=$markerName; manifest=($manifest | ConvertFrom-Json); zipSha256=$zipSha256; rustHashSynchronized=$true; backupPath=$backup } | ConvertTo-Json -Depth 8 | Tee-Object -FilePath (Join-Path $reportRoot 'result.json')
} finally { if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force } }
