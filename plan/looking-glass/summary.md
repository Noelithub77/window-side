# Looking Glass controls and overlay

The machine previously used Home as Looking Glass's escape key, which coupled
capture to menu shortcuts and prevented Home reaching Windows.

- **[chosen]** Insert toggles the overlay; left Ctrl+Alt toggles capture on the
  first modifier release. Home remains an ordinary guest key.
- Other keys cancel the capture chord, preserving Ctrl+Alt+Delete and normal
  shortcuts. Right Alt stays available for AltGr. Guest modifiers are released
  before capture changes; focus loss clears pending chord state.
- Keep the installed B7-829-cdba29f2 protocol and Windows driver. Patch the Linux
  client with opt-in settings and preserve the original behavior by default.
- **[chosen]** Polish the existing overlay with rounded dark surfaces, mint accents, larger
  spacing and Settings first. No new UI framework or dependency.

`assets/looking-glass/controls.patch` applies to the checksum-pinned official
source archive. `./ops looking-glass:build`, `looking-glass:test`,
`looking-glass:install` and `looking-glass:status` provide reproducible builds,
input regression checks, installation with backup, and read-only discovery.
Installation does not restart the viewer or either operating system.

Enable in the user's tracked Looking Glass `client.ini`:

```ini
[input]
escapeKey=KEY_SCROLLLOCK
overlayKey=KEY_INSERT
ctrlAltCapture=yes

[win]
softOverlay=yes
uiFont=DejaVu Sans
uiSize=16
```

Scroll Lock remains the leader for advanced shortcuts; tapping it does not
toggle capture while `ctrlAltCapture` is enabled. Insert is reserved for the
overlay. Esc closes it. Closing the overlay restores its earlier capture state.

The patch retains upstream's GPL-2.0-or-later licensing. Source:
[Looking Glass B7-829](https://looking-glass.io/artifact/B7-829/source).
The framebuffer tests requiring GoogleTest and Weston can be disabled separately
from input tests; the normal installed build has test support disabled.

## Verification

- `./ops looking-glass:build` and `looking-glass:install` passed. Installation
  compares the installed file with the built binary and retains the old client
  under `~/.cache/window-side/looking-glass-backups/`.
- `./ops looking-glass:test`: 73 input, keybinding, focus, mouse, display-server
  routing and configuration tests passed, including four new keyboard cases.
- `./ops check`: 52 Rust tests, Clippy, UI checks/build, CLI and installer
  fixtures passed. Vite retains its existing large-chunk advisory.
- OmaBox with the real Wayland/EGL renderer and synthetic test transport:
  inspected the rounded dark overlay, Settings tab and shortcut hint. Insert
  opened/closed the overlay and Esc closed it after reopening.
- The isolated visual fixture linked upstream's existing
  `client/tests/clipboard_files_stub.c` instead of the real clipboard file
  implementation because OmaBox has no `/dev/fuse`. The normal installed
  binary uses the real implementation and has test support disabled.
- Dotfiles `status.sh --arch` reported 148 passes, four warnings and seven
  unrelated symlink failures for existing Codex/T3, Zen/MIME and logind files.
  The Looking Glass config remains linked to its tracked dotfiles source.

Artifacts are in `~/.cache/window-side/looking-glass-controls/`: input/check
logs, installation metadata and `overlay-open.png`, `overlay-closed.png`,
`overlay-esc-closed.png`. Synthetic desktop results do not establish physical
Windows or keyboard behavior. An already running viewer must be reopened to
load the new binary and configuration; Windows does not need a restart.
