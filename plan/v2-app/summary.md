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
| Install | **[chosen]** One command installs everything: it checks the environment (distro, desktop, Omarchy), installs packages with the native package manager, installs `ws` and its desktop entry, asks about the Omarchy panel on Omarchy, then opens the app. Running it again is the updater: it updates `ws`, adds missing packages and asks again for anything still not set up, such as a declined Omarchy panel. Machine changes (VM, boot entries) stay in onboarding. Package lists live once, in Rust. |

## Order

1. UI polish, onboarding, dashboard waves, panel actions and settings.
2. Distro layer: os-release detection, package and firmware maps.
3. Desktop layer: focus and refresh rate on Hyprland, KDE and GNOME.
4. GPU usage in guest metrics.
5. Tray.
6. GRUB and systemd-boot GPU profiles with initramfs-tools and dracut.
7. One-command install.

Each step is committed separately with its verification. Ubuntu and Fedora
checks run in containers (detection, package plans, generated boot files);
real reboots on those distros stay unverified until tested on hardware.
