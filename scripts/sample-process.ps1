param(
    [Parameter(Mandatory = $true)][int]$ProcessId,
    [Parameter(Mandatory = $true)][string]$OutputJson,
    [int]$DurationMs = 10000,
    [int]$IntervalMs = 500,
    [string]$State = ''
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$logicalProcessors = [Environment]::ProcessorCount
$processRows = @(Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId)
$tracked = [Collections.Generic.HashSet[int]]::new(); [void]$tracked.Add($ProcessId)
do {
    $added = $false
    foreach ($row in $processRows) {
        if ($tracked.Contains([int]$row.ParentProcessId) -and $tracked.Add([int]$row.ProcessId)) { $added = $true }
    }
} while ($added)
$trackedIds = @($tracked)
$samples = [Collections.Generic.List[object]]::new()
$started = [Diagnostics.Stopwatch]::StartNew()
$previousCpu = $null; $previousTreeCpu = $null; $previousElapsed = $null
while ($started.ElapsedMilliseconds -le $DurationMs) {
    $process = Get-Process -Id $ProcessId -ErrorAction Stop
    $treeProcesses = @(Get-Process -Id $trackedIds -ErrorAction SilentlyContinue)
    $elapsed = $started.Elapsed.TotalSeconds
    $cpu = [double]$process.CPU
    $treeCpu = [double](($treeProcesses | Measure-Object CPU -Sum).Sum)
    $cpuPercent = if ($null -ne $previousCpu -and $elapsed -gt $previousElapsed) { 100.0 * ($cpu - $previousCpu) / ($elapsed - $previousElapsed) / $logicalProcessors } else { $null }
    $treeCpuPercent = if ($null -ne $previousTreeCpu -and $elapsed -gt $previousElapsed) { 100.0 * ($treeCpu - $previousTreeCpu) / ($elapsed - $previousElapsed) / $logicalProcessors } else { $null }
    $samples.Add([ordered]@{ timestamp = (Get-Date).ToString('o'); elapsedMs = [Math]::Round($started.Elapsed.TotalMilliseconds, 3); cpuSeconds = [Math]::Round($cpu, 6); normalizedCpuPercent = if ($null -ne $cpuPercent) { [Math]::Round($cpuPercent, 4) } else { $null }; treeCpuSeconds = [Math]::Round($treeCpu, 6); treeNormalizedCpuPercent = if ($null -ne $treeCpuPercent) { [Math]::Round($treeCpuPercent, 4) } else { $null }; workingSetBytes = [int64]$process.WorkingSet64; privateMemoryBytes = [int64]$process.PrivateMemorySize64; handleCount = [int]$process.HandleCount })
    $previousCpu = $cpu; $previousTreeCpu = $treeCpu; $previousElapsed = $elapsed
    Start-Sleep -Milliseconds $IntervalMs
}
$values = @($samples | Where-Object { $null -ne $_.treeNormalizedCpuPercent } | ForEach-Object { [double]$_.treeNormalizedCpuPercent } | Sort-Object)
$hostValues = @($samples | Where-Object { $null -ne $_.normalizedCpuPercent } | ForEach-Object { [double]$_.normalizedCpuPercent } | Sort-Object)
function Percentile([double[]]$Items, [double]$P) { if (-not $Items.Count) { return $null }; $index = [Math]::Max(0, [Math]::Ceiling($Items.Count * $P) - 1); [Math]::Round($Items[$index], 4) }
$result = [ordered]@{ state = $State; pid = $ProcessId; treePids = $trackedIds; logicalProcessors = $logicalProcessors; durationMs = $DurationMs; intervalMs = $IntervalMs; samples = $samples; summary = [ordered]@{ meanCpuPercent = if ($values.Count) { [Math]::Round(($values | Measure-Object -Average).Average, 4) } else { $null }; medianCpuPercent = if ($values.Count) { Percentile $values 0.5 } else { $null }; p95CpuPercent = if ($values.Count) { Percentile $values 0.95 } else { $null }; peakCpuPercent = if ($values.Count) { [Math]::Round(($values | Measure-Object -Maximum).Maximum, 4) } else { $null }; hostMeanCpuPercent = if ($hostValues.Count) { [Math]::Round(($hostValues | Measure-Object -Average).Average, 4) } else { $null } } }
$parent = Split-Path -Parent $OutputJson; if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
$result | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputJson -Encoding utf8
$result | ConvertTo-Json -Depth 3
