# window-side

<p align="center">
  <a href="https://github.com/Noelithub77/window-side/releases"><img src="docs/assets/hero.svg" alt="window-side — your Windows, beside Linux" width="1000"></a>
</p>

<p align="center">
  <a href="https://github.com/Noelithub77/window-side/releases"><img alt="Latest release including prereleases" src="https://img.shields.io/github/v/release/Noelithub77/window-side?include_prereleases&amp;style=flat&amp;color=8ae0c4&amp;labelColor=101515"></a>
  <a href="LICENSE"><img alt="License: GPL version 2 or later" src="https://img.shields.io/badge/license-GPL--2.0--or--later-c4fbec?style=flat&amp;labelColor=101515"></a>
  <a href="https://github.com/Noelithub77/window-side/stargazers"><img alt="GitHub stars" src="https://img.shields.io/github/stars/Noelithub77/window-side?style=flat&amp;color=c4fbec&amp;labelColor=101515"></a>
</p>

Run the Windows already on your dual-boot drive as a virtual machine inside Linux, and still boot it natively when you need to. One install, one set of apps and files.

```sh
curl -fsSL https://raw.githubusercontent.com/Noelithub77/window-side/main/get.sh | bash
```

The installer verifies the release checksum, installs the app and any missing VM packages, and never starts Windows. Run it again to update. [Installer options](docs/install.md).

![window-side welcome screen](docs/assets/welcome.webp)

## How it works

- **Same install, second door.** KVM boots your existing Windows. A drive of its own is passed whole; a drive shared with Linux exposes only the Windows partition.
- **Look first, approve once.** Guided setup detects storage, graphics and CPU, shows every file it will write, then asks for your password once.
- **Familiar first boot.** Windows starts on hardware it already knows, installs its drivers, then moves to fast VirtIO. A failed switch rolls back.
- **Three ways to look.** Headless for agents, a console, or a passed-through GPU shown through [Looking Glass](https://looking-glass.io/).
- **Built for agents.** Every `ws` command prints JSON: start Windows, run PowerShell, copy files, use SSH.

```sh
ws start
ws exec --json '$PSVersionTable.PSVersion.ToString()'
ws metrics
ws stop
```

## Status

v0.1 is a prerelease. Arch with Hyprland and Limine is tested end to end. Ubuntu, Debian, Fedora, KDE Plasma, GNOME, GRUB and systemd-boot have unit and fixture coverage only.

## Thanks

[Looking Glass](https://github.com/gnif/LookingGlass), by Geoffrey McRae and contributors, owns the display path for GPU passthrough. window-side is an independent companion that sets up the VM around it.

## Learn more

[Setup and commands](docs/guide.md) · [Development](docs/development.md) · [Releases](https://github.com/Noelithub77/window-side/releases) · [Report a bug](https://github.com/Noelithub77/window-side/issues)

If window-side saves you a reboot, a star helps someone else find it.

## License

Copyright © 2026 Noel Georgi. Free software under [GPL-2.0-or-later](LICENSE), matching Looking Glass. Dependencies and third-party assets keep their own licenses.
