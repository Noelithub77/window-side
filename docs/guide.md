# Setup and command guide

## Windows setup

Keep Windows partitions unmounted on Linux while the VM uses them. The setup
engine supports a separate Windows drive and has an experimental same-drive
mode that exposes only the Windows partition with copied EFI boot files.
BitLocker can require its recovery key because the VM has a different TPM.
Use a backed-up installation; same-drive support is not yet runtime verified.

Setup turns on `libvirt-guests` so a Linux shutdown or reboot first shuts
Windows down cleanly (up to two minutes) instead of cutting its power, which
leaves the real disk needing chkdsk. It never starts Windows at boot.

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
`C:\ProgramData\window-side\power-before.json`. It also restarts the guest
agent if a slow boot times out its start, but only inside the VM.

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
Selecting one copies [`assets/agent-setup.md`](../assets/agent-setup.md) so it can
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
Window Side includes its customized Looking Glass client and opens that private
viewer from the app, `ws console`, `ws looking-glass` and `ws launch` in both
profiles. In the normal profile Windows renders on the CPU, so its display runs
at 60 Hz. `virt-viewer` remains for first boot, and takes over if the Windows
display driver or shared memory is unavailable. Sound reaches the viewer through
a private SPICE socket, so a VM started before this change has none until its
next restart. The viewer passes the keyboard and theme settings itself: Insert toggles the
overlay, left Ctrl+Alt toggles capture on release, and Home reaches Windows.
The app installer updates the viewer along with `ws`; a separate system client
or personal `client.ini` is unnecessary. `ws looking-glass-setup` installs the
matching Windows IDD. Host reboot is explicit.

## Agent skills

[`skills/`](../skills) holds three agent skills: `window-side` (run Windows and
its shells), `windows-app-testing` (build, install and test Windows software)
and `window-side-setup` (setup and first boot). `ws` bundles the same files:

```sh
ws skills            # where each skill lives and whether it matches this ws
ws skills install    # write missing or outdated copies
```

Skills go to `~/.agents/skills` (Codex and OpenCode) and, when Claude Code is
installed, `~/.claude/skills`. A skill folder or file that is a symlink reports
`linked` and is left alone. Without `ws`, `npx skills add
Noelithub77/window-side -g` installs them from the repository.
