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

Extract a release archive, then run one command to install or update the app,
its desktop entry and missing VM packages:

```sh
./install.sh
```

On Omarchy it also offers the optional panel. Rerunning the same command updates
`ws`, checks packages again and offers anything previously skipped or declined.
It prints the detected distro family, desktop and Omarchy status. Packages use
Arch's pacman, Ubuntu/Debian's apt-get or Fedora's dnf through pkexec or sudo;
administrator authentication is still required.

From source:

```sh
git clone https://github.com/Noelithub77/window-side.git
cd window-side
./install.sh
ws
```

The source installer builds with the checked-in Cargo lockfile. Source builds
need Rust, Node.js, pnpm and the Tauri WebKitGTK/GTK development packages;
`scripts/system-deps.sh --build` prints the exact command for your distro.

The installer puts the binary and application entry under `~/.local` and
preserves machine configuration. VM creation and boot entry changes remain in
onboarding; installation does not start a VM or open the app. The setup window
detects Windows partitions and graphics,
lets you choose resources, and reviews machine changes before administrator
approval. GPU boot profiles support Limine with mkinitcpio, Ubuntu/Debian GRUB with
initramfs-tools, Fedora GRUB with dracut/BLS, and systemd-boot Type 1 entries.
Detection needs a readable entry for the running kernel; unreadable boot files,
unsupported unified images or missing tools disable the GPU choice. New paths
are fixture-tested only; real initramfs generation and reboots remain unverified.
GRUB and systemd-boot twins refer to the running kernel: rerun setup after
changing kernels before selecting that GPU profile.

Useful installer options:

```sh
./install.sh --yes                 # Accept the optional panel without a prompt
./install.sh --yes --no-plugin     # Install packages and app, skip the panel
./install.sh --dry-run             # No build, writes, package install or plugin change
./install.sh --binary /path/to/ws --prefix /absolute/prefix
./install.sh --binary /path/to/ws --replace-existing
```

Without a terminal the panel is skipped unless `--yes` is supplied. A skipped
panel is offered again on the next interactive run. To inspect or repair only
VM packages, use `ws setup packages` (JSON) or `ws setup packages --install`.
Package lists live in `src/detect/distro.rs`; the shell installer delegates to
the installed `ws`. A dry run can query packages only when a built binary exists.

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
file symlinks report `managed-externally` and stay intact. The installer offers
the panel on Omarchy, or accepts it with `--yes`; `--no-plugin` always skips it.

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

The welcome screen detects Claude Code, Codex and OpenCode when installed.
Selecting one copies [`assets/agent-setup.md`](assets/agent-setup.md) so it can
be pasted into that agent. The command does not launch a terminal or agent.
The prompt keeps the agent read-only until you approve `ws setup apply`.

The native API also exposes `launch_agent(agent)` separately: it opens the
chosen agent in `xdg-terminal-exec`, then falls back to desktop terminals,
with the setup prompt as one argument and its own process group.
`agent_prompt(agent)` only returns text. Browser mocks and native fixtures
resolve agent launches without starting processes.

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

The desktop is a Tauri 2 WebKitGTK window with a React UI in `ui/`.
The same `ws` binary embeds the built UI and retains every JSON CLI command.
The existing ksni tray and D-Bus single-instance service stay in Rust.

Source builds need Rust, Node.js 22.12 or newer, pnpm and the system development packages:

```sh
bash scripts/system-deps.sh --build  # prints the exact distro install command
pnpm -C ui install --frozen-lockfile
./ops ui:check
./ops build
./ops check
./ops ui:test
./ops gui:smoke --output /tmp/window-side-tauri-check
```

`./ops ui:dev` runs Vite on 5199. To open the development webview explicitly,
run `./ops ui:dev --window` inside OmaBox with the Vite server reachable at
5199. Ordinary `./ops dev` builds the embedded UI and opens the app; agents
must run it inside OmaBox. `./ops ui:bindings` generates typed IPC from a Rust
test, and `./ops ui:bindings:check` rejects stale output. No code generation
runs during app startup. Lint uses strict type-aware oxlint, with no formatter.

Runtime packages are WebKitGTK 4.1 and GTK 3 (`webkit2gtk-4.1` on Arch,
`libwebkit2gtk-4.1-0` on Debian/Ubuntu, `webkit2gtk4.1` on Fedora), plus OpenSSL.
`install.sh` bootstraps these if the release binary cannot load its libraries.
The release archive includes `scripts/system-deps.sh`; source builds install
frontend dependencies and build the UI before Cargo embeds it.

CLI fixtures fake libvirt and the guest agent. Browser interaction tests use
synthetic mocks. Native fixture modes are `setup`, `status`, `stopped`,
`first-boot-start`, `first-boot-ready` and `unavailable` through
`WS_GUI_FIXTURE`; they keep all command effects in memory, without VM, setup
or keyring access.
`gui:smoke` creates an invisible OmaBox, checks the real WebKitGTK canvas and
bundled Lottie renderer and clipboard fallback, and saves a screenshot and
JSON evidence.
`guest:check` requires an already running QEMU guest and reads script syntax,
VM guards and driver readiness without applying provisioning.

CI runs only on manual dispatch and `v*` tags to save GitHub Actions minutes;
branch pushes and pull requests do not start it. Run `./ops check` locally before
pushing. After pushing to `main`, `./ops ci:run` dispatches and watches debug
checks; `./ops ci:run --release` checks the optimized build and warms its separate
cache. These commands require authenticated `gh` and `jq`. Weekly pacman packages,
the lockfile-keyed pnpm store and Rust dependencies/builds are cached. Tags restore
default-branch caches without saving tag-only entries. Warm the release cache
manually before tagging when dependencies change.

Tag runs build on Arch, verify the tag against the binary version and
create a **draft prerelease** with the archive and checksum. Release promotion
and AUR publication remain manual. Packaging refuses dirty source unless
`--allow-dirty` explicitly marks a local preview. See `plan/v1/` for decisions,
verification evidence and remaining runtime work.
