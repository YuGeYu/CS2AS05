param(
  [Parameter(Mandatory = $true)][string]$ZipPath,
  [Parameter(Mandatory = $true)][string]$ArtifactDir,
  [Parameter(Mandatory = $true)][string]$BuildDir,
  [Parameter(Mandatory = $true)][string]$ConfigPath
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
New-Item -ItemType Directory -Path $ArtifactDir -Force | Out-Null
$zipPath = (Resolve-Path $ZipPath).Path
Copy-Item $zipPath (Join-Path $ArtifactDir 'CS2BotImprover.zip.before') -Force
$tmp = Join-Path $ArtifactDir 'CS2BotImprover.zip.tmp'
$entries = @{
  'addons/counterstrikesharp/plugins/CS2BotLlmChat/CS2BotLlmChat.dll' = Join-Path $BuildDir 'CS2BotLlmChat.dll'
  'addons/counterstrikesharp/plugins/CS2BotLlmChat/CS2BotLlmChat.deps.json' = Join-Path $BuildDir 'CS2BotLlmChat.deps.json'
  'addons/counterstrikesharp/plugins/CS2BotLlmChat/CS2BotLlmChat.pdb' = Join-Path $BuildDir 'CS2BotLlmChat.pdb'
  'addons/counterstrikesharp/configs/plugins/CS2BotLlmChat/CS2BotLlmChat.json' = $ConfigPath
}
$source = [IO.Compression.ZipFile]::OpenRead($zipPath)
$target = [IO.Compression.ZipFile]::Open($tmp, [IO.Compression.ZipArchiveMode]::Create)
try {
  foreach ($entry in $source.Entries) {
    if ($entries.ContainsKey($entry.FullName)) { continue }
    $outEntry = $target.CreateEntry($entry.FullName, [IO.Compression.CompressionLevel]::Optimal)
    $input = $entry.Open(); $output = $outEntry.Open()
    try { $input.CopyTo($output) } finally { $output.Dispose(); $input.Dispose() }
  }
  foreach ($entry in $entries.GetEnumerator()) {
    if (!(Test-Path -LiteralPath $entry.Value -PathType Leaf)) { throw "Missing build output: $($entry.Value)" }
    $outEntry = $target.CreateEntry($entry.Key, [IO.Compression.CompressionLevel]::Optimal)
    $bytes = [IO.File]::ReadAllBytes((Resolve-Path $entry.Value).Path); $output = $outEntry.Open()
    try { $output.Write($bytes, 0, $bytes.Length) } finally { $output.Dispose() }
  }
} finally { $target.Dispose(); $source.Dispose() }
Move-Item $tmp $zipPath -Force
$lines = @(); $zipInfo = Get-Item $zipPath
$lines += "ZIP`t$($zipInfo.Length)`t$((Get-FileHash $zipPath -Algorithm SHA256).Hash)"
foreach ($entry in $entries.GetEnumerator()) { $info = Get-Item $entry.Value; $lines += "$($entry.Key)`t$($info.Length)`t$((Get-FileHash $entry.Value -Algorithm SHA256).Hash)" }
$lines | Set-Content (Join-Path $ArtifactDir 'candidate-sha256.txt')
$lines
