# Boot profile fixtures

These are synthetic running-kernel entries, never files read from or installed
on the test host. Tests cover Limine compatibility, nested Ubuntu GRUB entries,
Fedora BLS metadata, systemd-boot's multiple initrds, and rejection of unsupported
UKIs, unresolved variables and paths outside the boot entry directories.

Implementation references checked on 2026-10-20:

- [grub-reboot(8)](https://manpages.debian.org/trixie/grub2-common/grub-reboot.8.en.html): selects the next boot only; a failed grubenv write must abort the reboot.
- [grubby upstream manual](https://github.com/rhboot/grubby/blob/main/grubby.8): read the running kernel with `--info`, target normal entries by title so updates cannot change the GPU twin.
- [Boot loader specification](https://uapi-group.org/specifications/specs/boot_loader_specification/): preserve kernel paths, all initrds and other entry metadata; options belong to each Type 1 entry.
- [bootctl(1)](https://www.freedesktop.org/software/systemd/man/latest/bootctl.html): `LoaderEntrySelected` identifies the running entry; `systemctl reboot --boot-loader-entry=ID` selects a single boot.
- [initramfs-tools(7)](https://manpages.debian.org/bookworm/initramfs-tools-core/initramfs-tools.7.en.html): modules in `/etc/initramfs-tools/modules` load early, before init-premount.
- [dracut.conf(5)](https://dracut-ng.github.io/dracut/man/dracut.conf.5.html): `force_drivers` includes and loads drivers early; disable stored host command lines so a GPU boot cannot contaminate the normal image.

No test invokes a boot writer, root setup or a reboot. Real Ubuntu/Fedora and
systemd-boot initramfs generation and reboots remain unverified on hardware.
