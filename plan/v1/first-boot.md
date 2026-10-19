# First boot

Continue phase 4 using the approved single-binary design. No new dependencies.

- `ws setup first-boot status` reads the VM, guest agent and driver readiness.
- `start` boots the compatibility configuration and attaches the guest-tools CD;
  installing that CD inside Windows remains the manual step.
- `finish` requires a signed-in Windows user, installed network drivers and a
  working VirtIO storage driver before changing the boot disk. A temporary
  empty VirtIO disk activates that driver while the real boot disk stays SATA.
- Guest provisioning recognizes QEMU before changing Windows. It disables
  Fast Startup and hibernation, sets the shared UTC clock convention, and
  registers a guarded display-startup task for Looking Glass.
- Finishing explicitly closes Windows apps through a clean shutdown, saves the
  VirtIO configuration while stopped, then starts it and verifies the agent.
  Failure to boot leaves the SATA configuration available for recovery.
- The desktop shows only the pending step and its next action. Existing
  configured VMs retain their status view. Guest mutations are never triggered
  by polling, opening the app, or visiting review.
- Plugin installation reuses the bundled assets and preserves dotfiles links.

Verification uses pure transition/file tests and synthetic OmaBox fixtures.
A fresh Windows install and same-drive virtual disk remain separate runtime
checks; do not claim them from fixtures.


Implementation details and limits:

- Fresh configs set `windows.provisioned=false`; completion requires both that
  flag and `vm.virtio=true`. Older already-VirtIO configs keep their status view.
- `recover` refuses a running VM and recovery files for a different disk.
  Rejected domain definitions restore the previous config and XML.
- Unqueried driver values are null, not a false claim about the installed driver.
- Both directory and per-file plugin symlinks are managed externally and preserved.
- `finish` requires the GPU host profile when passthrough was selected. No Linux
  reboot is triggered by opening the window or by readiness polling.

The temporary-disk approach follows the KVM project's
[Windows VirtIO boot guidance](https://www.linux-kvm.org/page/Boot_from_virtio_block_device).
The live-only attachment flags were checked against libvirt's current `virsh`
documentation using Context7. Windows hibernation uses Microsoft's documented
[powercfg command](https://learn.microsoft.com/en-us/windows-hardware/design/device-experiences/powercfg-command-line-options).


Verified on 20 October 2026:

- `./ops check`: 26 unit tests, Clippy with denied warnings, native build,
  fake-guest CLI readiness/rollback and plugin link-preservation checks passed.
- `./ops gui:test`: setup, status, first-boot driver instructions and restart
  navigation passed with synthetic data and disabled guest actions.
- `./ops guest:check`: all three PowerShell scripts parsed; the provisioning
  and Looking Glass installer guards rejected simulated physical identities;
  real existing-VM storage and network readiness were true with `applied=false`.
- The first read-only probe exposed that NetKVM lives in DriverStore on this
  Windows installation. Readiness now resolves the registered driver path,
  including SystemRoot and native path prefixes, instead of assuming the
  legacy System32\drivers location.
- Actual guest setup, transition from SATA, recovery on a real boot disk, a
  same-drive layout and native Windows boot remain unverified.
