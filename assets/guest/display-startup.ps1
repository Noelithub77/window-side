param([switch]$UseVirtio)
$ErrorActionPreference = 'Stop'
$system = Get-CimInstance Win32_ComputerSystem
$qemu = $system.Manufacturer -eq 'QEMU' -and $system.Model -match '^(Standard PC|KVM)'
$deadline = (Get-Date).AddSeconds(30)
do {
  # A disabled root display can be non-present after a native boot.
  $displays = @(Get-PnpDevice -Class Display)
  # A GPU from an earlier passthrough boot stays listed, but is not present.
  $hardware = @($displays | Where-Object { $_.Present -and $_.InstanceId -match '^PCI\\VEN_(10DE|1002|8086)&' })
  $ready = @($hardware | Where-Object Status -eq 'OK')
  if (-not $qemu -or $hardware.Count -eq 0 -or $ready.Count -gt 0) { break }
  Start-Sleep -Seconds 2
} while ((Get-Date) -lt $deadline)
$lookingGlass = @($displays | Where-Object FriendlyName -like 'Looking Glass*')
$virtio = @($displays | Where-Object FriendlyName -like 'Red Hat VirtIO GPU*')
$sharedMemory = @(Get-PnpDevice -PresentOnly | Where-Object InstanceId -like 'PCI\VEN_1AF4&DEV_1110*')
$gpuFailed = $hardware.Count -gt 0 -and $ready.Count -eq 0
$useLookingGlass = $qemu -and -not $UseVirtio -and -not $gpuFailed -and $sharedMemory.Count -gt 0 -and $lookingGlass.Count -gt 0
if ($useLookingGlass) {
  $settings = 'HKLM:\SOFTWARE\LookingGlass\IDD'
  if (-not (Test-Path $settings)) { New-Item $settings -Force | Out-Null }
  $values = Get-ItemProperty $settings
  $gpuRefresh = if ($values.GpuRefresh) { $values.GpuRefresh } else { $values.DefaultRefresh }
  if ($gpuRefresh -and -not $values.GpuRefresh) { Set-ItemProperty $settings -Name GpuRefresh -Value $gpuRefresh -Type String }
  # Windows renders on the CPU without a GPU; the host panel's full rate would starve it.
  $refresh = if ($hardware.Count -eq 0) { '60' } else { $gpuRefresh }
  $restart = @($virtio | Where-Object Status -eq 'OK').Count -gt 0
  if ($refresh -and $refresh -ne $values.DefaultRefresh) {
    Set-ItemProperty $settings -Name DefaultRefresh -Value $refresh -Type String
    $restart = $true
  }
  if (Get-Service LGIddHelper -ErrorAction SilentlyContinue) { Start-Service LGIddHelper }
  $virtio | Where-Object Status -eq 'OK' | Disable-PnpDevice -Confirm:$false
  # The driver picks its render adapter and refresh rate only when it starts.
  if ($restart) {
    $lookingGlass | Where-Object Status -eq 'OK' | Disable-PnpDevice -Confirm:$false
    $lookingGlass | Enable-PnpDevice -Confirm:$false
  } else {
    $lookingGlass | Where-Object Status -ne 'OK' | Enable-PnpDevice -Confirm:$false
  }
} else {
  if ($qemu) { $virtio | Where-Object Status -ne 'OK' | Enable-PnpDevice -Confirm:$false }
  $lookingGlass | Where-Object Status -eq 'OK' | Disable-PnpDevice -Confirm:$false
  if (Get-Service LGIddHelper -ErrorAction SilentlyContinue) { Stop-Service LGIddHelper -ErrorAction SilentlyContinue }
}
@{lookingGlass=[bool]$useLookingGlass} | ConvertTo-Json -Compress
