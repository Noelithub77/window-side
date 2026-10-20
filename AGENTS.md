# window-side agent instructions

## Purpose

window-side runs the Windows install from a dual boot as a KVM virtual machine
on Linux, or boots it natively. `ws` is one Rust binary: bare `ws` opens the
GPUI app (welcome, guided setup, live overview, settings, tray); every other
command is a JSON-printing CLI for people and agents.

It is a general Linux app, not an Omarchy app. Primary targets:

- Distros: Arch, Ubuntu/Debian and Fedora families (from `/etc/os-release`).
- Desktops: Hyprland, KDE Plasma and GNOME.
- Boot loaders: Limine today; GRUB and systemd-boot are planned.
- The Omarchy panel is an optional add-on, offered only on Omarchy.

Decisions and the build order live in `plan/v2-app/summary.md`. Read it before
larger work and keep it current.

## Layout

```text
src/
  main.rs, cli.rs        clap entry; bare `ws` opens the GUI
  config.rs, config/     config.json, its JSON Schema, `ws config`
  detect/                Windows installs, GPUs, CPUs, distro family
  setup.rs, setup/       plan and apply: domain XML, allowlisted root files, pkexec side
  vm/                    libvirt lifecycle, virsh, guest drivers
  guest/                 guest agent, metrics, updates, sign-in keyring, SSH
  display/               desktop layer (Hyprland, KDE, GNOME), console, Looking Glass
  first_boot.rs          SATA to VirtIO first boot with rollback
  host.rs                boot profile, GPU checks, hostdev XML, reboot into an entry
  agents.rs              detects Claude Code, Codex, OpenCode for agent setup
  notify.rs, process.rs  notifications and command helpers
  gui.rs, gui/           GPUI app: views, widgets, theme, tray, single instance
assets/                  guest PowerShell, Omarchy plugin, illustrations, agent prompt
scripts/, tests/         GUI, installer, first-boot and guest checks
packaging/, install.sh   desktop entry, AUR template, one-command installer
plan/                    design notes (`v2-app` is current)
```

## Commands

Use `./ops --help` first; prefer it over one-off scripts.

- `./ops check`: unit tests, Clippy, CLI, first-boot, plugin and installer fixtures. Run after every change.
- `./ops gui:test --output /new/abs/dir`: onboarding, setup, overview and settings in an isolated OmaBox with synthetic hardware (`WS_GUI_FIXTURE`).
- `./ops gui:installed --binary FILE --output DIR`: an installed binary in a fresh OmaBox HOME.
- `./ops guest:check`: read-only checks in a running Windows guest. Never start the VM just for a check without asking.
- `./ops dev`: build and replace this checkout's development app.

## Rules

- No `any`-style escapes: no unsafe casts, `unwrap`, `expect` or `panic!` in runtime paths. Return `anyhow` errors with a clear message.
- Keep files under about 500 lines, comments short, and sentence case everywhere (never full caps).
- Ask before new dependencies, architecture changes or new public behaviour. The approved platform crates are `zbus`, `oo7`, `notify-rust` and `ksni`; GPUI comes from `gpui-kit` (other GPUI libraries target an incompatible `gpui`).
- Package lists live once, in `src/detect/distro.rs`. The root side installs only packages known for the plan's distro family and writes only the paths in `setup/files.rs` `ALLOWED`.
- Config changes go through `Config::save` (validated, atomic, with the schema beside it). Agents edit config with `ws config set`, never by hand.
- Use the desktop layer (`display/desktop.rs`) instead of calling `hyprctl` or a desktop's tools directly.

## UI

- Calm, minimal, pastel and rounded (Luma-like). Show state with chips, dots and icons, not sentences.
- One entry point per action; secondary actions go in menus. Destructive actions need a second click.
- Embed every icon through `gui/assets.rs`; ship illustrations as PNGs rendered from the SVGs.
- Creative UI work stays with the lead agent; give most mechanical work to one GPT-6.1-Sol subagent on codex-3.

## Verification

- GUI checks run only in an isolated OmaBox. Never launch, drive or capture the live desktop.
- Fixture tests must keep off the real session bus (`tests/first-boot.sh` points `DBUS_SESSION_BUS_ADDRESS` at a missing socket).
- Ubuntu, Fedora, KDE, GNOME, GRUB and systemd-boot paths are unit-tested only unless stated otherwise. Report what each check proved and what stays unverified.

## Git

- Work on `main`. Commit and push each finished step, staging only the task's files.
- Commit messages: a clear title, then a short Markdown body explaining what changed and why.
- Machine config lives in dotfiles (`~/dotfiles/linux/window-side`, stowed to `~/.config/window-side`); this repo holds only source.
