param(
    [string]$ZipPath = "src-tauri/resources/CS2BotImprover.zip",
    [string]$ReportPath = "docs/CS2BotImprover-defaults-diff-0.5.4.json"
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.IO.Compression
Add-Type -AssemblyName System.IO.Compression.FileSystem

$PanelHash = "3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD"
$Targets = @(
    "cfg/my_bot_normal_config.cfg",
    "cfg/my_bot_ffa_config.cfg"
)

function Get-EntryBytes($Archive, [string]$Name) {
    $entry = $Archive.GetEntry($Name)
    if ($null -eq $entry) { return $null }
    $stream = $entry.Open()
    try {
        $memory = [IO.MemoryStream]::new()
        $stream.CopyTo($memory)
        return $memory.ToArray()
    } finally {
        $stream.Dispose()
    }
}

function Get-BytesHash([byte[]]$Bytes) {
    if ($null -eq $Bytes) { return $null }
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return [Convert]::ToHexString($sha.ComputeHash($Bytes)) } finally { $sha.Dispose() }
}

function Get-EntryInfo($Archive, [string]$Name) {
    $bytes = Get-EntryBytes $Archive $Name
    [ordered]@{ name = $Name; exists = $null -ne $bytes; size = if ($bytes) { $bytes.Length } else { 0 }; sha256 = Get-BytesHash $bytes }
}

function Set-EntryBytes($Archive, [string]$Name, [byte[]]$Bytes) {
    $existing = $Archive.GetEntry($Name)
    if ($existing) { $existing.Delete() }
    $entry = $Archive.CreateEntry($Name, [IO.Compression.CompressionLevel]::Optimal)
    $entry.LastWriteTime = [DateTimeOffset]::new(2026, 7, 26, 0, 0, 0, [TimeSpan]::Zero)
    $stream = $entry.Open()
    try { $stream.Write($Bytes, 0, $Bytes.Length) } finally { $stream.Dispose() }
}

function Set-ManagedCfgDefaults([byte[]]$Bytes) {
    $text = [Text.Encoding]::UTF8.GetString($Bytes)
    $newline = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
    $lines = $text -split "`r?`n" | Where-Object {
        $_ -notmatch '^\s*bot_aim\s+(head|mixed|body)\s*$' -and
        $_ -notmatch '^\s*bot_nades\s+(max|more|normal|less|off)\s*$'
    }
    $aliasIndex = [Array]::FindIndex([string[]]$lines, [Predicate[string]]{ param($line) $line -match '^alias\s+bot_aim_' })
    if ($aliasIndex -lt 0) { $aliasIndex = $lines.Count }
    $updated = @($lines[0..([Math]::Max(0, $aliasIndex - 1))]) + @("bot_aim mixed", "bot_nades normal", "") + @($lines[$aliasIndex..($lines.Count - 1)])
    return [Text.UTF8Encoding]::new($false).GetBytes(($updated -join $newline).TrimEnd() + $newline)
}

$resolved = (Resolve-Path $ZipPath).Path
$temporary = "$resolved.tmp-$PID"
Copy-Item -LiteralPath $resolved -Destination $temporary
(Get-Item -LiteralPath $resolved).IsReadOnly = $false
(Get-Item -LiteralPath $temporary).IsReadOnly = $false

$beforeArchive = [IO.Compression.ZipFile]::OpenRead($resolved)
try {
    $before = @($Targets | ForEach-Object { Get-EntryInfo $beforeArchive $_ })
    $panelBefore = Get-EntryBytes $beforeArchive "Panel v1.4.3.exe"
} finally { $beforeArchive.Dispose() }

try {
    $archive = [IO.Compression.ZipFile]::Open($temporary, [IO.Compression.ZipArchiveMode]::Update)
    try {
        foreach ($cfg in @("cfg/my_bot_normal_config.cfg", "cfg/my_bot_ffa_config.cfg")) {
            Set-EntryBytes $archive $cfg (Set-ManagedCfgDefaults (Get-EntryBytes $archive $cfg))
        }
    } finally { $archive.Dispose() }

    $afterArchive = [IO.Compression.ZipFile]::OpenRead($temporary)
    try {
        $after = @($Targets | ForEach-Object { Get-EntryInfo $afterArchive $_ })
        $panelAfter = Get-EntryBytes $afterArchive "Panel v1.4.3.exe"
        if ((Get-BytesHash $panelBefore) -ne $PanelHash -or (Get-BytesHash $panelAfter) -ne $PanelHash) {
            throw "Panel v1.4.3.exe changed or did not match the pinned SHA256"
        }
    } finally { $afterArchive.Dispose() }

    $report = [ordered]@{
        generatedAt = (Get-Date).ToUniversalTime().ToString("o")
        source = "ed0ard/CS2-Bot-Improver v1.4.3 official Windows package"
        beforeZipSha256 = (Get-FileHash -LiteralPath $resolved -Algorithm SHA256).Hash
        afterZipSha256 = (Get-FileHash -LiteralPath $temporary -Algorithm SHA256).Hash
        panelSha256 = $PanelHash
        entries = for ($index = 0; $index -lt $Targets.Count; $index++) { [ordered]@{ before = $before[$index]; after = $after[$index] } }
    }
    $reportJson = $report | ConvertTo-Json -Depth 8
    $reportDirectory = Split-Path -Parent $ReportPath
    if ($reportDirectory) { New-Item -ItemType Directory -Force $reportDirectory | Out-Null }
    [IO.File]::WriteAllText((Join-Path (Get-Location) $ReportPath), $reportJson + "`n", [Text.UTF8Encoding]::new($false))
    Move-Item -LiteralPath $temporary -Destination $resolved -Force
    $reportJson
} finally {
    if (Test-Path -LiteralPath $temporary) { Remove-Item -LiteralPath $temporary -Force }
}
