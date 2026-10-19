#!/usr/bin/env bash
set -euo pipefail
binary=''; output=''
while (( $# )); do
  case "$1" in
    --binary|--output)
      (( $# >= 2 )) || exit 2
      if [[ "$1" == --binary ]]; then binary=$2; else output=$2; fi
      shift 2 ;;
    *) echo 'Usage: ./ops gui:installed --binary /installed/ws --output /new/directory' >&2; exit 2 ;;
  esac
done
[[ "$binary" == /* && -x "$binary" && "$output" == /* && ! -e "$output" ]] || exit 2
mkdir -m 700 -p "$output"
box="window-side-installed-$$"
omabox up "$box" --no-shell --net isolated --size 1280x900@60 --seed "$binary:.local/bin/ws" --json > "$output/box.json"
cleanup() {
  result=$?
  if (( result != 0 )); then
    omabox log -b "$box" run -n 100 > "$output/failure.log" 2>&1 || true
    omabox windows -b "$box" --json > "$output/failure-windows.json" 2>&1 || true
  fi
  omabox down "$box" >&2
}
trap cleanup EXIT
# A fresh box HOME has no machine config or credentials; never apply setup.
omabox run -b "$box" -d -- sh -c 'exec "$HOME/.local/bin/ws"'
omabox wait -b "$box" --timeout 30s window class:window-side >/dev/null
omabox wait -b "$box" --timeout 15s still --window class:window-side --quiet 600ms >/dev/null
omabox shot -b "$box" --window class:window-side -o "$output/window.png" >/dev/null
tesseract "$output/window.png" stdout 2>/dev/null > "$output/window.txt"
rg -qi 'window.side' "$output/window.txt"
rg -qi 'right beside Linux' "$output/window.txt"
omabox keys -b "$box" --window class:window-side ctrl+q
jq -n --arg binary "$binary" '{environment:"isolated-omabox", binary:$binary,
  success:true, checked:["installed release window","welcome screen","keyboard quit"],
  setupApplied:false, guestActionsRun:false}' | tee "$output/result.json"
