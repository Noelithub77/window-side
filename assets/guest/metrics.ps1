$ErrorActionPreference = 'Stop'
function Read-OptionalMetric {
  param([scriptblock]$Query, [int]$TimeoutMilliseconds = 2000)
  $start = New-Object Diagnostics.ProcessStartInfo
  $start.FileName = "$PSHOME\powershell.exe"
  $source = '$ErrorActionPreference="Stop"; & {' + $Query.ToString() + '} | ConvertTo-Json -Compress'
  $encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($source))
  $start.Arguments = "-NoProfile -NonInteractive -EncodedCommand $encoded"
  $start.UseShellExecute = $false
  $start.CreateNoWindow = $true
  $start.RedirectStandardOutput = $true
  $start.RedirectStandardError = $true
  $process = New-Object Diagnostics.Process
  $process.StartInfo = $start
  try {
    [void]$process.Start()
    $output = $process.StandardOutput.ReadToEndAsync()
    [void]$process.StandardError.ReadToEndAsync()
    if (-not $process.WaitForExit($TimeoutMilliseconds)) {
      $process.Kill()
      $process.WaitForExit()
      return $null
    }
    if ($process.ExitCode -ne 0) { return $null }
    $output.Result | ConvertFrom-Json
  } catch { return $null }
  finally { $process.Dispose() }
}
$o = Get-CimInstance Win32_OperatingSystem -OperationTimeoutSec 2
$p = Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'" -OperationTimeoutSec 2
$n = Read-OptionalMetric {
  $stats = @(Get-NetAdapterStatistics -ErrorAction Stop)
  if ($stats.Count -eq 0) { throw 'Windows network counters unavailable' }
  @{
    received = [long](($stats | Measure-Object ReceivedBytes -Sum).Sum)
    sent = [long](($stats | Measure-Object SentBytes -Sum).Sum)
  }
}
if ($null -eq $p) { throw 'Windows CPU counter unavailable' }
# Like Task Manager: the busiest engine type, summed over processes. Null without a GPU driver.
$gpu = Read-OptionalMetric {
  $engines = @(Get-CimInstance Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine -ErrorAction Stop)
  if ($engines.Count -gt 0) {
    $busiest = ($engines | Group-Object { ($_.Name -split 'engtype_')[-1] } |
      ForEach-Object { ($_.Group | Measure-Object UtilizationPercentage -Sum).Sum } | Measure-Object -Maximum).Maximum
    [Math]::Min(100, [double]$busiest)
  }
}
$rebootRequired = Read-OptionalMetric { (New-Object -ComObject Microsoft.Update.SystemInfo).RebootRequired } -TimeoutMilliseconds 1000
@{
  cpuPercent = [double]$p.PercentProcessorTime
  gpuPercent = $gpu
  memoryTotalBytes = [long]$o.TotalVisibleMemorySize * 1024
  memoryUsedBytes = ([long]$o.TotalVisibleMemorySize - [long]$o.FreePhysicalMemory) * 1024
  uptimeSeconds = [long]((Get-Date) - $o.LastBootUpTime).TotalSeconds
  bootedAt = $o.LastBootUpTime.ToUniversalTime().ToString('o')
  networkReceivedBytes = if ($null -ne $n) { $n.received } else { $null }
  networkSentBytes = if ($null -ne $n) { $n.sent } else { $null }
  processCount = [int]$o.NumberOfProcesses
  rebootRequired = $rebootRequired
} | ConvertTo-Json -Compress
