# File structure

```text
window-side/
  Cargo.toml              package window-side, binary ws
  src/
    main.rs               no arguments: window; otherwise CLI
    cli.rs                clap commands and dispatch
    config.rs             ~/.config/window-side/config.toml
    notify.rs             desktop notifications
    host.rs               current boot profile and GPU ownership checks
    detect/               disks, GPUs, IOMMU groups, CPU topology
    vm/                   virsh calls, domain XML, start/stop/wait
    guest/                guest agent, PowerShell, copy, updates, metrics, sign-in, SSH
    display/              Looking Glass client and IDD install, Hyprland focus
    setup/                choices -> plan of files, privileged apply
    first_boot.rs         driver readiness, guest provisioning, VirtIO transition and recovery
    gui.rs                native window lifecycle, async reads and actions
    gui/                  teal theme, setup wizard, status, review and fixtures
  ops                     local build, checks and isolated GUI test commands
  scripts/gui-test.sh     native pixel and input checks inside OmaBox
  tests/first-boot.sh     CLI checks with a fake libvirt/guest and temporary config
  assets/
    guest/                PowerShell scripts embedded with include_str!
    plugin/               Omarchy plugin installed by ws
  plan/v1/
```
