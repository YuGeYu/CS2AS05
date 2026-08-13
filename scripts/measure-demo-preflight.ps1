param(
    [Parameter(Mandatory = $true)]
    [string]$DbPath,
    [Parameter(Mandatory = $true)]
    [string]$SqlitePath,
    [Parameter(Mandatory = $true)]
    [string]$EvidenceDir,
    [int]$TargetProcessId = 0,
    [string]$RuntimeEvidenceDir = '',
    [string]$WindowEvidenceDir = '',
    [string]$DbStabilityJson = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Get-Percentile95([double[]]$Values) {
    if ($Values.Count -eq 0) { return $null }
    $ordered = @($Values | Sort-Object)
    $index = [Math]::Max(0, [Math]::Ceiling($ordered.Count * 0.95) - 1)
    return [Math]::Round($ordered[$index], 3)
}

function Get-Median([double[]]$Values) {
    if ($Values.Count -eq 0) { return $null }
    $ordered = @($Values | Sort-Object)
    return [Math]::Round($ordered[[Math]::Max(0, [Math]::Ceiling($ordered.Count * 0.5) - 1)], 3)
}

function Get-Slope([double[]]$Values) {
    if ($Values.Count -lt 2) { return $null }
    $n = $Values.Count; $sumX = $n * ($n + 1) / 2.0; $sumY = 0.0; $sumXY = 0.0; $sumXX = $n * ($n + 1) * (2 * $n + 1) / 6.0
    for ($index = 0; $index -lt $n; $index++) { $sumY += $Values[$index]; $sumXY += ($index + 1) * $Values[$index] }
    return [Math]::Round(($n * $sumXY - $sumX * $sumY) / ($n * $sumXX - $sumX * $sumX), 3)
}

function Get-Trend([double[]]$Values) {
    if ($Values.Count -eq 0) { return $null }
    $first = @($Values | Select-Object -First ([Math]::Min(5, $Values.Count)))
    $last = @($Values | Select-Object -Last ([Math]::Min(5, $Values.Count)))
    [ordered]@{ first = $Values[0]; last = $Values[-1]; min = ($Values | Measure-Object -Minimum).Minimum; max = ($Values | Measure-Object -Maximum).Maximum; delta = $Values[-1] - $Values[0]; slope = Get-Slope $Values; first5Median = Get-Median $first; last5Median = Get-Median $last }
}

function Measure-SqliteQuery([string]$Database, [string]$Sql, [int]$Iterations = 30) {
    $values = [System.Collections.Generic.List[double]]::new()
    1..$Iterations | ForEach-Object {
        $timer = [System.Diagnostics.Stopwatch]::StartNew()
        & $SqlitePath -readonly $Database $Sql | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "sqlite query failed with exit code $LASTEXITCODE" }
        $timer.Stop()
        $values.Add($timer.Elapsed.TotalMilliseconds)
    }
    return [pscustomobject]@{
        iterations = $Iterations
        minMs = [Math]::Round(($values | Measure-Object -Minimum).Minimum, 3)
        medianMs = [Math]::Round(($values | Sort-Object)[[Math]::Floor($values.Count / 2)], 3)
        p95Ms = Get-Percentile95 $values.ToArray()
        maxMs = [Math]::Round(($values | Measure-Object -Maximum).Maximum, 3)
    }
}

if (-not (Test-Path -LiteralPath $DbPath -PathType Leaf)) { throw "DB not found: $DbPath" }
if (-not (Test-Path -LiteralPath $SqlitePath -PathType Leaf)) { throw "sqlite3 not found: $SqlitePath" }
New-Item -ItemType Directory -Force -Path $EvidenceDir | Out-Null

$fixturePath = Join-Path ([System.IO.Path]::GetTempPath()) ("cs2as-demo-library-{0}.sqlite3" -f [guid]::NewGuid())
try {
    & $SqlitePath $fixturePath @'
CREATE TABLE demo_files(
  id INTEGER PRIMARY KEY,
  file_name TEXT NOT NULL,
  map_name TEXT,
  mtime_ms INTEGER NOT NULL,
  status TEXT NOT NULL,
  size_bytes INTEGER NOT NULL
);
WITH RECURSIVE fixture(id) AS (
  SELECT 1 UNION ALL SELECT id + 1 FROM fixture WHERE id < 500
)
INSERT INTO demo_files(id,file_name,map_name,mtime_ms,status,size_bytes)
SELECT id, printf('fixture-%04d.dem',id), 'de_dust2', 2000000000000-id, 'done', 1048576+id FROM fixture;
CREATE INDEX demo_files_mtime ON demo_files(mtime_ms DESC);
'@ | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "fixture creation failed with exit code $LASTEXITCODE" }

    $libraryQuery = Measure-SqliteQuery $fixturePath "SELECT id,file_name,map_name,mtime_ms,status,size_bytes FROM demo_files ORDER BY mtime_ms DESC LIMIT 50 OFFSET 200;"
    $heatmapQuery = Measure-SqliteQuery $DbPath "SELECT e.tick,e.round_number,COALESCE(e.actor_key,e.target_key),CAST(json_extract(e.payload_json,'$.payload.x') AS REAL),CAST(json_extract(e.payload_json,'$.payload.y') AS REAL),CAST(json_extract(e.payload_json,'$.payload.z') AS REAL) FROM match_events e WHERE e.demo_id=4 AND e.kind='smokegrenade_detonate' AND json_extract(e.payload_json,'$.payload.x') IS NOT NULL AND json_extract(e.payload_json,'$.payload.y') IS NOT NULL ORDER BY e.tick;"
} finally {
    if (Test-Path -LiteralPath $fixturePath) { Remove-Item -LiteralPath $fixturePath -Force }
}

