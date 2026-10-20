# window-side

One Windows installation: boot it natively when you need it, or run it as a
headless KVM guest while working in Linux. Agents can execute PowerShell,
transfer files and test Windows applications without maintaining a second
Windows install.

`ws` opens a native window: a short welcome, guided setup, a live overview
with CPU, memory, GPU and network graphs, and settings. Explicit commands such
as `ws start`, `ws exec` and `ws ssh` stay in the terminal. The app and CLI are
one Rust binary with embedded Windows scripts; no Python runtime is required.
Looking Glass provides the passthrough desktop, while window-side manages the
VM, machine setup, guest commands and optional Omarchy panel around it.

## Current status

Prerelease for x86_64 with UEFI Windows and libvirt/KVM. Setup installs
packages on Arch, Ubuntu/Debian and Fedora, and the app raises its windows and
reads the refresh rate on Hyprland, KDE Plasma and GNOME. Only Arch with
Hyprland has been run end to end; the Ubuntu, Fedora, KDE and GNOME paths are
covered by unit tests so far. The existing
Windows VM, CLI, installation and isolated native interface have been checked.
Fresh Windows provisioning, the SATA-to-VirtIO transition, same-drive virtual
disk assembly and native-boot compatibility remain unverified end to end.
The repository remains private while those checks are open. VMware is outside
this version's scope.

## Install

From a release archive, extract it and run `./install.sh`. From source:

```sh
git clone https://github.com/Noelithub77/window-side.git
cd window-side
./install.sh
ws
```

The source installer builds with the checked-in Cargo lockfile. Build tools on
Arch are `base-devel`, `rust`, `clang`, `cmake`, `pkgconf`, `libxcb`,
`libxkbcommon`, `libxkbcommon-x11`, `fontconfig`, `wayland`, `vulkan-headers`
and `vulkan-icd-loader`. A working Vulkan driver is needed for the window.

The installer puts the binary and application entry under `~/.local` and
preserves machine configuration. It does not start a VM, install packages or
change boot entries. The setup window detects Windows partitions and graphics,
lets you choose resources, and reviews machine changes before administrator
approval. GPU boot profiles currently use Limine on Arch/Omarchy.

Useful installer options:

```sh
./install.sh --dry-run
./install.sh --binary /path/to/ws --prefix /absolute/prefix
./install.sh --binary /path/to/ws --replace-existing
```

Unknown existing files require `--replace-existing`. Backups go under
`~/.local/state/window-side/install-backups`; links are replaced themselves,
so their source files are not overwritten. `~/.local/bin` must be on PATH.
The Arch package recipe is generated with `./ops aur:prepare` from a reviewed
tag source archive; it has not been published to the AUR.

## Windows setup

Keep Windows partitions unmounted on Linux while the VM uses them. The setup
engine supports a separate Windows drive and has an experimental same-drive
mode that exposes only the Windows partition with copied EFI boot files.
BitLocker can require its recovery key because the VM has a different TPM.
Use a backed-up installation; same-drive support is not yet runtime verified.

First boot uses SATA and an emulated network adapter until the guest drivers
are installed. The desktop guides the flow; agents can use:

```sh
ws setup detect
ws setup plan --gpu none
ws setup first-boot status
ws setup first-boot start
# Inside Windows: run virtio-win-guest-tools.exe from the driver CD, then sign in.
ws setup first-boot finish
```

`finish` verifies the drivers, closes Windows apps through a clean shutdown,
and restarts with VirtIO. Configured passthrough needs the GPU boot profile
first. Provisioning disables Fast Startup and hibernation in the shared Windows
install, including native boots. Original power values are recorded in
`C:\ProgramData\window-side\power-before.json`.

If the transition fails, stop Windows and run `ws setup first-boot recover` to
restore the saved SATA definition. Recovery verifies disk identity and never
force-stops Windows. Completion is saved only after the post-transition guest
agent answers; unqueried driver readings are null.

