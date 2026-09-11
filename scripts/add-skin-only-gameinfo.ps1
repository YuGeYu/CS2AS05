param(
  [string]$ZipPath = 'src-tauri/resources/CS2BotImprover.zip',
  [Parameter(Mandatory = $true)][string]$SkinOnlyPath,
  [string]$ArtifactDir = 'artifacts/skin-only-gameinfo-candidate'
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = (Resolve-Path $ZipPath).Path
$skin = (Resolve-Path $SkinOnlyPath).Path
$artifact = Join-Path (Resolve-Path '.').Path $ArtifactDir
New-Item -ItemType Directory -Force $artifact | Out-Null
Copy-Item $zip (Join-Path $artifact 'CS2BotImprover.zip.before') -Force
$tmp = Join-Path $artifact 'CS2BotImprover.zip.tmp'
$src = [IO.Compression.ZipFile]::OpenRead($zip)
$dst = [IO.Compression.ZipFile]::Open($tmp, [IO.Compression.ZipArchiveMode]::Create)
try {
  foreach ($e in $src.Entries) {
    if ($e.FullName -in @('backup/SkinOnly/gameinfo.gi','gameinfo.manifest.json')) { continue }
    $o = $dst.CreateEntry($e.FullName, [IO.Compression.CompressionLevel]::Optimal)
    $i = $e.Open(); $s = $o.Open()
    try { $i.CopyTo($s) } finally { $s.Dispose(); $i.Dispose() }
  }
  $o = $dst.CreateEntry('backup/SkinOnly/gameinfo.gi', [IO.Compression.CompressionLevel]::Optimal)
  $bytes = [IO.File]::ReadAllBytes($skin); $s = $o.Open()
  try { $s.Write($bytes, 0, $bytes.Length) } finally { $s.Dispose() }
  $manifest = [ordered]@{ schema = 1; resourceVersion = '0.5.13'; officialSha256 = $null; entries = [ordered]@{} }
  $needed = @('gameinfo.gi','backup/Online/gameinfo.gi','backup/WithBots/gameinfo.gi','backup/SkinOnly/gameinfo.gi')
  foreach ($name in $needed) {
    $old = $src.GetEntry($name)
    if ($name -eq 'backup/SkinOnly/gameinfo.gi') { $data = $bytes } else { $stream=$old.Open();$ms=[IO.MemoryStream]::new();$stream.CopyTo($ms);$stream.Dispose();$data=$ms.ToArray();$ms.Dispose() }
    $manifest.entries[$name] = [ordered]@{ sha256 = ([BitConverter]::ToString(([Security.Cryptography.SHA256]::Create().ComputeHash($data))) -replace '-',''); size = $data.Length }
    if ($name -eq 'gameinfo.gi') { $manifest.officialSha256 = $manifest.entries[$name].sha256 }
  }
  $json = ($manifest | ConvertTo-Json -Depth 6)
  $o = $dst.CreateEntry('gameinfo.manifest.json', [IO.Compression.CompressionLevel]::Optimal); $w=[IO.StreamWriter]::new($o.Open(),[Text.UTF8Encoding]::new($false)); try{$w.Write($json)}finally{$w.Dispose()}
} finally { $dst.Dispose(); $src.Dispose() }
Move-Item $tmp $zip -Force
$zipInfo = Get-Item $zip
"ZIP`t$($zipInfo.Length)`t$((Get-FileHash $zip -Algorithm SHA256).Hash)" | Set-Content (Join-Path $artifact 'candidate-sha256.txt')
"SkinOnly`t$((Get-Item $skin).Length)`t$((Get-FileHash $skin -Algorithm SHA256).Hash)" | Add-Content (Join-Path $artifact 'candidate-sha256.txt')
Get-Content (Join-Path $artifact 'candidate-sha256.txt')
