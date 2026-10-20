#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
[[ $# == 2 && "$1" == --output && "$2" == /* && ! -e "$2" ]] || { echo 'Usage: ./ops gui:smoke --output /absolute/new-directory' >&2; exit 2; }
output=$2
binary="${WS_GUI_BINARY:-$root/target/debug/ws}"
[[ -x "$binary" ]] || { echo 'Run ./ops build first' >&2; exit 1; }
mkdir -m 700 -p "$output"
box="window-side-tauri-smoke-$$"
cleanup() {
  code=$?
  trap - EXIT
  omabox log -b "$box" run -n 100 > "$output/run.log" 2>&1 || true
  omabox windows -b "$box" --json > "$output/windows.json" 2>&1 || true
  omabox down "$box" >&2 || code=1
  jq -n --argjson success "$([[ "$code" == 0 ]] && echo true || echo false)" --arg output "$output" '{environment:"isolated-omabox", success:$success, artifacts:$output, hostSetupApplied:false, guestActionsRun:false}' > "$output/result.json"
  exit "$code"
}
trap cleanup EXIT
omabox up "$box" --no-shell --net isolated --size 1280x900@60 --seed "$binary:.local/bin/ws" --json > "$output/box.json"
omabox run -b "$box" -d -- env WS_GUI_FIXTURE=setup WS_GUI_SMOKE_OUTPUT=/tmp/window-side-smoke.json GDK_BACKEND=wayland sh -c 'exec "$HOME/.local/bin/ws"'
omabox wait -b "$box" --timeout 40s window class:ws >/dev/null
omabox wait -b "$box" --timeout 40s cmd -- test -s /tmp/window-side-smoke.json >/dev/null
omabox run -b "$box" -- cat /tmp/window-side-smoke.json > "$output/rendering.json"
omabox shot -b "$box" --window class:ws -o "$output/window.png" >/dev/null
jq -e '.webgl and .lottie and .clipboard and (.errors | length == 0)' "$output/rendering.json" >/dev/null
