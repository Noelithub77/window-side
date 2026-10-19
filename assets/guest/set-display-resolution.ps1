param(
  [ValidateRange(640, 7680)][int]$Width = 2560,
  [ValidateRange(480, 4320)][int]$Height = 1440,
  [string]$ResultPath
)
$ErrorActionPreference = 'Stop'
if ((Get-CimInstance Win32_ComputerSystem).Manufacturer -ne 'QEMU') {
  throw 'This command only changes the virtual Windows display inside QEMU.'
}

# The guest agent runs in session 0; display changes belong to the signed-in user.
if ([Diagnostics.Process]::GetCurrentProcess().SessionId -eq 0) {
  if (-not $PSCommandPath) { throw 'Copy this script into Windows and execute its file path.' }
  $users = @(Get-CimInstance Win32_Process -Filter "Name='explorer.exe'" | ForEach-Object {
    $owner = Invoke-CimMethod -InputObject $_ -MethodName GetOwner
    if ($owner.ReturnValue -eq 0) { "$($owner.Domain)\$($owner.User)" }
  } | Select-Object -Unique)
  if ($users.Count -ne 1) { throw 'Exactly one signed-in Windows desktop is required.' }
  $devices = @(Get-PnpDevice -PresentOnly -Class Display | Where-Object InstanceId -like 'PCI\VEN_1AF4&DEV_1050*')
  if ($devices.Count -ne 1) { throw 'Exactly one active VirtIO display adapter is required.' }
  $driver = Get-CimInstance Win32_SystemDriver -Filter "Name='VioGpuDod'"
  $binary = [Text.Encoding]::Unicode.GetString([IO.File]::ReadAllBytes($driver.PathName))
  foreach ($name in @('PersistentDispMode0Width', 'PersistentDispMode0Height')) {
    if (-not $binary.Contains($name)) { throw 'The installed VirtIO driver does not support a persistent custom mode.' }
  }
  $driverKey = (Get-PnpDeviceProperty -InstanceId $devices[0].InstanceId -KeyName DEVPKEY_Device_Driver).Data
  $registryPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\' + $driverKey
  $old = Get-ItemProperty $registryPath
  $modeChanged = $old.PersistentDispMode0Width -ne $Width -or $old.PersistentDispMode0Height -ne $Height
  if ($modeChanged) {
    New-ItemProperty $registryPath -Name PersistentDispMode0Width -Value $Width -PropertyType DWord -Force | Out-Null
    New-ItemProperty $registryPath -Name PersistentDispMode0Height -Value $Height -PropertyType DWord -Force | Out-Null
    & pnputil.exe /restart-device $devices[0].InstanceId | Out-Null
    if ($LASTEXITCODE -ne 0) { throw "Virtual display reload failed with exit code $LASTEXITCODE." }
    Start-Sleep -Seconds 2
  }
  $taskName = 'window-side-resolution-' + [Guid]::NewGuid().ToString('N')
  $resultFile = Join-Path $PSScriptRoot ($taskName + '.json')
  $arguments = '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "{0}" -Width {1} -Height {2} -ResultPath "{3}"' -f $PSCommandPath, $Width, $Height, $resultFile
  $action = New-ScheduledTaskAction -Execute powershell.exe -Argument $arguments
  $principal = New-ScheduledTaskPrincipal -UserId $users[0] -LogonType Interactive
  Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal | Out-Null
  try {
    Start-ScheduledTask -TaskName $taskName
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not (Test-Path $resultFile)) {
      if ([DateTime]::UtcNow -gt $deadline) { throw "Timed out waiting for user display task $taskName." }
      Start-Sleep -Milliseconds 500
    }
    $result = Get-Content -Raw $resultFile | ConvertFrom-Json
    if (-not $result.success) { throw $result.error }
    $result | Add-Member -NotePropertyName persistentModeChanged -NotePropertyValue $modeChanged
    $result | Add-Member -NotePropertyName timestamp -NotePropertyValue ([DateTime]::UtcNow.ToString('o'))
    $result | ConvertTo-Json -Compress
  } finally {
    Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
    Remove-Item $resultFile -ErrorAction SilentlyContinue
  }
  return
}

try {
  Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class WindowSideDisplay {
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)]
  public struct Mode {
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string Device;
    public ushort SpecVersion, DriverVersion, Size, DriverExtra;
    public uint Fields;
    public int X, Y;
    public uint Orientation, FixedOutput;
    public short Color, Duplex, YResolution, TTOption, Collate;
    [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string Form;
    public ushort LogPixels;
    public uint BitsPerPixel, Width, Height, Flags, Frequency;
    public uint ICMMethod, ICMIntent, MediaType, DitherType, Reserved1, Reserved2, PanningWidth, PanningHeight;
  }
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  static extern bool EnumDisplaySettings(string device, int index, ref Mode mode);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)]
  static extern int ChangeDisplaySettingsEx(string device, ref Mode mode, IntPtr window, uint flags, IntPtr parameter);
  static Mode Read(int index) {
    var mode = new Mode { Size=(ushort)Marshal.SizeOf(typeof(Mode)) };
    if (!EnumDisplaySettings(null, index, ref mode)) throw new InvalidOperationException("Cannot read Windows display mode.");
    return mode;
  }
  public static object Set(uint width, uint height) {
    var before = Read(-1);
    var selected = new Mode();
    bool found = false;
    for (int i=0; i<4096; i++) {
      var mode = new Mode { Size=(ushort)Marshal.SizeOf(typeof(Mode)) };
      if (!EnumDisplaySettings(null, i, ref mode)) break;
      if (mode.Width == width && mode.Height == height && mode.BitsPerPixel == before.BitsPerPixel) {
        selected = mode; found = true; break;
      }
    }
    if (!found) throw new InvalidOperationException("The requested resolution is absent after configuring the persistent VirtIO mode.");
    int test = ChangeDisplaySettingsEx(null, ref selected, IntPtr.Zero, 2, IntPtr.Zero);
    if (test != 0) throw new InvalidOperationException("Windows rejected the display mode test: " + test);
    int apply = ChangeDisplaySettingsEx(null, ref selected, IntPtr.Zero, 1, IntPtr.Zero);
    if (apply != 0) throw new InvalidOperationException("Windows could not save and apply the display mode: " + apply);
    var after = Read(-1);
    var saved = Read(-2);
    if (after.Width != width || after.Height != height || saved.Width != width || saved.Height != height)
      throw new InvalidOperationException("The current or saved Windows resolution did not match the request.");
    return new { success=true, environment="window-side", previousWidth=before.Width,
      previousHeight=before.Height, width=after.Width, height=after.Height, saved=true };
  }
}
'@
  $result = [WindowSideDisplay]::Set($Width, $Height) | ConvertTo-Json -Compress
} catch {
  $result = @{ success=$false; error=$_.Exception.Message } | ConvertTo-Json -Compress
}
if ($ResultPath) {
  $temporary = $ResultPath + '.tmp'
  $result | Set-Content -Encoding UTF8 $temporary
  Move-Item $temporary $ResultPath
} else {
  $result
  if (-not ($result | ConvertFrom-Json).success) { exit 1 }
}
