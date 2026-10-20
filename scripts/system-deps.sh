#!/usr/bin/env bash
# Shared by source installs and release archives, before ws can load WebKitGTK.
set -euo pipefail
mode=${1:---help}
case "$mode" in --runtime|--build|--help) ;; *) echo 'Usage: system-deps.sh --runtime|--build [--install]' >&2; exit 2 ;; esac
if [[ "$mode" == --help ]]; then
  echo 'Print required system packages; --install uses the native package manager. No VM or boot configuration changes.'
  exit 0
fi
source /etc/os-release
family="${ID:-} ${ID_LIKE:-}"
case " $family " in
  *' arch '*|*' archlinux '*) manager=pacman; packages=(webkit2gtk-4.1 gtk3 openssl); build=(base-devel rust nodejs pnpm); args=(-S --needed --noconfirm) ;;
  *' debian '*|*' ubuntu '*) manager=apt-get; packages=(libwebkit2gtk-4.1-0); build=(build-essential curl libwebkit2gtk-4.1-dev libgtk-3-dev libssl-dev librsvg2-dev patchelf pkg-config nodejs npm); args=(install -y) ;;
  *' fedora '*|*' rhel '*) manager=dnf; packages=(webkit2gtk4.1 gtk3 openssl-libs); build=(gcc gcc-c++ make rust cargo webkit2gtk4.1-devel gtk3-devel openssl-devel librsvg2-devel patchelf pkgconf-pkg-config nodejs pnpm); args=(install -y) ;;
  *) echo 'Install WebKitGTK 4.1, GTK 3 and OpenSSL using your package manager.' >&2; exit 1 ;;
esac
if [[ "$mode" == --build ]]; then packages+=("${build[@]}"); fi
if [[ "${2:-}" == --install ]]; then
  if (( EUID == 0 )); then exec "$manager" "${args[@]}" "${packages[@]}"; fi
  exec sudo "$manager" "${args[@]}" "${packages[@]}"
fi
printf 'sudo %q' "$manager"
printf ' %q' "${args[@]}" "${packages[@]}"
printf '\n'
if [[ "$mode" == --build ]]; then echo 'Use Rust from rustup, Node.js 22.12 or newer and pnpm; older distro packages may need an upstream install.'; fi
