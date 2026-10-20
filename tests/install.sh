#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
export HOME="$fixture/home" XDG_STATE_HOME="$fixture/state"
mkdir -p "$HOME" "$fixture/bin"
unset DISPLAY WAYLAND_DISPLAY
export DBUS_SESSION_BUS_ADDRESS="unix:path=$fixture/no-bus"
for utility in bash dirname realpath sha256sum grep date mkdir cp mktemp install chmod sed mv rm jq desktop-file-validate rg cat ln tail wc script update-desktop-database; do
  if command -v "$utility" >/dev/null; then ln -s "$(command -v "$utility")" "$fixture/bin/$utility"; fi
done
export PATH="$fixture/bin" WS_INSTALL_LOG="$fixture/actions.log"
export WS_INSTALL_FAMILY=arch XDG_CURRENT_DESKTOP=Hyprland
export WS_PACKAGE_FIXTURE="$root/tests/package-fixture.sh"
for command in pkexec sudo; do
  cat > "$fixture/bin/$command" <<'PRIVILEGE'
#!/usr/bin/env bash
printf 'privilege:%s %s\n' "${0##*/}" "$*" >> "$WS_INSTALL_LOG"
[[ "${1:-}" != -- ]] || shift
exec "$@"
PRIVILEGE
  chmod +x "$fixture/bin/$command"
done
for command in pacman apt-get dnf; do
  cat > "$fixture/bin/$command" <<'MANAGER'
#!/usr/bin/env bash
printf 'manager:%s %s\n' "${0##*/}" "$*" >> "$WS_INSTALL_LOG"
[[ "${WS_INSTALL_FAIL:-0}" == 0 ]]
MANAGER
  chmod +x "$fixture/bin/$command"
done
# Keep a fake Omarchy command ready for the explicit panel fixtures.
printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/bin/omarchy-hidden"
cat > "$fixture/bin/ws" <<'BINARY'
#!/usr/bin/env bash
if [[ "$*" == --version ]]; then printf 'ws 0.1.0\n'; else exec bash "$WS_PACKAGE_FIXTURE" "$@"; fi
BINARY
chmod +x "$fixture/bin/ws"
prefix="$fixture/prefix with spaces"
install_app() { "$root/install.sh" --binary "$fixture/bin/ws" --prefix "$prefix" "$@"; }
install_app --dry-run | jq -e '.applied == false and .environment == "local-install"' >/dev/null
[[ ! -e "$prefix" && ! -e "$XDG_STATE_HOME" ]]
install_app | jq -e '.applied == true' >/dev/null
[[ "$("$prefix/bin/ws" --version)" == 'ws 0.1.0' ]]
desktop-file-validate "$prefix/share/applications/window-side.desktop"
grep -Fxq "Exec=\"$prefix/bin/ws\" launch" "$prefix/share/applications/window-side.desktop"
install_app | jq -e '.applied == true' >/dev/null
printf 'user edit\n' >> "$prefix/bin/ws"
if install_app >"$fixture/out" 2>"$fixture/error"; then echo 'Overwrote a modified binary' >&2; exit 1; fi
grep -q 'Existing unmanaged file' "$fixture/error"
# A Stow-style symlink must be replaced, not followed into the source checkout.
source_file="$fixture/legacy-source"
printf 'legacy launcher\n' > "$source_file"
rm "$prefix/bin/ws"
ln -s "$source_file" "$prefix/bin/ws"
if install_app >"$fixture/out" 2>"$fixture/error"; then echo 'Replaced a symlink without explicit intent' >&2; exit 1; fi
install_app --replace-existing > "$fixture/result.json"
jq -e '.applied == true' "$fixture/result.json" >/dev/null
backup=$(jq -r .backup "$fixture/result.json")
[[ ! -L "$prefix/bin/ws" && -L "$backup/bin/ws" ]]
grep -Fxq 'legacy launcher' "$source_file"
grep -Fxq 'legacy launcher' "$backup/ws-before"
# Preserve the target of an existing desktop symlink too.
source_desktop="$fixture/legacy-desktop"
printf 'old desktop\n' > "$source_desktop"
rm "$prefix/share/applications/window-side.desktop"
ln -s "$source_desktop" "$prefix/share/applications/window-side.desktop"
install_app --replace-existing >/dev/null
[[ ! -L "$prefix/share/applications/window-side.desktop" ]]
grep -Fxq 'old desktop' "$source_desktop"
# Installation through a directory link must never modify a managed checkout.
linked="$fixture/linked-prefix"
mkdir -p "$linked" "$fixture/managed-bin"
ln -s "$fixture/managed-bin" "$linked/bin"
if "$root/install.sh" --binary "$fixture/bin/ws" --prefix "$linked" --replace-existing >"$fixture/out" 2>"$fixture/error"; then exit 1; fi
[[ ! -e "$fixture/managed-bin/ws" ]]
# Exercise every distro delegation and both privilege routes without host writes.
for family in arch debian fedora; do
  export WS_INSTALL_FAMILY=$family
  install_app --yes --no-plugin > "$fixture/result.json"
  jq -e --arg family "$family" '.packages.distro == $family and .packages.status == "ready"' "$fixture/result.json" >/dev/null
