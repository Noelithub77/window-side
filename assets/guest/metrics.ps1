$ErrorActionPreference = 'Stop'
$o = Get-CimInstance Win32_OperatingSystem
$p = Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'"
$n = @(Get-NetAdapterStatistics -ErrorAction Stop)
if ($null -eq $p) { throw 'Windows CPU counter unavailable' }
# Like Task Manager: the busiest engine type, summed over processes. Null without a GPU driver.
$gpu = $null
try {
  $engines = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine -ErrorAction Stop)
  if ($engines.Count -gt 0) {
    $busiest = ($engines | Group-Object { ($_.Name -split 'engtype_')[-1] } |
      ForEach-Object { ($_.Group | Measure-Object UtilizationPercentage -Sum).Sum } | Measure-Object -Maximum).Maximum
    $gpu = [Math]::Min(100, [double]$busiest)
  }
} catch { $gpu = $null }
@{
  cpuPercent = [double]$p.PercentProcessorTime
  gpuPercent = $gpu
  memoryTotalBytes = [long]$o.TotalVisibleMemorySize * 1024
  memoryUsedBytes = ([long]$o.TotalVisibleMemorySize - [long]$o.FreePhysicalMemory) * 1024
  uptimeSeconds = [long]((Get-Date) - $o.LastBootUpTime).TotalSeconds
  bootedAt = $o.LastBootUpTime.ToUniversalTime().ToString('o')
  networkReceivedBytes = [long](($n | Measure-Object ReceivedBytes -Sum).Sum)
  networkSentBytes = [long](($n | Measure-Object SentBytes -Sum).Sum)
  processCount = [int]$o.NumberOfProcesses
  rebootRequired = (New-Object -ComObject Microsoft.Update.SystemInfo).RebootRequired
} | ConvertTo-Json -Compress
