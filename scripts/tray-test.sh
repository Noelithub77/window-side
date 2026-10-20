#!/usr/bin/env bash
# Real compositor and D-Bus checks, confined to two fresh isolated OmaBoxes.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
if (( $# != 2 )) || [[ "$1" != --output || "$2" != /* || "$2" == *[$'\n\r\t']* || -e "$2" ]]; then
  echo 'Usage: ./ops gui:tray --output /absolute/new-directory' >&2
  exit 2
fi
output=$2
binary="$root/target/debug/ws"
for command in omabox busctl jq pgrep timeout; do
  command -v "$command" >/dev/null || { echo "Missing test tool: $command" >&2; exit 1; }
done
[[ -x "$binary" ]] || { echo 'Run ./ops build first' >&2; exit 1; }
mkdir -m 700 -p -- "$output"
boxes=()
stage=starting
box=''
cleanup() {
  result=$?
  trap - EXIT
  if (( result != 0 )) && [[ -n "$box" ]]; then
    omabox log -b "$box" run -n 100 > "$output/failure.log" 2>&1 || true
    omabox windows -b "$box" --json > "$output/failure-windows.json" 2>&1 || true
  fi
  local teardown=true
  for own_box in "${boxes[@]}"; do
    if ! omabox down "$own_box" >&2; then teardown=false; result=1; fi
  done
  jq -n --arg binary "$binary" --arg artifacts "$output" --arg stage "$stage" \
    --argjson success "$([[ "$result" == 0 ]] && echo true || echo false)" --argjson teardown "$teardown" \
    '{environment:"isolated-omabox", binary:$binary, artifacts:$artifacts, success:$success,
      stage:$stage, ownBoxesTornDown:$teardown, hostSetupApplied:false, guestActionsRun:false,
      checked:(if $success then ["stock bar registration", "dbusmenu labels and disabled start", "close keeps tray process", "activate reopens window", "menu quit exits", "no host close exits", "second launch exits zero with one window and process"] else [] end)}' \
    | tee "$output/result.json"
  exit "$result"
}
trap cleanup EXIT
inside() { omabox run -b "$box" -- "$@"; }
bus() { inside busctl --user --json=short "$@"; }
start() {
  omabox run -b "$box" -d -- sh -c 'exec "$HOME/.local/bin/ws"'
  omabox wait -b "$box" --timeout 30s window class:window-side >/dev/null
}
state() {
  local label=$1
  omabox windows -b "$box" --json > "$output/$label-windows.json"
  omabox hyprctl -b "$box" -j clients > "$output/$label-clients.json"
  local count
  count=$(inside sh -c 'pgrep -xc ws || true')
  jq -n --argjson processes "$count" --slurpfile clients "$output/$label-clients.json" \
    '{processes:$processes, windows:([$clients[0][] | select(.class == "window-side")] | length)}' > "$output/$label-state.json"
}
shot() { omabox shot -b "$box" --window class:window-side -o "$output/$1.png" >/dev/null; }
wait_exit() { omabox wait -b "$box" --timeout 15s cmd -- sh -c '! pgrep -x ws >/dev/null' >/dev/null; }

stage=stock-bar-start
box="window-side-tray-bar-$$"
boxes+=("$box")
omabox up "$box" --stock-bar --net isolated --size 1280x900@60 --seed "$binary:.local/bin/ws" --json > "$output/stock-bar-box.json"
start
omabox wait -b "$box" --timeout 20s cmd -- sh -c 'busctl --user get-property org.kde.StatusNotifierWatcher /StatusNotifierWatcher org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems | grep -q /StatusNotifierItem' >/dev/null
stage=stock-bar-menu
bus get-property org.kde.StatusNotifierWatcher /StatusNotifierWatcher org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems > "$output/registered-items.json"
item=$(jq -er '[.. | strings | select(contains("/StatusNotifierItem"))] | if length == 1 then .[0] else error("Expected exactly one tray item") end' "$output/registered-items.json")
service=${item%%/*}
item_path="/${item#*/}"
bus get-property "$service" "$item_path" org.kde.StatusNotifierItem Id > "$output/item-id.json"
jq -e '[.. | strings] | index("window-side") != null' "$output/item-id.json" >/dev/null
bus get-property "$service" "$item_path" org.kde.StatusNotifierItem Menu > "$output/menu-path.json"
menu=$(jq -er '[.. | strings | select(startswith("/"))] | .[0]' "$output/menu-path.json")
bus call "$service" "$menu" com.canonical.dbusmenu GetLayout iias -- 0 -1 0 > "$output/menu-layout.json"
jq '[.. | arrays | select(length == 3 and (.[0] | type == "number") and (.[1] | type == "object")) |
    {id:.[0], properties:.[1]}] | map(select(.properties | has("label"))) |
    def value: if type == "object" and has("data") then .data else . end;
    map({id, label:(.properties.label | value), enabled:(if .properties | has("enabled") then .properties.enabled | value else true end)})' \
  "$output/menu-layout.json" > "$output/menu-items.json"
