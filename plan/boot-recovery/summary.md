# Boot connection recovery

- [chosen] Keep the existing guest-agent metrics and pinned Looking Glass IDD.
- Reproduce: full metrics and `Get-NetAdapterStatistics` exceed the eight-second budget; basic CPU/memory counters succeed. Looking Glass reports no producer; Windows lists its installed IDD as disconnected with `ConfigFlags=1`, RTX and IVSHMEM started.
- Bound optional metrics in owned child processes, return null for unavailable network counters, and preserve CPU/memory readings. Verify timeout cleanup and partial readings.
- Restore the installed virtual display and correct startup recovery without changing native-boot guards or adding a network listener.
- Run project checks, install the repaired binary, verify live counters and transport separately. A clean Windows restart needs the user's answer because it closes apps.

## Implementation and evidence

- Optional network, GPU and update queries run in dedicated bounded PowerShell processes; timeout kills only that worker. Network counters may be null without discarding CPU/memory. Core CIM calls have explicit timeouts; CLI statistics failures are reported on stderr.
- Display startup includes non-present devices, so the disabled root IDD is eligible for recovery. The running guest's legacy task was redirected to this tracked script; its previous XML was backed up at `C:\ProgramData\window-side\guest-services-before-boot-repair.xml`. Installed script SHA256: `14517a1d9aaa0cc01f6cdbb7829acf9dcfa79fbe5f35c46eb84b873b33f91853`.
- `./ops check` passed: 52 Rust tests, strict Clippy, UI checks and CLI/installer fixtures. Windows regression fixtures passed via SSH: optional success, timeout worker termination, partial metrics and recovery of a disconnected virtual display.
- Updated binary installed under `~/.local/bin/ws`, with installer backups. No packages were missing or installed.
- Live recovery remains pending: clean Windows shutdown timed out, guest-agent process creation fails, and even the CPU provider now times out. The user approved a clean restart; forced power-off was separately requested and remains unapproved. No forced stop or host reboot was performed.
- No live desktop was captured or driven. No working Looking Glass pixels, fresh complete metrics or native Windows boot are claimed.