$integrity = (& $SqlitePath -readonly $DbPath "PRAGMA integrity_check;").Trim()
$userVersion = [int](& $SqlitePath -readonly $DbPath "PRAGMA user_version;")
$mapCount = [int](& $SqlitePath -readonly $DbPath "SELECT count(*) FROM map_metadata WHERE source='cs-demo-manager';")
$jobJson = & $SqlitePath -readonly -json $DbPath "SELECT kind,stage,created_at,started_at,finished_at,CASE WHEN started_at IS NOT NULL AND finished_at IS NOT NULL THEN finished_at-started_at END elapsed_ms FROM analysis_jobs ORDER BY id;"
$jobText = $jobJson -join "`n"
$jobs = if ([string]::IsNullOrWhiteSpace($jobText)) { @() } else { @($jobText | ConvertFrom-Json) }

$process = $null
if ($TargetProcessId -gt 0) {
    $candidate = Get-Process -Id $TargetProcessId -ErrorAction SilentlyContinue
    if ($candidate) {
        $process = [pscustomobject]@{
            pid = $candidate.Id
            startTime = $candidate.StartTime.ToString('o')
            cpuSeconds = [Math]::Round($candidate.CPU, 3)
            workingSetBytes = $candidate.WorkingSet64
            privateMemoryBytes = $candidate.PrivateMemorySize64
            handles = $candidate.HandleCount
        }
    }
}

$exePath = Join-Path $PSScriptRoot '..\src-tauri\target\debug\ai_pc_fac.exe'
$exe = $null
if (Test-Path -LiteralPath $exePath -PathType Leaf) {
    $exeFile = Get-Item -LiteralPath $exePath
    $exe = [pscustomobject]@{
        path = $exeFile.FullName
        bytes = $exeFile.Length
        mtimeUtc = $exeFile.LastWriteTimeUtc.ToString('o')
        sha256 = (Get-FileHash -LiteralPath $exeFile.FullName -Algorithm SHA256).Hash
    }
}

$screenshots = @(Get-ChildItem -LiteralPath $EvidenceDir -Recurse -File -ErrorAction SilentlyContinue |
    Where-Object { $_.Extension -match '^\.(png|jpg|jpeg)$' } |
    ForEach-Object {
        [pscustomobject]@{
            path = $_.FullName
            bytes = $_.Length
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash
            width = if (Test-Path -LiteralPath ([IO.Path]::ChangeExtension($_.FullName, '.json'))) { (Get-Content -LiteralPath ([IO.Path]::ChangeExtension($_.FullName, '.json')) -Raw | ConvertFrom-Json).width } else { $null }
            height = if (Test-Path -LiteralPath ([IO.Path]::ChangeExtension($_.FullName, '.json'))) { (Get-Content -LiteralPath ([IO.Path]::ChangeExtension($_.FullName, '.json')) -Raw | ConvertFrom-Json).height } else { $null }
        }
    })

