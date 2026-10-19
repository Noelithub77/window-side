param([ValidateSet('check', 'install')][string]$Mode = 'check')
$ErrorActionPreference = 'Stop'
$session = New-Object -ComObject Microsoft.Update.Session
$session.ClientApplicationID = 'window-side'
$found = $session.CreateUpdateSearcher().Search("IsInstalled=0 and IsHidden=0").Updates
$titles = @($found | ForEach-Object { $_.Title })
$result = @{ installed = @(); failed = @() }
if ($Mode -eq 'install' -and $found.Count -gt 0) {
  $todo = New-Object -ComObject Microsoft.Update.UpdateColl
  foreach ($update in $found) { if (-not $update.EulaAccepted) { $update.AcceptEula() }; [void]$todo.Add($update) }
  $downloader = $session.CreateUpdateDownloader(); $downloader.Updates = $todo; [void]$downloader.Download()
  $installer = $session.CreateUpdateInstaller(); $installer.Updates = $todo
  $outcome = $installer.Install()
  for ($i = 0; $i -lt $todo.Count; $i++) {
    # 2 = succeeded, 3 = succeeded with errors.
    if ($outcome.GetUpdateResult($i).ResultCode -in 2, 3) { $result.installed += $todo.Item($i).Title }
    else { $result.failed += $todo.Item($i).Title }
  }
  $titles = @($titles | Where-Object { $_ -notin $result.installed })
}
@{
  bootedAt = (Get-CimInstance Win32_OperatingSystem).LastBootUpTime.ToUniversalTime().ToString('o')
  pending = $titles
  installed = $result.installed
  failed = $result.failed
  rebootRequired = (New-Object -ComObject Microsoft.Update.SystemInfo).RebootRequired
} | ConvertTo-Json -Compress
