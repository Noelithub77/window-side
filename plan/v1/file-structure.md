# File structure

```text
window-side/
  Cargo.toml              package window-side, binary ws
  src/
    main.rs               no arguments: window; otherwise CLI
    cli.rs                clap commands and dispatch
    config.rs             ~/.config/window-side/config.toml
    notify.rs             desktop notifications
    host/                 boot profile, disks, GPUs and IOMMU groups, CPU topology
    vm/                   virsh calls, domain XML, start/stop/wait
    guest/                guest agent, PowerShell, copy, updates, metrics, sign-in, SSH
    display/              Looking Glass client and IDD install, Hyprland focus
    setup/                choices -> plan of files, privileged apply
    gui/                  GPUI window: setup wizard, status and settings
  assets/
    guest/                PowerShell scripts embedded with include_str!
    plugin/               Omarchy plugin installed by ws
  plan/v1/
```