$cold = $null; $positions = $null; $cpu = $null; $lifecycle = $null
if ($RuntimeEvidenceDir) {
    $coldRaw = @(Get-Content -LiteralPath (Join-Path $RuntimeEvidenceDir 'cold-starts.json') -Raw | ConvertFrom-Json)
    $coldValues = [double[]]@($coldRaw | ForEach-Object { $_.coldStartInteractiveMs })
    $cold = [ordered]@{ samples = $coldRaw; medianMs = Get-Median $coldValues; p95Ms = Get-Percentile95 $coldValues }
    $positionRaw = Get-Content -LiteralPath (Join-Path $RuntimeEvidenceDir 'round-positions.json') -Raw | ConvertFrom-Json
    $positionValues = [double[]]@($positionRaw.samples | ForEach-Object { $_.elapsedMs })
    $positions = [ordered]@{ demoId = $positionRaw.demoId; roundNumber = $positionRaw.roundNumber; samplingHz = $positionRaw.samplingHz; warmups = $positionRaw.warmups; iterations = $positionRaw.iterations; samples = $positionRaw.samples; p95Ms = Get-Percentile95 $positionValues }
    $cpu = Get-Content -LiteralPath (Join-Path $RuntimeEvidenceDir 'cpu-and-fps.json') -Raw | ConvertFrom-Json
    $lifeRaw = Get-Content -LiteralPath (Join-Path $RuntimeEvidenceDir 'lifecycle-20.json') -Raw | ConvertFrom-Json
    $left = @($lifeRaw.cycles | ForEach-Object { $_.left.process })
    $lifecycle = [ordered]@{ baseline = $lifeRaw.baseline; cycles = $lifeRaw.cycles; mountDelta = $lifeRaw.cycles[-1].left.viewer.mounts - $lifeRaw.baseline.mounts; unmountDelta = $lifeRaw.cycles[-1].left.viewer.unmounts - $lifeRaw.baseline.unmounts; workingSet = Get-Trend ([double[]]@($left | ForEach-Object workingSetBytes)); privateMemory = Get-Trend ([double[]]@($left | ForEach-Object privateMemoryBytes)); handles = Get-Trend ([double[]]@($left | ForEach-Object handleCount)) }
}
$windowEvidence = if ($WindowEvidenceDir) { @(Get-ChildItem -LiteralPath $WindowEvidenceDir -Filter '*.json' | ForEach-Object { Get-Content -LiteralPath $_.FullName -Raw | ConvertFrom-Json }) } else { @() }
$dbStability = if ($DbStabilityJson) { Get-Content -LiteralPath $DbStabilityJson -Raw | ConvertFrom-Json } else { $null }

$dbFile = Get-Item -LiteralPath $DbPath
$walPath = "$DbPath-wal"
$result = [ordered]@{
    generatedAt = (Get-Date).ToString('o')
    source = [ordered]@{
        head = (git rev-parse HEAD).Trim()
        diffFingerprint = (git diff --no-ext-diff | git hash-object --stdin).Trim()
        executable = $exe
    }
    database = [ordered]@{
        path = $dbFile.FullName
        bytes = $dbFile.Length
        sha256 = (Get-FileHash -LiteralPath $DbPath -Algorithm SHA256).Hash
        walBytes = if (Test-Path -LiteralPath $walPath) { (Get-Item -LiteralPath $walPath).Length } else { 0 }
        integrityCheck = $integrity
        userVersion = $userVersion
        mapMetadataCount = $mapCount
        jobs = $jobs
        stability = $dbStability
    }
    queries = [ordered]@{
        library500ProcessInclusive = $libraryQuery
        heatmapRealDbProcessInclusive = $heatmapQuery
        roundPositionsDecodeP95Ms = if ($positions) { $positions.p95Ms } else { $null }
        roundPositions = $positions
    }
    runtime = [ordered]@{
        processSnapshot = $process
        coldStartInteractiveMs = if ($cold) { $cold.p95Ms } else { $null }
        coldStarts = $cold
        viewerFps = if ($cpu) { $cpu.active.viewer } else { $null }
        viewerActiveCpu = if ($cpu) { $cpu.active } else { $null }
        viewerPausedCpu = if ($cpu) { $cpu.paused } else { $null }
        tabAwayCpu = if ($cpu) { $cpu.'tab-away' } else { $null }
        viewerLoop20WorkingSetTrend = if ($lifecycle) { $lifecycle.workingSet } else { $null }
        viewerLoop20PrivateMemoryTrend = if ($lifecycle) { $lifecycle.privateMemory } else { $null }
        viewerLoop20HandleTrend = if ($lifecycle) { $lifecycle.handles } else { $null }
        lifecycle20 = $lifecycle
    }
    windows = $windowEvidence
    screenshots = $screenshots
    notes = @(
        'Query timings include sqlite3 process startup and are therefore conservative.',
        'Runtime fields are populated only from structured real-Tauri evidence; missing fields remain null.'
    )
}

$jsonPath = Join-Path $EvidenceDir 'performance.json'
$csvPath = Join-Path $EvidenceDir 'performance.csv'
$result | ConvertTo-Json -Depth 20 | Set-Content -LiteralPath $jsonPath -Encoding utf8
[pscustomobject]@{
    generatedAt = $result.generatedAt
    library500P95Ms = $libraryQuery.p95Ms
    heatmapQueryP95Ms = $heatmapQuery.p95Ms
    roundPositionsDecodeP95Ms = if ($positions) { $positions.p95Ms } else { $null }
    coldStartInteractiveMs = if ($cold) { $cold.p95Ms } else { $null }
    viewerFps = if ($cpu) { $cpu.active.viewer.fps } else { $null }
} | Export-Csv -LiteralPath $csvPath -NoTypeInformation -Encoding utf8

$result | ConvertTo-Json -Depth 20
