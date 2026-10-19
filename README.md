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
configure disks or restart the computer. The first-boot guest workflow is implemented, with fixture verification; a fresh
Windows installation and native-boot compatibility still need runtime testing.
Public packaging and migration of the installed legacy launcher remain planned.

Development commands:

```sh
./ops build
./ops check
./ops gui:test --output /tmp/window-side-ui-check
```

The GUI check uses synthetic hardware and read-only fixtures in an invisible
OmaBox session. It verifies native pixels and keyboard navigation; it does not
apply setup, boot Windows, or verify a real GPU handoff.


First boot, from the built binary:

```sh
target/debug/ws setup first-boot status
target/debug/ws setup first-boot start
# Inside Windows: sign in and run virtio-win-guest-tools.exe from the driver CD.
target/debug/ws setup first-boot finish
```

`finish` closes Windows apps through a clean shutdown and restarts the VM after
checking the storage and network drivers. If GPU passthrough was selected, use
the GPU boot profile first. This provisions the shared Windows install: Fast
Startup and hibernation are disabled, including on native boots. The original
power values are recorded in `C:\ProgramData\window-side\power-before.json`.
Automatic sign-in remains optional through `ws sign-in setup` in a terminal;
credentials stay in the Linux login keyring.

If the VirtIO transition fails, stop Windows and run
`target/debug/ws setup first-boot recover` to restore the saved SATA definition.
Recovery checks the configured disk identity and never force-stops Windows.
`setup first-boot status` reports saved completion separately from live driver
readings; unqueried readings are null. Completion is saved only after the
post-transition guest agent answers.

`target/debug/ws setup plugin` repairs the bundled Omarchy panel. It returns
`managed-externally` and leaves existing directory or file symlinks intact.

Developer fixtures:

```sh
./ops check
./ops test:first-boot
./ops guest:check  # Read-only checks; requires an already running QEMU guest.
./ops gui:test --output /tmp/window-side-first-boot-check
```

The CLI fixtures fake libvirt and the guest agent; the GUI fixtures fake guest
actions in an invisible OmaBox. Neither provisions Windows or changes a real
boot disk. The first-boot sources and test boundaries are described in
`plan/v1/first-boot.md`.
