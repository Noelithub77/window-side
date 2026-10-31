#!/usr/bin/env bash
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
action=${1:---help}
if [[ $# -eq 2 && "$2" == --help ]]; then action=--help; fi
case "$action" in
  --help|-h|help)
    cat <<'HELP'
Usage: ./ops looking-glass:<build|test|install|status>

Build the pinned B7-829 Linux client with opt-in Insert/Ctrl+Alt controls
and a rounded dark overlay. No new packages, guest changes or VM actions.

build    Download verified source if missing, apply the patch and build.
test     Build and run input regression tests without a desktop.
install  Build, back up the current /usr/local/bin client, then replace it
         atomically with sudo. Does not restart an existing viewer.
status   Print the patch hash and expected build paths without building.

Requires the existing Looking Glass build dependencies, curl, tar, patch,
cmake and jq. Tests use no display, guest, credentials or real session bus.
Build diagnostics go to stderr; completed operations print JSON to stdout.
Machine settings remain in the user's Looking Glass client.ini.
HELP
    exit 0 ;;
  build|test|install|status) ;;
  *) echo 'Use ./ops looking-glass:build|test|install|status' >&2; exit 2 ;;
esac
[[ $# -eq 1 ]] || { echo 'This command accepts no extra arguments' >&2; exit 2; }

tag=B7-829
archive_hash=60d7067b4fa7f42d57fc31b6e85a48baa88652f57d65fa54de98d2070ff478ce
patch_file="$root/assets/looking-glass/controls.patch"
patch_hash=$(sha256sum "$patch_file" | cut -d ' ' -f 1)
cache="${XDG_CACHE_HOME:-$HOME/.cache}/window-side"
archive="$cache/looking-glass-$tag/source.tar.gz"
work="$cache/looking-glass-controls/$patch_hash"
source_dir="$work/source"
build="$work/build"
[[ "$action" != test ]] || build="$work/test-build"

result() {
  local binary=""
  [[ ! -x "$build/looking-glass-client" ]] || binary="$build/looking-glass-client"
  jq -n --arg action "$action" --arg version "$tag" \
    --arg patch "$patch_hash" --arg source "$source_dir" \
    --arg build "$build" --arg binary "$binary" \
    '{action: $action, version: $version, patch_sha256: $patch,
      source: $source, build: $build,
      binary: (if $binary == "" then null else $binary end)}'
}
if [[ "$action" == status ]]; then result; exit 0; fi

mkdir -p "$(dirname "$archive")" "$work"
exec 9>"$work/build.lock"
flock 9
if [[ ! -f "$archive" ]]; then
  download=$(mktemp "${archive}.XXXXXX")
  trap 'rm -f -- "$download"' EXIT
  curl --fail --location --retry 2 "https://looking-glass.io/artifact/$tag/source" -o "$download" >&2
  printf '%s  %s\n' "$archive_hash" "$download" | sha256sum --check >&2
  mv -- "$download" "$archive"
  trap - EXIT
fi
printf '%s  %s\n' "$archive_hash" "$archive" | sha256sum --check >&2
if [[ ! -f "$work/patched" ]]; then
  staging=$(mktemp -d "$work/source.XXXXXX")
  trap 'rm -rf -- "$staging"' EXIT
  tar -xzf "$archive" --strip-components=1 -C "$staging"
  patch --batch --fuzz=0 -p1 -d "$staging" < "$patch_file" >&2
  mv -- "$staging" "$source_dir"
  touch "$work/patched"
  trap - EXIT
fi

tests=OFF
[[ "$action" != test ]] || tests=ON
cmake -S "$source_dir/client" -B "$build" -DCMAKE_BUILD_TYPE=Release \
  -DENABLE_TESTS="$tests" -DENABLE_RENDER_TESTS=OFF >&2
targets=(looking-glass-client)
if [[ "$action" == test ]]; then
  targets=(app-keybind-tests keybind-tests input-tests mouse-tests
    displayserver-input-tests config-message-tests)
fi
cmake --build "$build" --parallel 4 --target "${targets[@]}" >&2
if [[ "$action" == test ]]; then
  DBUS_SESSION_BUS_ADDRESS=unix:path=/missing-window-side-test-bus \
    ctest --test-dir "$build" --output-on-failure \
      -R '^(app-keybind-|keybind-|input-|mouse-|display-input-|config-message-)' >&2
fi
if [[ "$action" == install ]]; then
  destination=/usr/local/bin/looking-glass-client
  backup="$cache/looking-glass-backups/$(date +%Y%m%dT%H%M%S)-$$"
  mkdir -p "$backup"
  if [[ -f "$destination" ]]; then cp -p -- "$destination" "$backup/looking-glass-client"; fi
  sudo install -m 755 "$build/looking-glass-client" "${destination}.window-side-new"
  sudo mv -f -- "${destination}.window-side-new" "$destination"
  cmp -- "$build/looking-glass-client" "$destination"
  result | jq --arg installed "$destination" --arg backup "$backup" \
    '. + {installed: $installed, backup: $backup}'
else
  result
fi
