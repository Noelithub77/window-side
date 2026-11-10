#!/usr/bin/env bash
# Page-switch frame timing in the real WebKitGTK window, inside OmaBox with a fixture.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
[[ $# == 2 && "$1" == --output && "$2" == /* && ! -e "$2" ]] || { echo 'Usage: ./ops gui:perf --output /absolute/new-directory' >&2; exit 2; }
output=$2
binary="${WS_GUI_BINARY:-$root/target/debug/ws}"
[[ -x "$binary" ]] || { echo 'Run ./ops build first' >&2; exit 1; }
mkdir -m 700 -p "$output"
box="window-side-perf-$$"
cleanup() {
  code=$?
  trap - EXIT
  omabox log -b "$box" run -n 100 > "$output/run.log" 2>&1 || true
  omabox down "$box" >&2 || code=1
  exit "$code"
}
trap cleanup EXIT
omabox up "$box" --no-shell --net isolated --size 1280x900@60 --seed "$binary:.local/bin/ws" --json > "$output/box.json"
omabox run -b "$box" -d -- env WS_GUI_FIXTURE=status WS_GUI_PERF_OUTPUT=/tmp/window-side-perf.json GDK_BACKEND=wayland sh -c 'exec "$HOME/.local/bin/ws"'
omabox wait -b "$box" --timeout 40s window class:window-side >/dev/null
omabox wait -b "$box" --timeout 90s cmd -- test -s /tmp/window-side-perf.json >/dev/null
omabox run -b "$box" -- cat /tmp/window-side-perf.json > "$output/perf.json"
jq -e '.errors | length == 0' "$output/perf.json" >/dev/null
jq '{environment: "isolated-omabox", idleMs, scenarios}' "$output/perf.json"
