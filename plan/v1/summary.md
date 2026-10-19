# window-side v1

One real Windows install that boots natively for games and anti-cheat, and runs
as a headless KVM guest for everything else: agents can run commands, copy
files, sign in and test Windows apps from Linux.

## Decisions

| Topic | Choice |
|---|---|
| Language | **[chosen]** One Rust crate, package `window-side`, binary `ws`. No Python. |
| Interface | **[chosen]** `ws` with no arguments opens a GPUI window; `ws <command>` is the CLI; `ws --help` prints CLI help. |
| UI kit | **[chosen]** GPUI Kit (`gpui-kit`, Longbridge), using its components wherever one exists. Same base as leet. |
| Backend | **[chosen]** libvirt/KVM with optional GPU passthrough and Looking Glass. VMware is not part of v1. |
| Omarchy plugin | **[chosen]** Bundled in this repo and installed by `ws` so its version always matches the binary. |
| Repo | **[chosen]** `Noelithub77/window-side`, private until v1 works, then public. Noel's dotfiles track it as the submodule `apps/window-side`. |
| Personal values | Never committed here. Setup detects them and writes `~/.config/window-side/config.toml`. |

## Behaviour change

Bare `ws` used to start Windows. It now opens the window. Every existing caller
(Omarchy panel, desktop entries, agents) already passes an explicit command
such as `ws launch`.

## Setup flow

The window walks through these steps; `ws setup` accepts the same choices as
flags so agents can run it without the window.

1. Omarchy plugin: offered only when `omarchy` is installed.
2. Windows disk: whole disks holding a Windows partition. Refuse any disk with
   mounted Linux partitions or the running root. Check that Fast Startup and
   hibernation are off before first boot.
3. GPU passthrough: GPUs whose IOMMU group holds only that GPU's functions,
   or "no passthrough".
4. Memory and CPU split: sliders bounded by the host, defaulting to half.
5. Review: every file to be written, then one `pkexec` prompt to apply.
6. First boot: driver CD, guest agent, Looking Glass display (with passthrough),
   then the optional keyring sign-in.

## Phases

1. CLI parity: port `ws`, the guest helper, metrics, sign-in and SSH into the
   crate. Embed the PowerShell scripts. Dotfiles switches to the binary.
2. Setup engine: host detection, domain XML and system file generation,
   privileged apply, all reachable from the CLI.
3. GPUI window: setup wizard when unconfigured, status and settings otherwise.
4. First-boot flow and plugin install.
5. README, GitHub release workflow, AUR `window-side` package (conflicts with
   the unrelated AUR `ws`), then make the repo public.

## Dependencies

`clap`, `serde`, `serde_json`, `toml`, `anyhow`, `base64`, `gpui-kit`. The
keyring goes through `secret-tool`, and file locking uses `std::fs::File::lock`.

Host tools called at runtime: `virsh`, `lsblk`, `findmnt`, `udisksctl`,
`secret-tool`, `curl`, `bsdtar`, `xorriso`, `virt-viewer` or
`looking-glass-client`, and `hyprctl` when Hyprland runs.

Phase 1 drops two SPICE-only measurement tools from the old scripts:
`frames` and `display-check`.
