#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
action=${1:---help}
if [[ $# -eq 2 && "$2" == --help ]]; then action=--help; fi
case "$action" in
  --help|-h|help)
    cat <<'HELP'
Usage: ./ops looking-glass:<build|test|install|status>

Build the vendored Looking Glass client; no source download or submodule step.
build    Build the native viewer into target/libexec/window-side/.
test     Build and run input regression tests without a desktop.
install  Build the viewer and install it with target/debug/ws through the
         regular Window Side installer; run ./ops build first.
status   Report source, build and viewer paths without building.

Requires the Looking Glass build dependencies documented in
third_party/looking-glass/doc/build.rst, cmake, make and jq.
Build/test/status use only checkout files and build outputs. Install delegates
to the app installer, including its VM package check/install and file backups.
Running viewers and guests are not restarted.
Diagnostics go to stderr; successful operations print JSON to stdout.
HELP
    exit 0 ;;
  build|test|install|status) ;;
  *) echo 'Use ./ops looking-glass:build|test|install|status' >&2; exit 2 ;;
esac
[[ $# -eq 1 ]] || { echo 'This command accepts no extra arguments' >&2; exit 2; }
source_dir="$root/third_party/looking-glass"
build="$root/target/looking-glass/build"
[[ "$action" != test ]] || build="$root/target/looking-glass/test-build"
viewer="$root/target/libexec/window-side/window-side-viewer"
result() {
  jq -n --arg action "$action" --arg version "$(cat "$source_dir/VERSION")" \
    --arg source "$source_dir" --arg build "$build" --arg viewer "$viewer" \
    '{action: $action, version: $version, source: $source, build: $build, viewer: $viewer}'
}
if [[ "$action" == status ]]; then result; exit 0; fi
mkdir -p "$root/target/looking-glass"
exec 9>"$root/target/looking-glass/build.lock"
flock 9
tests=OFF
[[ "$action" != test ]] || tests=ON
cmake -S "$source_dir/client" -B "$build" -DCMAKE_BUILD_TYPE=Release \
  -DOPTIMIZE_FOR_NATIVE=OFF -DENABLE_TESTS="$tests" -DENABLE_RENDER_TESTS=OFF >&2
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
else
  mkdir -p "$(dirname "$viewer")"
  temporary=$(mktemp "${viewer}.XXXXXXXX")
  trap 'rm -f -- "$temporary"' EXIT
  install -m 755 "$build/looking-glass-client" "$temporary"
  mv -f -- "$temporary" "$viewer"
  trap - EXIT
fi
if [[ "$action" == install ]]; then
  app="$root/target/debug/ws"
  [[ -x "$app" ]] || { echo 'Build Window Side with ./ops build before installing' >&2; exit 1; }
  exec bash "$root/install.sh" --binary "$app" --viewer "$viewer" --no-plugin
else
  result
fi
