param(
  [Parameter(Mandatory)][string]$Installer,
  [Parameter(Mandatory)][string]$Refresh
)
$ErrorActionPreference = 'Stop'
$system = Get-CimInstance Win32_ComputerSystem
if ($system.Manufacturer -ne 'QEMU' -or $system.Model -notmatch '^(Standard PC|KVM)') {
  throw 'Install the Looking Glass display only inside the QEMU Windows VM'
}
$path = $Installer
$hash = (Get-FileHash $path -Algorithm SHA256).Hash.ToLowerInvariant()
if ($hash -ne '1dd363a58ace6fd5e824d1a6c36744a58bbf181b739b3d75df669002cb1c68b5') {
  throw 'Looking Glass installer hash does not match the reviewed B7-829 IDD download'
}
$signature = Get-AuthenticodeSignature $path
if ($signature.Status -ne 'Valid') { throw 'Looking Glass installer signature is invalid' }
$signer = $signature.SignerCertificate
if ($signer.Thumbprint -ne '57476AACDE410C05A63E9FEC8B97938605B9D8F5') { throw 'Looking Glass installer signer is not the reviewed HostFission certificate' }
$device = & pnputil.exe /enum-devices /deviceid 'PCI\VEN_1AF4&DEV_1110' /drivers | Out-String
if ($LASTEXITCODE -ne 0 -or $device -notmatch 'ivshmem.inf') { throw 'IVSHMEM device or driver not found' }

# The IDD replaces the host app; both running at once fight over the shared memory.
$oldHost = 'C:\Program Files\Looking Glass (host)\uninstaller.exe'
if (Test-Path $oldHost) {
  Stop-Service 'Looking Glass (host)' -ErrorAction SilentlyContinue
  $process = Start-Process $oldHost -ArgumentList '/S' -PassThru -Wait
  if ($process.ExitCode -ne 0) { throw "Old Looking Glass host uninstaller exited $($process.ExitCode)" }
}

# A silent install cannot answer the driver publisher prompt, so trust this signer up front.
$store = New-Object System.Security.Cryptography.X509Certificates.X509Store 'TrustedPublisher', 'LocalMachine'
$store.Open('ReadWrite')
$store.Add($signer)
$store.Close()

$process = Start-Process $path -ArgumentList '/S' -PassThru -Wait
if ($process.ExitCode -ne 0) { throw "Looking Glass IDD installer exited $($process.ExitCode)" }

# Match the host panel so window-sized modes are not stuck at 60 Hz.
$settings = 'HKLM:\SOFTWARE\LookingGlass\IDD'
if (-not (Test-Path $settings)) { New-Item $settings -Force | Out-Null }
Set-ItemProperty $settings -Name DefaultRefresh -Value $Refresh -Type String

$display = Get-PnpDevice -Class Display | Where-Object FriendlyName -like '*Looking Glass*' | Select-Object -First 1
$helper = Get-Service -Name 'LGIddHelper' -ErrorAction SilentlyContinue
@{
  environment = 'window-side-libvirt'
  display = if ($display) { @{name=$display.FriendlyName; status=$display.Status} } else { $null }
  helper = if ($helper) { @{state=$helper.Status.ToString(); startMode=$helper.StartType.ToString()} } else { $null }
  oldHostRemoved = -not (Test-Path $oldHost)
} | ConvertTo-Json -Depth 4 -Compress
