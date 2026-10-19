$ErrorActionPreference = 'Stop'
$system = Get-CimInstance Win32_ComputerSystem
$qemu = $system.Manufacturer -eq 'QEMU' -and $system.Model -match '^(Standard PC|KVM)'
$deadline = (Get-Date).AddSeconds(30)
do {
  $displays = @(Get-PnpDevice -Class Display -PresentOnly)
  $hardware = @($displays | Where-Object { $_.InstanceId -match '^PCI\\VEN_(10DE|1002|8086)&' })
  $ready = @($hardware | Where-Object Status -eq 'OK')
  if (-not $qemu -or $hardware.Count -eq 0 -or $ready.Count -gt 0) { break }
  Start-Sleep -Seconds 2
} while ((Get-Date) -lt $deadline)
$lookingGlass = @($displays | Where-Object FriendlyName -like 'Looking Glass*')
$virtio = @($displays | Where-Object FriendlyName -like 'Red Hat VirtIO GPU*')
if ($qemu -and $ready.Count -gt 0 -and $lookingGlass.Count -gt 0) {
  if (Get-Service LGIddHelper -ErrorAction SilentlyContinue) { Start-Service LGIddHelper }
  $lookingGlass | Where-Object Status -ne 'OK' | Enable-PnpDevice -Confirm:$false
  $virtio | Where-Object Status -eq 'OK' | Disable-PnpDevice -Confirm:$false
} else {
  if ($qemu) { $virtio | Where-Object Status -ne 'OK' | Enable-PnpDevice -Confirm:$false }
  $lookingGlass | Where-Object Status -eq 'OK' | Disable-PnpDevice -Confirm:$false
  if (Get-Service LGIddHelper -ErrorAction SilentlyContinue) { Stop-Service LGIddHelper -ErrorAction SilentlyContinue }
}
