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
| Windows location | **[chosen]** Auto-detected. Own drive: pass the whole drive. Same drive as Linux: a virtual disk assembled at boot by a root service from a small image (GPT and a copy of the Windows boot files) plus the real Windows partition, joined with device-mapper. Linux partitions are never exposed. |
| Colours | **[chosen]** Near-black and teal by default: evergreen surfaces, pine-teal selection, mint accents and frosted-mint text. An optional Omarchy appearance reads the active `colors.toml`; missing colors produce an error and preserve teal. |

## Behaviour change

Bare `ws` used to start Windows. It now opens the window. Every existing caller
(Omarchy panel, desktop entries, agents) already passes an explicit command
such as `ws launch`.

## Setup flow

The window walks through these steps; `ws setup` accepts the same choices as
flags so agents can run it without the window.

1. Omarchy plugin: offered only when `omarchy` is installed.
2. Windows: every NTFS or BitLocker system partition, with its drive. The mode
   follows from the drive: whole drive when it holds no Linux partitions,
   virtual disk otherwise. BitLocker is flagged because the VM's TPM differs.
   Fast Startup and hibernation are turned off from inside Windows at first boot.
3. GPU passthrough: GPUs whose IOMMU group holds only that GPU's functions,
   or "no passthrough".
4. Memory and CPU split: sliders bounded by the host, defaulting to half.
5. Review: every file to be written, then one `pkexec` prompt to apply.
6. First boot: SATA disk and e1000e network until the VirtIO drivers and guest
   agent are installed from the driver CD (the one manual step, in the console).
   Then ws switches to VirtIO, reads the Windows user name, disables Fast
   Startup, installs the Looking Glass display with passthrough, and offers the
   keyring sign-in.

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

## Window implementation

Phase 3 uses one GPUI root with a status view or a four-step setup wizard.
The window owns navigation, choices and in-flight operations; the existing
setup engine owns detection, plans and privileged apply. Windows usage comes
from the same `metrics` command as the panel. Blocking work runs off the UI
thread. Nothing is applied merely by opening the window or reaching review.

Theme definitions live in `src/gui/theme.rs`; views read semantic tokens.
`appearance.toml` stores only the appearance choice beside the machine config.
Normal CLI commands keep their terminal behavior; bare `ws` opens the window.

The first-boot driver and guest provisioning flow is implemented in phase 4;
its fresh-install and native-boot runtime checks remain open. Public
release packaging remains phase 5; a working window does not prove those flows.

### Phase 3 verification, 19 October 2026

- Native debug build, 24 unit tests and Clippy with warnings denied passed.
- `./ops gui:test` passed in an invisible OmaBox: keyboard step transitions,
  pointer/radio choices, resource changes via sliders and keyboard buttons,
  review/back navigation, status pixels and appearance menu dismissal.
- The resource page was also inspected at the 640×500 minimum window size.
- The fixtures use synthetic hardware and disable guest/setup actions. No
  privileged setup, Windows boot, GPU handoff or live desktop launch was tested.
- The installed global `ws` still uses the legacy launcher. The new window is
  available from this checkout's `target/debug/ws`; installer migration is pending.


### Phase 4 implementation, 20 October 2026

- Added shared first-boot readiness, compatibility boot, explicit finish and
  stopped-only SATA recovery. No new dependencies or Python were added.
- First boot detects the Windows console user, verifies live viostor and staged
  NetKVM, disables hibernation/Fast Startup, installs the matching Looking Glass
  display when configured, and registers a VM-aware display startup task.
- The VirtIO transition uses a readonly scratch disk, a clean shutdown, saved
  recovery files, and guest-agent readback after the cold start. Completion is
  stored separately from the selected disk bus, so failed verification remains
  pending. Original power values are saved inside Windows.
- The desktop shows first-boot steps only for pending installs. Its restart
  control explicitly says it closes Windows apps. Existing completed configs
  keep the normal status view; old configs remain compatible.
- `setup plugin` installs bundled assets or reports externally managed links;
  it preserves both directory and per-file symlinks.
- Automatic keyring sign-in is available through the existing terminal command;
  an inline desktop credential prompt is not implemented.
- Verification: 26 unit tests, Clippy with warnings denied, CLI readiness and
  rollback fixtures, plugin symlink fixtures, and the extended isolated GUI
  keyboard/pixel workflow passed. The native debug build passed.
- The existing Windows VM was booted headlessly for read-only checks. All three
  PowerShell scripts parsed without errors, the simulated physical identities
  rejected both provisioning entry points, and the real guest reported both
  VirtIO drivers ready. This caught and corrected a DriverStore path assumption.
  These checks are reusable with `./ops guest:check` and never pass `-Apply` on
  the real machine.
- No real provisioning, driver installation, storage-bus transition, native
  Windows boot or Linux reboot was performed. The test VM was shut down
  afterward. The installed global `ws` still uses the legacy script.


### Phase 5 implementation, 20 October 2026

- Added a local source/archive installer with explicit replacement, backups,
  dry runs and symlink-safe replacement. It installs app files only.
- Built the optimized binary, verified the release-profile unit/Clippy/CLI
  checks, and installed it as the global `~/.local/bin/ws`. Dotfiles no longer
  stows the legacy launcher or desktop entry; its operator delegates to ws.
- Added Arch CI and tag-triggered draft prerelease packaging. The source
  archive/checksum metadata for AUR is generated from a reviewed tag archive;
  no package was submitted to the AUR.
- The installed optimized binary opened in an isolated fresh HOME. Window pixels,
  the keyboard appearance menu, dismissal and quit passed; no setup was applied.
- Release archive checksums and a temporary-prefix installation passed.
  `makepkg --printsrcinfo` accepted generated metadata from a source fixture.
- Repository visibility remains private. Fresh installation, actual storage
  migration, same-drive assembly and native Windows boot remain runtime gaps;
  this packaging work does not establish a stable public release.
