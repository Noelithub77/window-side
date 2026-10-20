#!/usr/bin/env bash
# Called only by the fake ws in installer fixtures.
set -euo pipefail
printf '%s\n' "$*" >> "$WS_INSTALL_LOG"
if [[ "$*" == 'setup plugin' ]]; then
  printf '{"status":"installed"}\n'
  exit 0
fi
[[ "$*" == 'setup packages' || "$*" == 'setup packages --install' ]] || exit 91
printf 'Environment: distro family=%s, desktop=%s, Omarchy=%s\n' "$WS_INSTALL_FAMILY" "${XDG_CURRENT_DESKTOP:-unknown}" "${WS_INSTALL_OMARCHY:-no}" >&2
if [[ "$*" == *--install ]]; then
  case "$WS_INSTALL_FAMILY" in
    arch) manager=pacman ;;
    debian) manager=apt-get ;;
    fedora) manager=dnf ;;
    *) echo 'Unsupported fixture distro' >&2; exit 2 ;;
  esac
  # The real package lists and installer flags are tested in Rust, not copied here.
  "${WS_INSTALL_PRIVILEGE:-pkexec}" "$manager" fixture-package
fi
printf '{"environment":"local-packages","distro":"%s","missing":[],"status":"ready"}\n' "$WS_INSTALL_FAMILY"
