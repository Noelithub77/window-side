# window-side agent instructions

## Purpose

window-side runs the Windows install from a dual boot as a KVM virtual machine
on Linux, or boots it natively. `ws` is one Rust binary: bare `ws` opens the
Tauri 2 app (welcome, guided setup, live overview, settings, tray); every other
command is a JSON-printing CLI for people and agents.

It is a general Linux app, not an Omarchy app. Primary targets:

- Distros: Arch, Ubuntu/Debian and Fedora families (from `/etc/os-release`).
- Desktops: Hyprland, KDE Plasma and GNOME.
- Boot loaders: Limine, GRUB and systemd-boot.
- The Omarchy panel is an optional add-on, offered only on Omarchy.

For UI and desktop integration, read `plan/tauri-app/summary.md` first.
Host and VM platform decisions live in `plan/v2-app/summary.md`.

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
  app.rs, app/           Tauri commands, types, fixtures, metrics, tray, single instance
ui/src/                  React screens, components, styles, browser mocks
ui/src/bindings.ts        Generated Rust command/type bindings
ui/tests/                Playwright Chromium interaction tests
assets/                  guest PowerShell, Omarchy plugin, illustrations, agent prompt
scripts/, tests/         GUI, installer, first-boot and guest checks
packaging/, install.sh   desktop entry, AUR template, one-command installer
plan/                    design notes (`tauri-app` is current)
```

## Commands

Use `./ops --help` first; prefer it over one-off scripts.

- `./ops check`: unit tests, Clippy, CLI, first-boot, plugin and installer fixtures. Run after every change.
- `./ops ci:run [--release]`: dispatch CI on GitHub `main` and watch it; authenticated `gh` and `jq` required. `--release` warms the optimized cache.
- `./ops ui:check`: TypeScript, strict oxlint (`--type-aware --deny-warnings`) and Vite build.
- `./ops ui:bindings` regenerates bindings; `ui:bindings:check` fails on drift.
- `./ops ui:test`: Chromium interaction tests against the built UI.
- `./ops gui:smoke --output /new/abs/dir`: native WebKitGTK fixture smoke in OmaBox.
- `./ops gui:installed --binary FILE --output DIR`: an installed binary in a fresh OmaBox HOME.
- `./ops guest:check`: read-only checks in a running Windows guest. Never start the VM just for a check without asking.
- `./ops dev`: build and replace this checkout's development app.

CI is manual and `v*` tag-only to save minutes. Run `./ops check` locally before
pushing; branch pushes and pull requests do not trigger CI. Debug and release
Rust caches are separate; manual release runs warm the default-branch cache for tags.

## Rules

- No `any`-style escapes: no unsafe casts, `unwrap`, `expect` or `panic!` in runtime paths. Return `anyhow` errors with a clear message.
- Keep files under about 500 lines, comments short, and sentence case everywhere (never full caps).
- Ask before new dependencies, architecture changes or new public behaviour. The approved platform crates include `tauri`, `specta`, `tauri-specta`, `zbus`, `oo7`, `notify-rust` and `ksni`.
- Package lists live once, in `src/detect/distro.rs`. The root side installs only packages known for the plan's distro family and writes only the paths in `setup/files.rs` `ALLOWED`.
- Config changes go through `Config::save` (validated, atomic, with the schema beside it). Agents edit config with `ws config set`, never by hand.
- Use the desktop layer (`display/desktop.rs`) instead of calling `hyprctl` or a desktop's tools directly.

## UI

- Calm, minimal, pastel and rounded (Luma-like). Show state with chips, dots and icons, not sentences.
- One entry point per action; secondary actions go in menus. Destructive actions need a second click.
- The lead owns screens, components, styles and browser mock data in `ui/src/`. Rust owns setup, host and VM logic. `api.ts` adapts generated commands; it contains no business logic.
- Use oxlint, never ESLint. Add no formatters or formatter configs.
- Creative UI work stays with the lead agent; give most mechanical work to one GPT-6.1-Sol subagent on codex-3.

## Verification

- GUI checks run only in an isolated OmaBox. Never launch, drive or capture the live desktop.
- Fixture tests must keep off the real session bus (`tests/first-boot.sh` points `DBUS_SESSION_BUS_ADDRESS` at a missing socket).
- Ubuntu, Fedora, KDE, GNOME, GRUB and systemd-boot paths are unit-tested only unless stated otherwise. Report what each check proved and what stays unverified.

## Git

- Work on `main`. Commit and push each finished step, staging only the task's files.
- Commit messages: a clear title, then a short Markdown body explaining what changed and why.
- Machine config lives in dotfiles (`~/dotfiles/linux/window-side`, stowed to `~/.config/window-side`); this repo holds only source.
