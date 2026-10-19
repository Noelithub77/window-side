$ErrorActionPreference = 'Stop'
$o = Get-CimInstance Win32_OperatingSystem
$p = Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'"
$n = @(Get-NetAdapterStatistics -ErrorAction Stop)
if ($null -eq $p) { throw 'Windows CPU counter unavailable' }
@{
  cpuPercent = [double]$p.PercentProcessorTime
  memoryTotalBytes = [long]$o.TotalVisibleMemorySize * 1024
  memoryUsedBytes = ([long]$o.TotalVisibleMemorySize - [long]$o.FreePhysicalMemory) * 1024
  uptimeSeconds = [long]((Get-Date) - $o.LastBootUpTime).TotalSeconds
  bootedAt = $o.LastBootUpTime.ToUniversalTime().ToString('o')
  networkReceivedBytes = [long](($n | Measure-Object ReceivedBytes -Sum).Sum)
  networkSentBytes = [long](($n | Measure-Object SentBytes -Sum).Sum)
  processCount = [int]$o.NumberOfProcesses
  rebootRequired = (New-Object -ComObject Microsoft.Update.SystemInfo).RebootRequired
} | ConvertTo-Json -Compress
