# Setup and command guide

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
Looking Glass requires a client matching the embedded IDD build; `ws
looking-glass-setup` installs the guest side. Host reboot is explicit.

