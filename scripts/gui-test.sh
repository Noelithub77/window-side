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
click() { omabox click -b "$box" --window class:window-side "$@" >/dev/null; }
start setup
shot welcome 'right beside Linux'
key alt+Right
shot windows 'Choose Windows'
key alt+Right
shot graphics 'Passthrough reserves'
click 700 170
key alt+Right
shot resources 'Leave room for Linux'
# Memory minus, six times: 16 GiB becomes 10 GiB.
for _ in 1 2 3 4 5 6; do click 1091 175; done
shot resources-edited '10[[:space:]]*GiB'
key alt+Right
shot review 'Review setup'
rg -qi 'Virtual display' "$output/review.txt"
rg -qi '10[[:space:]]*GiB' "$output/review.txt"
key alt+Left
shot back-resources 'Leave room for Linux'
key alt+Right
shot review-again 'Review setup'
key ctrl+q
omabox wait -b "$box" --timeout 10s window class:window-side --gone >/dev/null
start status
shot status '7[.]4[[:space:]]*GiB'
# Below the last card the window shows its own ink surface (#0c0c12).
magick "$output/status.png" -format '%[pixel:p{10,850}]' info: > "$output/background.txt"
rg -q '12,12,18' "$output/background.txt"
click 1204 16
shot settings 'Windows VM'
click 1131 16
shot overview-again '7[.]4[[:space:]]*GiB'
key ctrl+q
omabox wait -b "$box" --timeout 10s window class:window-side --gone >/dev/null
start first-boot-start
shot first-boot-start 'Not started'
key Tab space
shot first-boot-drivers 'Drivers needed'
key ctrl+q
omabox wait -b "$box" --timeout 10s window class:window-side --gone >/dev/null
start first-boot-ready
shot first-boot-ready 'Restarting closes'
key Tab space
shot first-boot-complete '7[.]4[[:space:]]*GiB'
jq -n --arg artifacts "$output" '{environment:"isolated-omabox", success:true,
  checked:["welcome then setup steps by keyboard","back navigation","card selection","memory stepper change in review","review before apply","Windows status pixels","ink surface","overview and settings navigation","first boot keyboard start and driver instructions","first boot restart returns to status"],
  hostSetupApplied:false, guestActionsRun:false, artifacts:$artifacts}' | tee "$output/result.json"
