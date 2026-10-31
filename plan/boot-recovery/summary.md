# Boot connection recovery

- [chosen] Keep the existing guest-agent metrics and pinned Looking Glass IDD.
- Reproduce: full metrics and `Get-NetAdapterStatistics` exceed the eight-second budget; basic CPU/memory counters succeed. Looking Glass reports no producer; Windows lists its installed IDD as disconnected with `ConfigFlags=1`, RTX and IVSHMEM started.
- Bound optional metrics in owned child processes, return null for unavailable network counters, and preserve CPU/memory readings. Verify timeout cleanup and partial readings.
- Restore the installed virtual display and correct startup recovery without changing native-boot guards or adding a network listener.
- Run project checks, install the repaired binary, verify live counters and transport separately. Clean and forced Windows restarts were approved; a host reboot remains a separate decision.

## Implementation and evidence

- Optional network, GPU and update queries run in dedicated bounded PowerShell processes; timeout kills only that worker. Network counters may be null without discarding CPU/memory. Core CIM calls have explicit timeouts; CLI statistics failures are reported on stderr.
- Display startup includes non-present devices, so the disabled root IDD is eligible for recovery. The running guest's legacy task was redirected to this tracked script; its previous XML was backed up at `C:\ProgramData\window-side\guest-services-before-boot-repair.xml`. Installed script SHA256: `14517a1d9aaa0cc01f6cdbb7829acf9dcfa79fbe5f35c46eb84b873b33f91853`.
- `./ops check` passed: 52 Rust tests, strict Clippy, UI checks and CLI/installer fixtures. Windows regression fixtures passed via SSH: optional success, timeout worker termination, partial metrics and recovery of a disconnected virtual display.
- Updated binary installed under `~/.local/bin/ws`, with installer backups. No packages were missing or installed.
- Live recovery remains pending. The approved forced stop initially failed while QEMU was blocked in `vfio_pci_core_disable` / `pcie_flr`; it eventually exited after the kernel reset timeout. The VM is now off. The approved start attempt failed before guest boot with `Unknown PCI header type '127'` for RTX `0000:01:00.0`. No host reboot was performed.
- No live desktop was captured or driven. No working Looking Glass pixels, fresh complete metrics or native Windows boot are claimed.

## SSD slot swap investigation

- The user swapped SSD slots. The configured stable disk ID still resolves to the Crucial CT1000P3PSSD8, serial `24464C379AB9`, now `/dev/nvme1n1`. Windows partition UUID `E07E46097E45D944` matches the saved configuration.
- Windows SSD controller is `0000:02:00.0`, under root port `0000:00:06.0`. The storage hook discovers the controller by vendor/device and verifies disk identity, rather than hardcoding its PCI address. Controller reports `live`; no NVMe timeout or disk I/O errors were found in this boot. This is not a full SSD health or filesystem check.
- The failing PCIe link is the GPU root port `0000:00:01.0`, leading to RTX `0000:01:00.0` and its audio function. Kernel logs show pending-transaction and reset timeouts, failed link retraining, and inability to return from D3cold to D0. GPU config reads returned `ff ff ff ff`; the Windows SSD returned the expected vendor/device bytes.
- The slot swap has not invalidated the VM disk mapping. No causal link between the physical swap and GPU failure is established. The disabled IDD and statistics collector weaknesses were found and patched, but they do not establish the original cause of the GPU/Windows failure.
- Recovery is blocked by the inaccessible GPU. A Linux reboot into the GPU profile was presented for separate approval, because it closes host apps. A full power cycle may be needed if a reboot does not restore the GPU; neither is yet verified.
