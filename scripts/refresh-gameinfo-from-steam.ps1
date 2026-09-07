param(
  [string]$OfficialPath = 'D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\gameinfo.gi',
  [string]$ZipPath = 'src-tauri/resources/CS2BotImprover.zip',
  [string]$ReportDirectory = 'workspace/runtime/gameinfo-refresh'
)

$ErrorActionPreference = 'Stop'
$workspace = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
$official = (Resolve-Path $OfficialPath).Path
$zip = (Resolve-Path (Join-Path $workspace $ZipPath)).Path
$officialBytes = [IO.File]::ReadAllBytes($official)
$text = [Text.Encoding]::UTF8.GetString($officialBytes)
$botLines = "`t`t`tGame`tcsgo/overrides/botprofile.vpk`r`n`t`t`tGame`tcsgo/addons/metamod"
if (($text.Split([Environment]::NewLine) | Where-Object { $_ -match 'csgo/(overrides/botprofile\.vpk|addons/metamod)' }).Count -ne 0) { throw 'Steam official baseline already contains BOT SearchPaths.' }
$matches = [regex]::Matches($text, '(?m)^(\s*Game_LowViolence\s+csgo_lv[^\r\n]*)(\r?\n)')
if ($matches.Count -ne 1) { throw 'Expected exactly one official Game_LowViolence SearchPath anchor.' }
$anchor = $matches[0].Groups[0].Value
$withBotsText = $text.Replace($anchor, "$anchor$botLines`r`n")
$withBotsBytes = [Text.Encoding]::UTF8.GetBytes($withBotsText)

function Get-Sha([byte[]]$Bytes) { $h=[Security.Cryptography.SHA256]::Create(); try { ([BitConverter]::ToString($h.ComputeHash($Bytes))).Replace('-', '') } finally { $h.Dispose() } }
$currentVersion = (Get-Content (Join-Path $workspace 'package.json') -Raw | ConvertFrom-Json).version
$manifest = [ordered]@{
  schema = 1
  resourceVersion = $currentVersion
  generatedAt = (Get-Date).ToString('yyyy-MM-dd')
  officialSha256 = Get-Sha $officialBytes
  entries = [ordered]@{
    'gameinfo.gi' = [ordered]@{ sha256 = Get-Sha $officialBytes; size = $officialBytes.Length }
    'backup/Online/gameinfo.gi' = [ordered]@{ sha256 = Get-Sha $officialBytes; size = $officialBytes.Length }
    'backup/WithBots/gameinfo.gi' = [ordered]@{ sha256 = Get-Sha $withBotsBytes; size = $withBotsBytes.Length }
  }
} | ConvertTo-Json -Depth 5

Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem
$temp = "$zip.tmp-$([guid]::NewGuid().ToString('N'))"
$backup = "$zip.before-gameinfo-refresh-$((Get-Date).ToString('yyyyMMdd-HHmmss')).bak"
Copy-Item -LiteralPath $zip -Destination $temp -Force
try {
  $archive = [IO.Compression.ZipFile]::Open($temp, [IO.Compression.ZipArchiveMode]::Update)
  try {
    $entries = @{
      'gameinfo.gi' = $officialBytes
      'backup/Online/gameinfo.gi' = $officialBytes
      'backup/WithBots/gameinfo.gi' = $withBotsBytes
      'gameinfo.manifest.json' = [Text.Encoding]::UTF8.GetBytes("$manifest`n")
    }
    foreach ($name in $entries.Keys) {
      @($archive.Entries | Where-Object FullName -eq $name) | ForEach-Object Delete
      $entry = $archive.CreateEntry($name, [IO.Compression.CompressionLevel]::Optimal)
      $stream = $entry.Open(); try { $stream.Write($entries[$name], 0, $entries[$name].Length) } finally { $stream.Dispose() }
    }
  } finally { $archive.Dispose() }
  [IO.File]::Replace($temp, $zip, $backup)
  # 资源 ZIP 变化后必须重建插件 marker，避免旧版本 marker 触发 BOT 启动版本门禁。
  & (Join-Path $PSScriptRoot 'generate-plugin-manifest.ps1') -ZipPath $ZipPath | Out-Null
  $reportRoot = Join-Path $workspace $ReportDirectory; New-Item -ItemType Directory -Force -Path $reportRoot | Out-Null
  [pscustomobject]@{ completedAt=(Get-Date).ToString('o'); officialPath=$official; onlineSha256=(Get-Sha $officialBytes); withBotsSha256=(Get-Sha $withBotsBytes); onlineBytes=$officialBytes.Length; withBotsBytes=$withBotsBytes.Length; zipSha256=(Get-FileHash $zip -Algorithm SHA256).Hash; backupPath=$backup } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $reportRoot 'result.json') -Encoding utf8
  Get-Content -Raw (Join-Path $reportRoot 'result.json')
} finally { if (Test-Path -LiteralPath $temp) { Remove-Item -LiteralPath $temp -Force } }
