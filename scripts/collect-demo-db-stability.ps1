param(
    [Parameter(Mandatory = $true)][string]$DbPath,
    [Parameter(Mandatory = $true)][string]$SqlitePath,
    [Parameter(Mandatory = $true)][string]$EvidenceDir
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-Sha256([string]$Path) {
    $stream = [IO.File]::OpenRead($Path)
    try {
        $hash = [Security.Cryptography.SHA256]::Create()
        try { return -join ($hash.ComputeHash($stream) | ForEach-Object { $_.ToString('X2') }) }
        finally { $hash.Dispose() }
    } finally { $stream.Dispose() }
}
function File-State([string]$Path) {
    if (-not [IO.File]::Exists($Path)) { return [ordered]@{ exists = $false; bytes = 0; mtimeUtc = $null; sha256 = $null } }
    $item = [IO.FileInfo]::new($Path)
    [ordered]@{ exists = $true; bytes = $item.Length; mtimeUtc = $item.LastWriteTimeUtc.ToString('o'); sha256 = Get-Sha256 $Path }
}

if (-not [IO.File]::Exists($DbPath)) { throw "DB not found: $DbPath" }
if (-not [IO.File]::Exists($SqlitePath)) { throw "sqlite3 not found: $SqlitePath" }
$writers = @(Get-Process ai_pc_fac -ErrorAction SilentlyContinue)
if ($writers.Count) { throw "ai_pc_fac is still running: $($writers.Id -join ',')" }
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null
$samples = [Collections.Generic.List[object]]::new()
for ($index = 0; $index -lt 6; $index++) {
    $samples.Add([ordered]@{ index = $index + 1; timestamp = (Get-Date).ToString('o'); main = File-State $DbPath; wal = File-State "$DbPath-wal"; shm = File-State "$DbPath-shm" })
    if ($index -lt 5) { Start-Sleep -Seconds 2 }
}
$first = $samples[0]
$mainStable = -not @($samples | Where-Object { $_.main.bytes -ne $first.main.bytes -or $_.main.mtimeUtc -ne $first.main.mtimeUtc -or $_.main.sha256 -ne $first.main.sha256 }).Count
$walNonZero = @($samples | Where-Object { $_.wal.exists -and $_.wal.bytes -gt 0 }).Count -gt 0
$copyDir = Join-Path $EvidenceDir 'diagnostic-copy'; New-Item -ItemType Directory -Force -Path $copyDir | Out-Null
$copyDb = Join-Path $copyDir ([IO.Path]::GetFileName($DbPath))
Copy-Item -LiteralPath $DbPath -Destination $copyDb
foreach ($suffix in @('-wal','-shm')) { if ([IO.File]::Exists("$DbPath$suffix")) { Copy-Item -LiteralPath "$DbPath$suffix" -Destination "$copyDb$suffix" } }
$readback = $null
if (-not $walNonZero) {
    $integrity = (& $SqlitePath -readonly $copyDb 'PRAGMA integrity_check;').Trim()
    $version = [int](& $SqlitePath -readonly $copyDb 'PRAGMA user_version;')
    $maps = [int](& $SqlitePath -readonly $copyDb "SELECT count(*) FROM map_metadata WHERE source='cs-demo-manager';")
    $active = [int](& $SqlitePath -readonly $copyDb "SELECT count(*) FROM analysis_jobs WHERE stage NOT IN ('done','spatial_done','error','canceled');")
    $leases = [int](& $SqlitePath -readonly $copyDb "SELECT count(*) FROM analysis_jobs WHERE lease_owner IS NOT NULL OR lease_until IS NOT NULL;")
    $readback = [ordered]@{ path = $copyDb; sha256BeforeRead = Get-Sha256 $copyDb; integrityCheck = $integrity; userVersion = $version; mapMetadataCount = $maps; activeJobs = $active; activeLeases = $leases }
}
$result = [ordered]@{
    generatedAt = (Get-Date).ToString('o')
    liveFileStability = [ordered]@{ path = [IO.Path]::GetFullPath($DbPath); writerPids = @(); samples = $samples; mainStable = $mainStable; walNonZero = $walNonZero; passed = ($mainStable -and -not $walNonZero) }
    diagnosticCopySqliteReadback = $readback
    passed = ($mainStable -and -not $walNonZero -and $null -ne $readback -and $readback.integrityCheck -eq 'ok' -and $readback.userVersion -eq 7 -and $readback.mapMetadataCount -eq 44 -and $readback.activeJobs -eq 0 -and $readback.activeLeases -eq 0)
}
$output = Join-Path $EvidenceDir 'database-stability.json'
$result | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $output -Encoding utf8
$result | ConvertTo-Json -Depth 5
if (-not $result.passed) { throw "database stability gate failed; see $output" }

