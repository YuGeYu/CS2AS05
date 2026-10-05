param(
  [string]$ZipPath = 'src-tauri/resources/CS2BotImprover.zip',
  [string]$ReportDirectory = 'workspace/runtime/plugin-manifest'
)

# v1.4.5 is shipped byte-for-byte from ed0ard/CS2-Bot-Improver. This legacy
# helper is now a read-only verifier; it must never append a downstream marker
# or mutate the official archive.
# Report contract: fixedZipHashGuard=$false (the archive is verified, never rewritten).
$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$zip = (Resolve-Path (Join-Path $workspace $ZipPath)).Path
$reportRoot = Join-Path $workspace $ReportDirectory
New-Item -ItemType Directory -Force -Path $reportRoot | Out-Null

Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [IO.Compression.ZipFile]::OpenRead($zip)
try {
  $names = @($archive.Entries | Where-Object { -not $_.FullName.EndsWith('/') } | ForEach-Object { $_.FullName.TrimStart('./') })
  $required = @(
    'Panel v1.4.5.exe',
    'gameinfo.gi',
    'backup/Online/gameinfo.gi',
    'backup/WithBots/gameinfo.gi',
    'addons/counterstrikesharp/plugins/BotAI/BotAI.dll',
    'addons/counterstrikesharp/plugins/BotRandomizer/BotRandomizer.dll',
    'addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll'
  )
  $missing = @($required | Where-Object { $names -notcontains $_ })
  if ($missing.Count -gt 0) { throw "Official v1.4.5 archive is missing: $($missing -join ', ')" }
  $forbidden = @($names | Where-Object { $_ -like 'addons/counterstrikesharp/plugins/MapRotation/*' -or $_ -like 'addons/counterstrikesharp/plugins/CS2BotLlmChat/*' -or $_ -like 'addons/counterstrikesharp/configs/plugins/MapRotation/*' -or $_ -like 'addons/counterstrikesharp/configs/plugins/CS2BotLlmChat/*' })
  if ($forbidden.Count -gt 0) { throw "Archive contains retired downstream components: $($forbidden -join ', ')" }
  $sha = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash
  [pscustomobject]@{
    completedAt = (Get-Date).ToString('o')
    source = 'ed0ard/CS2-Bot-Improver v1.4.5'
    entries = $names.Count
    zipSha256 = $sha
    fixedZipHashGuard = $false
    mutated = $false
  } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $reportRoot 'result.json') -Encoding utf8
} finally {
  $archive.Dispose()
}
Get-Content -LiteralPath (Join-Path $reportRoot 'result.json')
