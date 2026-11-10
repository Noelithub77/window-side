#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)
plugin="$root/assets/plugin"
box="window-side-metrics-$$"
omabox up "$box" --no-shell --net isolated --size 1280x800@60 --json >/dev/null
cleanup() { result=$?; if (( result != 0 )); then omabox log -b "$box" run -n 70 >&2 || true; fi; omabox log -b "$box" run -n 200 > "${WINDOW_SIDE_TEST_LOG:-/dev/null}" 2>&1 || true; omabox down "$box" >/dev/null; }
trap cleanup EXIT
box_root=$(omabox path -b "$box")
stage="$box_root/home/window-side-test/shell"
mkdir -p "$stage/plugins" "$box_root/home/window-side-test/bin"
cp -a "$plugin" "$stage/plugins/noel.window-side"
cp -a /usr/share/omarchy/shell/{Commons,Ui} "$stage/"
cp "$root/tests/plugin/shell.qml" "$stage/shell.qml"
# All native actions and guest reads stay mocked in this isolated UI test.
cat > "$box_root/home/window-side-test/bin/ws" <<'MOCK'
#!/bin/sh
if [ "$1" = launch ]; then
  echo 'Windows SSD PCI device is not present' >&2
  echo 'window-side: The Windows SSD is unavailable. Check its status before starting Windows.' >&2
  exit 1
fi
cat "$HOME/window-side-test/metrics.json"
MOCK
printf '%s\n' '{"status":"running","available":true,"cpu":6,"memory":"7.4 GiB / 16.0 GiB","network":"↓ 461.3 MiB\n↑ 63.6 MiB","uptimeSeconds":3349,"bootProfile":"normal","storage":{"status":"ready","error":""},"updates":{"pending":4,"fresh":true,"titles":["Fixture update"],"checkedAt":1792317371}}' > "$box_root/home/window-side-test/metrics.json"
chmod +x "$box_root/home/window-side-test/bin/ws"
omabox run -b "$box" -d -- sh -c 'export PATH="$HOME/window-side-test/bin:$PATH"; exec qs -p "$HOME/window-side-test/shell"'
ipc() { omabox run -b "$box" -- sh -c 'qs -p "$HOME/window-side-test/shell" ipc call window-side-fixture "$@"' sh "$@"; }
for attempt in {1..30}; do
  if ipc state >/dev/null 2>&1; then break; fi
  sleep .2
done
sample='{"status":"running","available":true,"cpu":6,"memory":"7.4 GiB / 16.0 GiB","network":"↓ 461.3 MiB\n↑ 63.6 MiB","uptimeSeconds":3349,"bootProfile":"normal","storage":{"status":"ready","error":""}}'
ipc apply "$sample"
ipc state | jq -e '.available and .shared and .cpu == 6 and .memory == "7.4 GiB / 16.0 GiB" and .uptime == "55m" and .status == "running"'
missing='{"status":"shut off","available":false,"storage":{"status":"missing","error":"Windows SSD not detected"}}'
printf '%s\n' "$missing" > "$box_root/home/window-side-test/metrics.json"
ipc apply "$missing"
ipc state | jq -e '.status == "shut off" and .busy == false and .barError and .errorMessage == "Windows SSD not detected"'
ipc open
if [[ -n "${WINDOW_SIDE_TEST_SCREENSHOT:-}" ]]; then
  omabox shot -b "$box" -o "$WINDOW_SIDE_TEST_SCREENSHOT" >/dev/null
fi
printf '%s\n' "$sample" > "$box_root/home/window-side-test/metrics.json"
ipc apply "$sample"
ipc state | jq -e '.barError == false and .errorMessage == ""'
ipc apply '{"status":"shut off","available":false,"storage":{"status":"unknown","error":"Could not check Windows SSD"}}'
ipc state | jq -e '.barError and .errorMessage == "Could not check Windows SSD"'
ipc apply "$sample"
ipc actionError
ipc apply "$sample"
ipc state | jq -e '.error == "Update failed"'
ipc apply '{"status":"running","available":false,"updates":{"pending":4,"fresh":false}}'
ipc state | jq -e '.available == false and .memory == "—" and .uptime == "—"'
ipc fail
ipc state | jq -e '.available == false and .memory == "—"'
printf '%s\n' '{"status":"shut off","available":false,"bootProfile":"gpu"}' > "$box_root/home/window-side-test/metrics.json"
ipc start
for attempt in {1..30}; do
  if ipc state | jq -e '.busy == false and .status == "shut off" and .error == "The Windows SSD is unavailable. Check its status before starting Windows."'; then
    break
  fi
  sleep .2
done
ipc state | jq -e '.busy == false and .status == "shut off" and .error == "The Windows SSD is unavailable. Check its status before starting Windows."'
printf 'Window Side UI regression passed: guest readings, unavailable values, actual ws startup failure and shared service.\n'
