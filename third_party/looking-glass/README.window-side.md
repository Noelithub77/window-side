# Window Side's Looking Glass fork

This is the complete source archive for Looking Glass **B7-829-cdba29f2**, with
Window Side's client changes applied directly. Bundled dependencies, copyright
notices and licenses are retained. A regular Window Side checkout contains the
source needed to build it; Git submodule initialization is unnecessary.

- Upstream: https://github.com/gnif/LookingGlass
- Archive: https://looking-glass.io/artifact/B7-829/source
- Archive SHA-256: `60d7067b4fa7f42d57fc31b6e85a48baa88652f57d65fa54de98d2070ff478ce`
- Upstream license: GPL-2.0-or-later; see `LICENSE` and dependency licenses.
- Windows IDD remains pinned to `B7-829-cdba29f2`; the wire protocol is unchanged.

## Local changes

- `input:overlayKey`: dedicated overlay toggle, disabled by default.
- `input:ctrlAltCapture`: left Ctrl+Alt capture toggle on chord release,
  preserving other shortcuts, AltGr, repeat behavior and focus-loss recovery.
- `win:softOverlay`: rounded dark mint theme; Settings first and a shortcut hint.
- Sentence case capture notifications and input regression tests.
- `ENABLE_RENDER_TESTS`: allow input tests without GoogleTest or Weston.

Window Side launches its private `window-side-viewer` binary with these settings.
The build output remains named `looking-glass-client` upstream; packaging gives
it a private name so system installations and other VM viewers remain separate.
Use `./ops looking-glass:build|test|status` from the repository root. Build output
stays under `target/`; never build inside this source tree.

When updating upstream, compare this tree with the recorded archive, retain the
local changes, rerun client and app checks, and update the Windows IDD pin only
as part of an explicitly reviewed matching-version update.
