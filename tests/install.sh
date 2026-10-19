#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
export HOME="$fixture/home" XDG_STATE_HOME="$fixture/state"
mkdir -p "$HOME" "$fixture/bin"
unset DISPLAY WAYLAND_DISPLAY DBUS_SESSION_BUS_ADDRESS
cat > "$fixture/bin/ws" <<'BINARY'
#!/usr/bin/env bash
[[ "$*" == --version ]] || exit 91
printf 'ws 0.1.0\n'
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
printf '%s\n' 'Installer fixtures passed: dry run, spaced prefix, upgrade, edit refusal and symlink preservation'