jq -e '([.[].label] == ["Windows · Unavailable", "Start Windows", "Open window-side", "Quit"])
  and (map(select(.label == "Start Windows")) | length == 1 and .[0].enabled == false)' "$output/menu-items.json" >/dev/null
quit_id=$(jq -er '.[] | select(.label == "Quit") | .id' "$output/menu-items.json")
state stock-bar-open
jq -e '.processes == 1 and .windows == 1' "$output/stock-bar-open-state.json" >/dev/null
shot stock-bar-open
stage=stock-bar-close
omabox keys -b "$box" --window class:window-side super+w
omabox wait -b "$box" --timeout 15s window class:window-side --gone >/dev/null
state stock-bar-closed
jq -e '.processes == 1 and .windows == 0' "$output/stock-bar-closed-state.json" >/dev/null
stage=stock-bar-activate
bus call "$service" "$item_path" org.kde.StatusNotifierItem Activate ii 0 0 > "$output/activate.txt"
omabox wait -b "$box" --timeout 15s window class:window-side >/dev/null
state stock-bar-reopened
jq -e '.processes == 1 and .windows == 1' "$output/stock-bar-reopened-state.json" >/dev/null
shot stock-bar-reopened
stage=stock-bar-quit
bus call "$service" "$menu" com.canonical.dbusmenu Event isvu "$quit_id" clicked s '' 0 > "$output/quit.txt"
wait_exit
state stock-bar-quit
jq -e '.processes == 0 and .windows == 0' "$output/stock-bar-quit-state.json" >/dev/null

stage=no-host-start
box="window-side-tray-none-$$"
boxes+=("$box")
omabox up "$box" --no-shell --net isolated --size 1280x900@60 --seed "$binary:.local/bin/ws" --json > "$output/no-host-box.json"
# No shell means no StatusNotifierWatcher, which is explicitly checked inside the box.
inside busctl --user --json=short list > "$output/no-host-bus.json"
jq -e '[.. | strings] | index("org.kde.StatusNotifierWatcher") == null' "$output/no-host-bus.json" >/dev/null
start
state no-host-open
jq -e '.processes == 1 and .windows == 1' "$output/no-host-open-state.json" >/dev/null
stage=no-host-close
omabox keys -b "$box" --window class:window-side super+w
wait_exit
state no-host-closed
jq -e '.processes == 0 and .windows == 0' "$output/no-host-closed-state.json" >/dev/null
stage=no-host-single-instance
start
inside sh -c 'timeout 15s "$HOME/.local/bin/ws"; result=$?; printf "%s\n" "$result"; exit "$result"' > "$output/second-launch-exit.txt"
[[ "$(cat "$output/second-launch-exit.txt")" == 0 ]]
state no-host-second-launch
jq -e '.processes == 1 and .windows == 1' "$output/no-host-second-launch-state.json" >/dev/null
shot no-host-second-launch
omabox keys -b "$box" --window class:window-side super+w
wait_exit
stage=complete
