# window-side v2 app

window-side becomes a general, easy-to-onboard dual-boot app for any Linux
user: the native window is the whole product. Ubuntu, Arch (Omarchy) and Fedora
are supported distros; Hyprland, KDE Plasma and GNOME are fully supported
desktops. The Omarchy panel stays an optional add-on, offered only on Omarchy.

## Decisions

| Topic | Choice |
|---|---|
| Visual style | **[chosen]** Soft, rounded, pastel-on-ink: large radii, gradient washes, SVG illustrations rendered with `img`, short one-shot animations. No infinite animations except spinners. |
| Graphs | **[chosen]** Smooth filled area waves (GPUI Kit `AreaChart`) for CPU, memory, GPU and network rate, each with its own pastel colour. |
| Panel parity | **[chosen]** Start, open, shut down, force stop (two-step confirm), boot profile switch with reboot (two-step confirm), errors and storage state live in the window. |
| Settings | **[chosen]** In-app: VM resources (via setup), sign-in credential, Looking Glass display, resolution, Windows Update, appearance. Omarchy panel and Omarchy colours only when Omarchy is installed. |
| Tray | **[chosen]** `ksni` (StatusNotifierItem over D-Bus, no GTK). Closing keeps ws in the tray when a tray host exists (KDE, Ubuntu GNOME, Hyprland bars); otherwise closing quits. No autostart. |
| GPU usage | **[chosen]** Guest metrics read Windows GPU engine counters; the GPU wave shows only when Windows reports them. |
| Distros | **[chosen]** Ubuntu/Debian, Arch and Fedora families, detected from `/etc/os-release` (`ID`, `ID_LIKE`). Each maps packages, firmware paths and services. |
| Bootloaders | **[chosen]** GPU boot profiles for Limine (mkinitcpio), GRUB (initramfs-tools on Ubuntu, dracut and BLS on Fedora) and systemd-boot. |
| Desktops | **[chosen]** Window focus and refresh rate for Hyprland (`hyprctl`), KDE (KWin D-Bus, `kscreen-doctor`) and GNOME (Mutter D-Bus); other desktops fall back to reopening the viewer and 60 Hz. |
| Platform crates | **[chosen]** `zbus` for KWin and Mutter D-Bus (focus, refresh rate, tray host), `oo7` for the Secret Service (GNOME Keyring, KDE Wallet) instead of `secret-tool`, `notify-rust` for notifications instead of `notify-send`. GPUI stays on gpui-kit: other GPUI libraries target `gpui` 0.2 and cannot share its elements. |
| Config | **[chosen]** `config.json` (machine) and `preferences.json` (portable), migrated once from TOML. `ws config path/show/get/set/validate/schema` print JSON; `set` validates, redefines stopped Windows for memory and CPU, and refuses disk, GPU and first-boot keys. `schemars` generates `config.schema.json`. |
| Agent setup | **[chosen]** Welcome offers "Set up with an agent": installed Claude Code, Codex or OpenCode open in a terminal with `assets/agent-setup.md` as the prompt, or the prompt is copied. The agent stays read-only until the user approves `ws setup apply`. |
| Install | **[chosen]** One command installs everything: it checks the environment (distro, desktop, Omarchy), installs packages with the native package manager, installs `ws` and its desktop entry, asks about the Omarchy panel on Omarchy, then opens the app. Running it again is the updater: it updates `ws`, adds missing packages and asks again for anything still not set up, such as a declined Omarchy panel. Machine changes (VM, boot entries) stay in onboarding. Package lists live once, in Rust. |

## Order

1. UI polish, onboarding, dashboard waves, panel actions and settings.
2. Distro layer: os-release detection, package and firmware maps. Done.
   Unknown distros get a clear "install these commands" message instead of
   a package install. Ubuntu needs no AppArmor file: libvirt's `virt-aa-helper`
   is expected to allow the `<shmem>` device (unverified on Ubuntu).
3. Desktop layer: focus and refresh rate on Hyprland, KDE and GNOME. Done.
   KDE raises windows with a KWin script over D-Bus and reads rates from
   `kscreen-doctor -j`; GNOME reads rates from Mutter and reopens the viewer
   because it has no API to raise another app's window. `oo7` and
   `notify-rust` replace `secret-tool` and `notify-send`. KDE and GNOME paths
   are untested on those desktops.
4. GPU usage in guest metrics. Done; parsed and checked with sample counters,
   not yet read from a running Windows guest.
5. Tray. Done: ksni item with state, start, open, shut down, the window
   and quit; closing keeps the app in the tray only while a bar hosts it.
   A second `ws` launch shows the running window over D-Bus.
6. GRUB and systemd-boot GPU profiles with initramfs-tools and dracut. Done:
   running-entry detection, normal/GPU twins, early vfio modules and one-shot
   selection; Limine keeps its existing tools. Unsupported or unreadable entries
   disable GPU setup. Firmware-selected BLS entries may omit the optional version;
   initramfs-tools/dracut rebuild the initrd referenced by the entry. Fixture files
   cover each loader, output paths and generated command lines.
   Root writes, initramfs generation and real reboots remain unverified; no boot
   writer or privileged setup command was run on this host. GRUB/systemd-boot
   twins stay on the setup kernel and need setup again after kernel changes.
7. One-command install. Done: `install.sh` updates the app and desktop entry,
   calls `ws setup packages --install` for the single Rust package list, prints
   environment facts and offers the optional Omarchy panel. `--yes`,
   `--no-plugin` and `--dry-run` support unattended installs and previews;
   skipped/declined panels are offered again. Machine setup stays in onboarding,
   and the installer leaves opening the app to the user. `./ops check` and
   `./ops test:install` cover fake package managers/privilege tools, idempotency,
   failure retry, interactive panel choices and isolated prefixes. Real distro
   package transactions remain unverified; no host package installation ran.

Each step is committed separately with its verification. Ubuntu and Fedora
checks run in containers (detection, package plans, generated boot files)
when Docker access is available; until then they are unit tests only;
real reboots on those distros stay unverified until tested on hardware.
