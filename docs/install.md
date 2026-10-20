# Installation

The [README](../README.md#quick-install) has the one-line installer. It downloads the version selected in `get.sh` from GitHub Releases, checks its SHA-256 digest, and runs the bundled installer. This includes published prereleases; it does not depend on GitHub’s stable-only `latest` redirect.

The archive targets Linux x86_64 and is built on Arch. Runtime and VM package setup supports Arch, Ubuntu/Debian and Fedora. Only Arch with Hyprland has runtime verification; other families have fixture coverage.

## Installer options

Download `get.sh` to inspect it or pass installer options:

```sh
curl -fsSL https://raw.githubusercontent.com/Noelithub77/window-side/main/get.sh -o get.sh
bash get.sh --dry-run
bash get.sh --yes --no-plugin
```

`--dry-run` downloads and verifies the archive but does not install it. `--yes` accepts the optional Omarchy panel; `--no-plugin` skips it. Missing packages use pkexec or sudo and can require administrator authentication. Without a terminal the panel is skipped unless `--yes` is passed. Rerunning interactively offers previously skipped or declined panels again.

The installer writes under `~/.local`, including the launcher, desktop entry and icons. Ensure `~/.local/bin` is on your shell’s search path. Machine configuration stays in `~/.config/window-side` and is changed through the app or `ws config set`.

Unknown existing files require `--replace-existing`; the installer preserves backups under `~/.local/state/window-side/install-backups`. Symlinks are replaced themselves without overwriting their targets. An existing app process must be quit and reopened to use the updated binary.

## From a release archive

Extract the archive and run `./install.sh`. Optional flags include `--prefix /absolute/prefix`, `--binary /absolute/ws`, `--replace-existing`, `--dry-run`, `--yes` and `--no-plugin`.

## From source

```sh
git clone https://github.com/Noelithub77/window-side.git
cd window-side
./install.sh
```

Source builds need Rust, Node.js 22.12 or newer, pnpm and the platform development packages. `bash scripts/system-deps.sh --build` prints the exact package command. The installer builds the UI before the locked Rust release.

For local changes, the T3 **Local deploy** action runs `./ops release:build` followed by the installer with the panel skipped. It updates the installed app without opening it. `./ops dev` builds and launches a separate checkout-local development process; agents run graphical apps only inside OmaBox.

The Arch package recipe can be generated with `./ops aur:prepare` from a reviewed source archive. It has not been published to the Arch user repository.