`ws setup plugin` installs the bundled Omarchy panel. Existing directory or
file symlinks report `managed-externally` and stay intact. The installer does
not install or enable the panel automatically.

## Configuration

Setup writes `~/.config/window-side/config.json` beside a generated
`config.schema.json`; window preferences live in `preferences.json`. A
`config.toml` from older versions is converted once and kept as
`config.toml.migrated`. Every command prints JSON:

```sh
ws config path                       # config, schema and VM definition paths
ws config show                       # the whole config, defaults included
ws config get vm.memory_mib
ws config set vm.memory_mib 12288    # validates, then redefines stopped Windows
ws config set windows.refresh_hz 144
ws config validate
ws config schema
```

`set` reports the value before and after. Memory and CPU changes go through the
same host limits and VM rendering as setup, and are refused while Windows
runs. Disk and GPU values change system files, so they stay with
`ws setup plan` and `ws setup apply`; first-boot state stays with
`ws setup first-boot`.

## Agent access

The welcome screen can hand setup to a local agent. "Set up with an agent"
lists Claude Code, Codex and OpenCode when installed and opens the chosen one
in a terminal (`xdg-terminal-exec`, then the desktop's own terminal) with
[`assets/agent-setup.md`](assets/agent-setup.md) as its first prompt. "Copy
prompt" puts the same text on the clipboard for an agent elsewhere. The prompt
keeps the agent read-only until you approve `ws setup apply`.

```sh
ws --help
ws status
ws start
ws exec --file /absolute/check.ps1 --timeout 60 --json
ws ssh '$PSVersionTable.PSVersion.ToString()'
ws ssh --put /absolute/input.bin 'C:/Users/YourUser/AppData/Local/Temp/input.bin'
ws metrics
ws stop
```

`start` is headless; `launch` also opens the viewer, and `console` opens a
running guest. Guest-agent PowerShell runs as Windows SYSTEM in session 0.
SSH uses the configured user, an existing key and strict host-key trust;
server provisioning is separate. Neither shell path automatically enters the
visible Windows desktop. GUI automation needs a signed-in interactive session.

`ws sign-in setup` or Settings optionally stores a console credential in the
Linux login keyring (GNOME Keyring or KDE Wallet). Native Windows auto-login stays off. Starting the VM
from an unlocked Linux session can then open the Windows desktop.

The normal host profile keeps the GPU on Linux. The passthrough profile gives
Windows exclusive use of the selected GPU until Linux reboots normally.
Looking Glass requires a client matching the embedded IDD build; `ws
looking-glass-setup` installs the guest side. Host reboot is explicit.

## Development and release

Run `./ops dev` to build and open the development app. Re-run it after edits
to replace the previous development instance from this checkout; a failed build
keeps that instance running. Installed apps and CLI commands are left running.
The tracked `t3.json` declares the same command as the **Dev** action. Import it
once through T3 Code’s Settings → Project → Actions → Import scripts.

```sh
./ops check
./ops check:release
./ops gui:test --output /tmp/window-side-ui-check
./ops gui:installed --binary /absolute/ws --output /tmp/window-side-installed-check
./ops guest:check
./ops release:package --output /absolute/new-release-directory
```

CLI fixtures fake libvirt and the guest agent. GUI fixtures use synthetic
hardware in an invisible OmaBox with guest/setup actions disabled. The installed
GUI check uses a fresh box HOME and only checks the welcome screen and quit.
`guest:check` requires an already running QEMU guest and reads script syntax,
VM guards and driver readiness without applying provisioning.

Tag workflows build on Arch, verify the tag against the binary version and
create a **draft prerelease** with the archive and checksum. Release promotion
and AUR publication remain manual. Packaging refuses dirty source unless
`--allow-dirty` explicitly marks a local preview. See `plan/v1/` for decisions,
verification evidence and remaining runtime work.
