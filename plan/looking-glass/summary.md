# Looking Glass controls and app ownership

## Decisions

- **[chosen]** Insert toggles the overlay; left Ctrl+Alt toggles capture on the
  first modifier release. Home remains available to Windows. Other keys cancel
  the capture chord; AltGr, repeats and focus-loss recovery have regression tests.
- **[chosen]** Polish the existing controls: rounded dark surfaces, mint accents,
  larger spacing and font, Settings first and a compact shortcut hint.
- **[chosen]** Vendor the pinned source in this repository instead of maintaining
  an external fork/submodule. `third_party/looking-glass/README.window-side.md`
  records the official archive, SHA-256, upstream version and local changes.
- Keep B7-829-cdba29f2 and its Windows IDD protocol. The Windows driver is unchanged.
- The native client stays a separate process owned and installed by Window Side.

## Normal profile

- **[chosen]** All setups use Looking Glass in the normal profile, rendered by
  the driver's software mode. `virt-viewer` stays for first boot, and as a fallback
  that restores the VirtIO display first, since it would otherwise show nothing.
- **[chosen]** Audio uses a private SPICE socket at `/dev/shm/window-side-spice.sock`;
  LGMP has no audio. Input and clipboard stay on LGMP.
- **[chosen]** For integrated GPUs, software mode caps the Windows display at 60 Hz
  and turns off viewer multisampling. `GpuRefresh` keeps the passthrough rate.
- Not yet seen running: software rendering, smoothness on an integrated GPU,
  SPICE audio beside LGMP, and the guest script changes.
- Reaching the frame buffer needs passwordless `sudo setfacl`. Without it Windows
  stays on `virt-viewer` in the normal profile.

## App and packaging

`./ops build`, `dev` and `release:build` build the vendored client into
`target/libexec/window-side/window-side-viewer`. Normal installers copy it into
`PREFIX/libexec/window-side/` and record its checksum in the install receipt.
Unmanaged/modified viewers require explicit replacement and are backed up.
The regular app installer is also used by `./ops looking-glass:install`.

The GPU Open and Launch paths resolve this exact private client from the app's
installation prefix. They pass Insert, Ctrl+Alt, Scroll Lock as the advanced
shortcut leader, the soft theme and font on the command line. No personal
`client.ini` or system `looking-glass-client` is needed. A unique application ID
and executable name keep focus/close operations scoped to Window Side's viewer.
Missing clients and service launch failures return actionable errors.

Release archives contain the viewer and complete corresponding source, including
upstream and dependency licenses. Arch packaging builds that same source and
records its dependencies. Source installs require the pinned upstream's build
dependencies plus FUSE 3 headers; no build step downloads source or submodules.
The installer checks for missing viewer runtime libraries and reports them.

## Verification

- Original client changes: 73 input regressions and isolated Wayland/EGL overlay
  inspection passed. Insert opened/closed the overlay and Esc closed it after
  reopening. The visual fixture used upstream's clipboard file stub because
  OmaBox has no `/dev/fuse`; normal builds retain the real implementation.
- Vendored native build passed with test support disabled for the installed
  viewer. `./ops check` passed 73 client tests, 56 Rust tests, Clippy, UI build
  and the CLI, installer and release/package fixtures. Three new app tests
  verify private viewer selection, argument boundaries/defaults and launch
  error propagation. Another regression covers viewer identification after
  Looking Glass changes its process title. All eight browser interaction tests
  passed.
- The normal installer installed the app and private viewer and recorded both
  checksums; readback matched the build outputs. Installed client help confirmed
  the app's options override a fixture config that still assigns Home.
- Physical Windows input, native Windows boot, other Linux distributions and
  published-release installation remain separate checks. No VM or host reboot
  is required to install these changes. Reopen Window Side after updating.

Previous patch-build and screenshot artifacts remain under
`~/.cache/window-side/looking-glass-controls/`; current build outputs are under
`target/looking-glass/` and `target/libexec/window-side/`.

## Display match and controls (2026-11-10)

- **[chosen]** The viewer opens fullscreen. The client sends its window size
  in physical pixels (`windowW × windowScale`), so Windows gets the panel's
  native resolution. After the viewer starts, `set-display-scale.ps1` sets
  Windows scaling to the host scale snapped to Windows' steps (1.25 → 125%).
  It runs in the signed-in session through a one-shot scheduled task, and waits
  3 s first because Windows stores scale relative to a recommended value that
  changes with resolution.
- **[chosen]** Settings hides the resolution buttons while `status.lookingGlass`
  is set. That flag is the GPU profile, or the last `prepare()` result saved in
  `~/.local/state/window-side/display`. The buttons stay for the VirtIO
  fallback.
- **[chosen]** Overview shows a strip with the viewer keys (Ctrl+Alt, Insert,
  Scroll Lock+F, Scroll Lock+Q) while the Looking Glass display is in use.
  A one-time toast after the first Open or Launch names Ctrl+Alt.
- Not yet run on a guest: the scale script, fullscreen sizing on the IDD, and
  scaling behaviour on multi-monitor hosts (the focused or primary monitor wins).
