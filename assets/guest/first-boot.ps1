param([switch]$Apply)
$ErrorActionPreference = 'Stop'
$system = Get-CimInstance Win32_ComputerSystem
if ($system.Manufacturer -ne 'QEMU' -or $system.Model -notmatch '^(Standard PC|KVM)') {
  throw 'First-boot provisioning is available only in the QEMU Windows VM'
}
$storage = Get-CimInstance Win32_SystemDriver -Filter "Name='viostor'"
$network = Get-CimInstance Win32_SystemDriver -Filter "Name='netkvm'"
$storageReady = $null -ne $storage -and $storage.State -eq 'Running'
$networkPath = [Environment]::ExpandEnvironmentVariables([string]$network.PathName).Trim('"')
$networkPath = $networkPath -replace '^\\SystemRoot\\', "$env:windir\"
if ($networkPath.StartsWith('\??\')) { $networkPath = $networkPath.Substring(4) }
$networkReady = $null -ne $network -and -not [string]::IsNullOrWhiteSpace($networkPath) -and (Test-Path -LiteralPath $networkPath)
$user = [string]$system.UserName
if ($Apply) {
  if (-not $storageReady -or -not $networkReady) { throw 'Install the VirtIO guest tools before finishing setup' }
  if ([string]::IsNullOrWhiteSpace($user)) { throw 'Sign in to Windows before finishing setup' }
  $backup = 'C:\ProgramData\window-side\power-before.json'
  if (-not (Test-Path $backup)) {
    $power = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Power'
    $session = Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Power'
    @{ hibernateEnabled=$power.HibernateEnabled; fastStartup=$session.HiberbootEnabled } |
      ConvertTo-Json -Compress | Set-Content $backup -Encoding UTF8
  }
  & powercfg.exe /hibernate off | Out-Null
  if ($LASTEXITCODE -ne 0) { throw 'Could not disable Windows hibernation' }
  Set-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\Session Manager\Power' -Name HiberbootEnabled -Value 0 -Type DWord
  Set-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\TimeZoneInformation' -Name RealTimeIsUniversal -Value 1 -Type DWord
  $action = New-ScheduledTaskAction -Execute 'powershell.exe' -Argument '-NoProfile -NonInteractive -ExecutionPolicy Bypass -File "C:\ProgramData\window-side\display-startup.ps1"'
  $trigger = New-ScheduledTaskTrigger -AtStartup
  $principal = New-ScheduledTaskPrincipal -UserId 'SYSTEM' -LogonType ServiceAccount -RunLevel Highest
  $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -ExecutionTimeLimit (New-TimeSpan -Minutes 1) -StartWhenAvailable
  Register-ScheduledTask -TaskName 'Window Side guest services' -Action $action -Trigger $trigger -Principal $principal -Settings $settings -Force | Out-Null
  if (Get-Service LGIddHelper -ErrorAction SilentlyContinue) { Set-Service LGIddHelper -StartupType Manual }
  & 'C:\ProgramData\window-side\display-startup.ps1'
}
@{
  environment='window-side-libvirt'
  user=$user
  storageReady=$storageReady
  networkReady=$networkReady
  applied=[bool]$Apply
} | ConvertTo-Json -Compress
