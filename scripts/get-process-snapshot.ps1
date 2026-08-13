param([Parameter(Mandatory = $true)][int]$ProcessId)
$ErrorActionPreference = 'Stop'
$process = Get-Process -Id $ProcessId -ErrorAction Stop
[ordered]@{ timestamp = (Get-Date).ToString('o'); pid = $ProcessId; cpuSeconds = [Math]::Round([double]$process.CPU, 6); workingSetBytes = [int64]$process.WorkingSet64; privateMemoryBytes = [int64]$process.PrivateMemorySize64; handleCount = [int]$process.HandleCount } | ConvertTo-Json -Compress
