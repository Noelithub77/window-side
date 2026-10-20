The first window-side prerelease: your existing Windows installation, available from Linux through a Rust + Tauri desktop app and a JSON command line.

- Guided detection and setup, with a review before host changes.
- VM start/stop, live resource graphs, guest PowerShell, SSH and file transfer.
- Looking Glass integration for configured GPU passthrough.
- Mint application icons, desktop launcher and an optional Omarchy panel.
- GPL version 2 or later, matching Looking Glass.

## Install or update

```sh
curl -fsSL https://raw.githubusercontent.com/Noelithub77/window-side/main/get.sh | bash
```

The installer verifies the release archive, installs the app and missing VM packages, and preserves machine configuration. It does not start Windows. Alternatively, extract the Linux x86_64 archive and run `./install.sh`. Source is available from this tag; the archive includes the license and build revision.

## Verification limits

Built and checked on Arch Linux x86_64. Ubuntu/Debian, Fedora, KDE, GNOME, GRUB and systemd-boot paths have fixture or unit coverage only. Fresh Windows provisioning, SATA-to-VirtIO transition, same-drive storage and native-boot compatibility remain unverified end to end. This is a prerelease, not a stable release.

[Looking Glass](https://looking-glass.io/) provides the passthrough display. window-side is an independent companion; thanks to its authors and contributors.
