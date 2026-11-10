# Automatic Windows mounts

- [chosen] Automatically clear the configured Windows volume's dirty flag when the ordinary Dolphin drive click cannot mount it, only while Windows is fully off. Handle native Windows boots as well as VM shutdowns.
- Found the existing release hook absent from this host. Installed it through a focused root setup plan; reloaded libvirt while all VMs were off.
- `./ops disk:integration --preview|--apply` installs the project-owned NTFS mount helper and release/start hook on this merged `/usr/bin` host. It refuses to replace another installer's helper. No extra mount options or forced mounts are needed.
- The helper checks system libvirt state, locks the volume, checks the startup marker, and tries the normal kernel mount. Repair requires an unmounted, readable, dirty volume without `hiberfil.sys`. Failed repair stops the mount. Other NTFS volumes delegate directly to the standard kernel mount.
- The libvirt prepare hook takes the same lock, refuses mounted partitions, and sets the ownership marker before QEMU starts. Release removes it and performs the existing guarded cleanup. No libvirt calls run inside the hook.
- Native Windows boots do not fire libvirt hooks; the Dolphin mount helper covers the next Linux mount instead. A hibernated Windows install still requires a full Windows shutdown.
- `./ops check` passed: five focused Python regression cases, 61 Rust tests, strict Clippy, UI checks/build and native/installer fixtures. Shell syntax checks passed.
- Installed both files using the focused root plan and restarted libvirt with all VMs off. The actual Windows partition remounted read/write through `udisksctl`, and Downloads was accessible. The helper created its lock state, confirming UDisks invoked it.
- A disposable real NTFS loop image exercised dirty mount failure, successful repair and mount without force, mounted-volume startup refusal, and startup-marker mount refusal. The image, loop device and fixture state were removed afterward.
- No real VM boot/stop cycle, native Windows reboot, or live Dolphin interaction was performed. No broader filesystem-health claim is made. Full check log: `/tmp/window-side-dolphin-check.log`.
