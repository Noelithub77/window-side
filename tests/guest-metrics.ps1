param([Parameter(Mandatory)][string]$MetricsSource, [Parameter(Mandatory)][string]$DisplaySource)
$ErrorActionPreference = 'Stop'
$source = $MetricsSource
$tokens = $null; $errors = $null
$ast = [Management.Automation.Language.Parser]::ParseInput($source, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
$helper = $ast.Find({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Read-OptionalMetric' }, $true)
. ([scriptblock]::Create($helper.Extent.Text))
$value = Read-OptionalMetric { @{value=42} } -TimeoutMilliseconds 3000
if ($value.value -ne 42) { throw 'Successful optional query lost its result' }
$directory = Join-Path $env:TEMP ([guid]::NewGuid().ToString())
New-Item -ItemType Directory $directory | Out-Null
try {
  $pidFile = Join-Path $directory 'worker-pid'
  $query = [scriptblock]::Create("[IO.File]::WriteAllText('$pidFile', [string]`$PID); Start-Sleep -Seconds 30")
  $watch = [Diagnostics.Stopwatch]::StartNew()
  $value = Read-OptionalMetric $query -TimeoutMilliseconds 2000
  if ($null -ne $value -or $watch.Elapsed.TotalSeconds -gt 6) { throw 'Optional query did not time out' }
  if (-not (Test-Path $pidFile)) { throw 'Timeout fixture never started' }
  $worker = [int](Get-Content $pidFile)
  if (Get-Process -Id $worker -ErrorAction SilentlyContinue) { throw 'Timed-out worker survived' }
} finally { Remove-Item $directory -Recurse -Force }
# Exercise the real collector with an unavailable optional provider.
$collector = $source.Remove($helper.Extent.StartOffset, $helper.Extent.EndOffset - $helper.Extent.StartOffset)
$sample = & {
  function Read-OptionalMetric { param($Query, $TimeoutMilliseconds) $null }
  function Get-CimInstance {
    param($ClassName, $Filter, $OperationTimeoutSec)
    if ($ClassName -eq 'Win32_OperatingSystem') {
      [pscustomobject]@{TotalVisibleMemorySize=8192;FreePhysicalMemory=4096;LastBootUpTime=(Get-Date).AddMinutes(-10);NumberOfProcesses=100}
    } else { [pscustomobject]@{PercentProcessorTime=25} }
  }
  & ([scriptblock]::Create($collector)) | ConvertFrom-Json
}
if ($sample.cpuPercent -ne 25 -or $sample.memoryUsedBytes -ne 4194304 -or $null -ne $sample.networkReceivedBytes) {
  throw 'Unavailable optional queries discarded or fabricated metrics'
}
# The same disabled IDD must be found after leaving and returning from native Windows.
$displaySource = $DisplaySource
$global:wsDisplayEnabled = $false
& {
  function Get-CimInstance { [pscustomobject]@{Manufacturer='QEMU';Model='Standard PC'} }
  function Get-PnpDevice {
    param($Class, [switch]$PresentOnly)
    [pscustomobject]@{InstanceId='PCI\VEN_10DE&DEV_1234';Status='OK';FriendlyName='NVIDIA'}
    if (-not $PresentOnly) { [pscustomobject]@{InstanceId='ROOT\DISPLAY\0000';Status='Unknown';FriendlyName='Looking Glass IDD'} }
  }
  function Get-Service { $null }
  function Enable-PnpDevice { process { $global:wsDisplayEnabled = $true } }
  function Disable-PnpDevice { }
  & ([scriptblock]::Create($displaySource))
}
if (-not $global:wsDisplayEnabled) { throw 'Disconnected Looking Glass display was skipped' }
Remove-Variable wsDisplayEnabled -Scope Global
@{optionalSuccess=$true;timeoutCleanup=$true;partialMetrics=$true;disconnectedDisplayRecovery=$true}|ConvertTo-Json -Compress
