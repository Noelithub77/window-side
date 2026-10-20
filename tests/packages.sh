#!/usr/bin/env bash
# Every package manager and privilege program is fake and unreachable outside this PATH.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
binary="${WS_TEST_BINARY:-$root/target/debug/ws}"
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/home"
export WS_PACKAGE_STATE="$fixture/installed" WS_PACKAGE_LOG="$fixture/actions"
export HOME="$fixture/home" XDG_CURRENT_DESKTOP=fixture-desktop DBUS_SESSION_BUS_ADDRESS="unix:path=$fixture/no-bus"
unset DISPLAY WAYLAND_DISPLAY
for manager in pacman apt-get dnf rpm dpkg-query; do
  cat > "$fixture/bin/$manager" <<'MANAGER'
#!/bin/bash
printf '%s %s\n' "${0##*/}" "$*" >> "$WS_PACKAGE_LOG"
case "${1:-}" in
  -Q|-q|-W)
    [[ -f "$WS_PACKAGE_STATE" ]] || exit 1
    if [[ "${0##*/}" == dpkg-query ]]; then printf installed; fi ;;
  update) ;;
  -S|install)
    [[ "${WS_PACKAGE_FAIL:-0}" == 0 ]] || exit 7
    : > "$WS_PACKAGE_STATE" ;;
  *) exit 99 ;;
esac
MANAGER
  chmod +x "$fixture/bin/$manager"
done
for privilege in pkexec sudo; do
  cat > "$fixture/bin/$privilege" <<'PRIVILEGE'
#!/bin/bash
printf 'privilege:%s %s\n' "${0##*/}" "$*" >> "$WS_PACKAGE_LOG"
[[ "${1:-}" != -- ]] || shift
exec "$@"
PRIVILEGE
  chmod +x "$fixture/bin/$privilege"
done
package_cli() { PATH="$fixture/bin" "$binary" setup packages "$@"; }
package_cli | jq -e '.environment == "local-packages" and .desktop == "fixture-desktop" and (.missing | length > 0)' >/dev/null
! rg -q '^privilege:' "$WS_PACKAGE_LOG"
package_cli --install | jq -e '.status == "ready" and (.installed | length > 0) and .missing == []' >/dev/null
count=$(rg -c '^privilege:' "$WS_PACKAGE_LOG")
package_cli --install | jq -e '.status == "ready" and .installed == []' >/dev/null
[[ "$(rg -c '^privilege:' "$WS_PACKAGE_LOG")" == "$count" ]]
rm "$WS_PACKAGE_STATE" "$fixture/bin/pkexec"
package_cli --install | jq -e '.status == "ready"' >/dev/null
rg -q '^privilege:sudo -- ' "$WS_PACKAGE_LOG"
rm "$WS_PACKAGE_STATE"
export WS_PACKAGE_FAIL=1
if package_cli --install > "$fixture/out" 2> "$fixture/error"; then echo 'Failed package manager reported success' >&2; exit 1; fi
rg -q 'Package installation failed' "$fixture/error"
[[ ! -e "$WS_PACKAGE_STATE" ]]
unset WS_PACKAGE_FAIL
package_cli --install | jq -e '.status == "ready"' >/dev/null
printf '%s\n' 'Package CLI fixtures passed: missing JSON, native install, verification, idempotency, sudo fallback and failure retry'
