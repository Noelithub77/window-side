#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
if (( $# != 2 )) || [[ "$1" != --output || "$2" != /* || -e "$2" ]]; then
  echo 'Usage: ./ops gui:test --output /absolute/new-directory' >&2
  exit 2
fi
output=$2
for command in omabox tesseract jq magick; do
  command -v "$command" >/dev/null || { echo "Missing test tool: $command" >&2; exit 1; }
done
[[ -x "$root/target/debug/ws" ]] || { echo 'Run ./ops build first' >&2; exit 1; }
mkdir -m 700 -p "$output"
box="window-side-ui-$$"
omabox up "$box" --no-shell --net isolated --size 1280x900@60 --json > "$output/box.json"
cleanup() {
  result=$?
  if (( result != 0 )); then omabox log -b "$box" run -n 70 > "$output/failure.log" 2>&1 || true; fi
  omabox down "$box" >&2
}
trap cleanup EXIT
start() {
  omabox run -b "$box" -d --env "WS_GUI_FIXTURE=$1" -- "$root/target/debug/ws"
  omabox wait -b "$box" --timeout 30s window class:window-side >/dev/null
}
shot() {
  omabox wait -b "$box" --timeout 10s still --window class:window-side --quiet 500ms >/dev/null
  omabox shot -b "$box" --window class:window-side -o "$output/$1.png" >/dev/null
  tesseract "$output/$1.png" stdout 2>/dev/null > "$output/$1.txt"
  rg -qi "$2" "$output/$1.txt" || { cat "$output/$1.txt" >&2; return 1; }
}
key() { omabox keys -b "$box" --window class:window-side "$@"; }
start setup
shot windows 'Choose Windows'
key alt+Right
shot graphics 'Passthrough reserves'
omabox click -b "$box" --window class:window-side 29 232 >/dev/null
key alt+Right
shot resources 'Leave room for Linux'
omabox click -b "$box" --window class:window-side 350 285 >/dev/null
key Tab Tab space
key Tab space space
shot resources-edited 'Windows memory'
key alt+Right
shot review 'Review setup'
rg -qi 'Virtual display' "$output/review.txt"
rg -qi 'Memory[[:space:]]+10' "$output/review.txt"
key alt+Left
shot back-resources 'Leave room for Linux'
key alt+Right
shot review-again 'Review setup'
key ctrl+q
omabox wait -b "$box" --timeout 10s window class:window-side --gone >/dev/null
start status
shot status '7[.]4[[:space:]]*GiB'
key Tab space
shot appearance-menu 'Teal'
key Escape
shot appearance-dismissed '7[.]4[[:space:]]*GiB'
# Read the window surface itself, outside text/controls; proves teal's near-black base.
magick "$output/status.png" -format '%[pixel:p{30,760}]' info: > "$output/background.txt"
rg -q '0,0,0|#000000' "$output/background.txt"
jq -n --arg artifacts "$output" '{environment:"isolated-omabox", success:true,
  checked:["setup steps by keyboard","back navigation","radio selection","slider and keyboard resource changes in review","review before apply","Windows status pixels","keyboard appearance menu and dismissal","black teal surface"],
  hostSetupApplied:false, guestActionsRun:false, artifacts:$artifacts}' | tee "$output/result.json"
