param(
  [Parameter(Mandatory)][string]$Root,
  [ValidateSet('Snapshot','Diff','Restore')][string]$Action = 'Snapshot',
  [string]$Name = 'baseline'
)

$ErrorActionPreference = 'Stop'
$resolved = (Resolve-Path -LiteralPath $Root).Path
$csgo = if (Test-Path -LiteralPath (Join-Path $resolved 'game\csgo')) { Join-Path $resolved 'game\csgo' } else { $resolved }
if (!(Test-Path -LiteralPath (Join-Path $csgo 'gameinfo.gi'))) { throw "目标不是有效 game/csgo：$csgo" }
$evidence = Join-Path $PSScriptRoot "..\workspace\panel-diff\$Name"
$snapshot = Join-Path $evidence 'snapshot.json'
$content = Join-Path $evidence 'content'

function Get-Tree([string]$Path) {
  Get-ChildItem -LiteralPath $Path -Recurse -File | ForEach-Object {
    [pscustomobject]@{ Path = $_.FullName.Substring($Path.Length + 1).Replace('\','/'); Size = $_.Length; SHA256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
  } | Sort-Object Path
}

if ($Action -eq 'Snapshot') {
  New-Item -ItemType Directory -Force $content | Out-Null
  Get-Tree $csgo | ConvertTo-Json -Depth 3 | Set-Content -LiteralPath $snapshot -Encoding utf8
  foreach ($file in Get-ChildItem -LiteralPath $csgo -Recurse -File | Where-Object Extension -in '.cfg','.json','.gi','.txt') {
    $relative = $file.FullName.Substring($csgo.Length + 1)
    $target = Join-Path $content $relative
    New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
    Copy-Item -LiteralPath $file.FullName -Destination $target -Force
  }
  Write-Output "快照：$snapshot"
  exit
}
if (!(Test-Path -LiteralPath $snapshot)) { throw "缺少基线快照：$snapshot" }
if ($Action -eq 'Diff') {
  $before = Get-Content -Raw $snapshot | ConvertFrom-Json
  $after = Get-Tree $csgo
  Compare-Object $before $after -Property Path,Size,SHA256 | Sort-Object Path | Format-Table -AutoSize
  exit
}
foreach ($file in Get-ChildItem -LiteralPath $content -Recurse -File) {
  $relative = $file.FullName.Substring($content.Length + 1)
  $target = Join-Path $csgo $relative
  New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
  Copy-Item -LiteralPath $file.FullName -Destination $target -Force
}
Write-Output "已恢复文本契约文件。二进制文件需从受摘要保护的 ZIP 重新提取。"