done
export WS_INSTALL_PRIVILEGE=sudo
install_app --yes --no-plugin >/dev/null
rg -q 'privilege:sudo' "$WS_INSTALL_LOG"
unset WS_INSTALL_PRIVILEGE
export WS_INSTALL_FAIL=1
if install_app --yes --no-plugin >"$fixture/out" 2>"$fixture/error"; then echo 'Package failure reported success' >&2; exit 1; fi
rg -q 'package setup failed' "$fixture/error"
export WS_INSTALL_FAIL=0 WS_INSTALL_OMARCHY=yes
mv "$fixture/bin/omarchy-hidden" "$fixture/bin/omarchy"
chmod +x "$fixture/bin/omarchy"
install_app --yes --no-plugin > "$fixture/result.json"
jq -e '.plugin.status == "skipped"' "$fixture/result.json" >/dev/null
[[ "$(cat "$XDG_STATE_HOME/window-side/install-plugin-choice")" == skipped ]]
install_app --yes > "$fixture/result.json"
jq -e '.plugin.status == "installed"' "$fixture/result.json" >/dev/null
[[ "$(cat "$XDG_STATE_HOME/window-side/install-plugin-choice")" == installed ]]
# Non-interactive reruns skip, but --yes offers the panel again on the next run.
install_app </dev/null > "$fixture/result.json"
jq -e '.plugin.status == "skipped"' "$fixture/result.json" >/dev/null
install_app --yes > "$fixture/result.json"
jq -e '.plugin.status == "installed"' "$fixture/result.json" >/dev/null
before=$(wc -l < "$WS_INSTALL_LOG")
install_app --dry-run --yes > "$fixture/result.json"
jq -e '.applied == false and .plugin.status == "would-ask"' "$fixture/result.json" >/dev/null
[[ "$(tail -n +"$((before + 1))" "$WS_INSTALL_LOG")" == 'setup packages' ]]
# A declined interactive prompt is worded as a retry, never treated as permanent.
printf 'declined\n' > "$XDG_STATE_HOME/window-side/install-plugin-choice"
if command -v script >/dev/null; then
  printf 'y\n' | script -qec "bash '$root/install.sh' --binary '$fixture/bin/ws' --prefix '$prefix'" /dev/null > "$fixture/prompt.txt"
  rg -q 'declined last time' "$fixture/prompt.txt"
  [[ "$(cat "$XDG_STATE_HOME/window-side/install-plugin-choice")" == installed ]]
fi
printf '%s\n' 'Installer fixtures passed: preview, updates, package families, privilege routes, failure retry, panel choices and symlink preservation'
