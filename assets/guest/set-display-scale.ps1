param(
  [ValidateSet(100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500)][int]$Percent = 100,
  [string]$ResultPath
)
$ErrorActionPreference = 'Stop'
if ((Get-CimInstance Win32_ComputerSystem).Manufacturer -ne 'QEMU') {
  throw 'This command only changes the virtual Windows display inside QEMU.'
}

# The guest agent runs in session 0; display scaling belongs to the signed-in user.
if ([Diagnostics.Process]::GetCurrentProcess().SessionId -eq 0) {
  if (-not $PSCommandPath) { throw 'Copy this script into Windows and execute its file path.' }
  $users = @(Get-CimInstance Win32_Process -Filter "Name='explorer.exe'" | ForEach-Object {
    $owner = Invoke-CimMethod -InputObject $_ -MethodName GetOwner
    if ($owner.ReturnValue -eq 0) { "$($owner.Domain)\$($owner.User)" }
  } | Select-Object -Unique)
  if ($users.Count -ne 1) { throw 'Exactly one signed-in Windows desktop is required.' }
  $taskName = 'window-side-scale-' + [Guid]::NewGuid().ToString('N')
  $resultFile = Join-Path $PSScriptRoot ($taskName + '.json')
  $arguments = '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "{0}" -Percent {1} -ResultPath "{2}"' -f $PSCommandPath, $Percent, $resultFile
  $action = New-ScheduledTaskAction -Execute powershell.exe -Argument $arguments
  $principal = New-ScheduledTaskPrincipal -UserId $users[0] -LogonType Interactive
  Register-ScheduledTask -TaskName $taskName -Action $action -Principal $principal | Out-Null
  try {
    Start-ScheduledTask -TaskName $taskName
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not (Test-Path $resultFile)) {
      if ([DateTime]::UtcNow -gt $deadline) { throw "Timed out waiting for user display task $taskName." }
      Start-Sleep -Milliseconds 300
    }
    $result = Get-Content -Raw $resultFile | ConvertFrom-Json
    if (-not $result.success) { throw $result.error }
    $result | ConvertTo-Json -Compress -Depth 4
  } finally {
    Stop-ScheduledTask -TaskName $taskName -ErrorAction SilentlyContinue
    Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
    Remove-Item $resultFile -ErrorAction SilentlyContinue
  }
  return
}

try {
  # The per-display scale calls behind Settings > Display > Scale. Windows reports each
  # display's steps relative to its recommended scale, so the target becomes an offset.
  Add-Type @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class WindowSideScale {
  [StructLayout(LayoutKind.Sequential)] public struct Luid { public uint Low; public int High; }
  [StructLayout(LayoutKind.Sequential)] public struct Header { public int Type; public int Size; public Luid Adapter; public uint Id; }
  [StructLayout(LayoutKind.Sequential)] public struct ScaleGet { public Header Header; public int Min; public int Current; public int Max; }
  [StructLayout(LayoutKind.Sequential)] public struct ScaleSet { public Header Header; public int Scale; }
  [StructLayout(LayoutKind.Sequential)] public struct Source { public Luid Adapter; public uint Id; public uint ModeIndex; public uint Flags; }
  [StructLayout(LayoutKind.Sequential)] public struct Target {
    public Luid Adapter; public uint Id; public uint ModeIndex; public uint Technology; public uint Rotation; public uint Scaling;
    public uint RefreshNumerator; public uint RefreshDenominator; public uint ScanLine; public int Available; public uint Flags;
  }
  [StructLayout(LayoutKind.Sequential)] public struct Path { public Source Source; public Target Target; public uint Flags; }
  [StructLayout(LayoutKind.Sequential, Size=64)] public struct Mode { public uint InfoType; public uint Id; public Luid Adapter; }
  [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
  [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint pathCount, [Out] Path[] paths, ref uint modeCount, [Out] Mode[] modes, IntPtr topology);
  [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref ScaleGet packet);
  [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref ScaleSet packet);
  static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };
  const uint ActivePaths = 2;

  static ScaleGet Read(Source source) {
    var packet = new ScaleGet();
    packet.Header.Type = -3; packet.Header.Size = Marshal.SizeOf(typeof(ScaleGet));
    packet.Header.Adapter = source.Adapter; packet.Header.Id = source.Id;
    int status = DisplayConfigGetDeviceInfo(ref packet);
    if (status != 0) throw new InvalidOperationException("Windows could not read the display scale: " + status);
    return packet;
  }

  public static object Set(int percent) {
    uint pathCount, modeCount;
    int status = GetDisplayConfigBufferSizes(ActivePaths, out pathCount, out modeCount);
    if (status != 0) throw new InvalidOperationException("Windows could not list displays: " + status);
    var paths = new Path[pathCount]; var modes = new Mode[modeCount];
    status = QueryDisplayConfig(ActivePaths, ref pathCount, paths, ref modeCount, modes, IntPtr.Zero);
    if (status != 0) throw new InvalidOperationException("Windows could not read displays: " + status);
    var seen = new HashSet<string>();
    var displays = new List<object>();
    for (int i = 0; i < pathCount; i++) {
      var source = paths[i].Source;
      if (!seen.Add(source.Adapter.Low + ":" + source.Adapter.High + ":" + source.Id)) continue;
      var before = Read(source);
      int recommended = Math.Abs(before.Min);
      int highest = Math.Min(Steps.Length - 1, recommended + before.Max);
      int wanted = Math.Min(Array.IndexOf(Steps, percent), highest);
      int previous = Math.Max(0, Math.Min(Steps.Length - 1, recommended + before.Current));
      if (wanted != previous) {
        var change = new ScaleSet();
        change.Header.Type = -4; change.Header.Size = Marshal.SizeOf(typeof(ScaleSet));
        change.Header.Adapter = source.Adapter; change.Header.Id = source.Id;
        change.Scale = wanted - recommended;
        status = DisplayConfigSetDeviceInfo(ref change);
        if (status != 0) throw new InvalidOperationException("Windows rejected the display scale: " + status);
      }
      var after = Read(source);
      int current = Math.Max(0, Math.Min(Steps.Length - 1, recommended + after.Current));
      displays.Add(new { previous = Steps[previous], current = Steps[current], maximum = Steps[highest] });
    }
    if (displays.Count == 0) throw new InvalidOperationException("Windows has no active display.");
    return new { success = true, environment = "window-side", requested = percent, displays = displays };
  }
}
'@
  $result = [WindowSideScale]::Set($Percent) | ConvertTo-Json -Compress -Depth 4
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
