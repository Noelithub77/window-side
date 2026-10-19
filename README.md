# window-side

Run your real dual-boot Windows install as a headless VM from Linux, and boot
the same install natively when you need it.

Work in progress; see `plan/v1/summary.md`.

The built `ws` binary opens the native setup/status window. `ws --help` and explicit commands
stay in the terminal. The default appearance is black and teal; the
Appearance menu optionally reads Omarchy's `colors.toml`. If that theme file is
unavailable, teal stays active and the window explains the missing file.

The wizard detects Windows partitions and graphics, offers resource allocation,
and previews the setup before administrator approval. Opening it does not
configure disks or restart the computer. The first-boot guest driver workflow
and public packaging are still planned; see the phase list above.

Development commands:

```sh
./ops build
./ops check
./ops gui:test --output /tmp/window-side-ui-check
```

The GUI check uses synthetic hardware and read-only fixtures in an invisible
OmaBox session. It verifies native pixels and keyboard navigation; it does not
apply setup, boot Windows, or verify a real GPU handoff.
