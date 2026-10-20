#!/usr/bin/env bash
set -euo pipefail

root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd -P)
if (( $# != 1 )) || [[ "$1" != /* || ( -e "$1" && ! -d "$1" ) ]]; then
  echo 'Usage: site/scripts/record-native.sh /absolute/output-directory' >&2
  exit 2
fi
output=$1
raw=/tmp/ws-landing/native
binary="$root/target/debug/ws"
main_box="ws-landing-main-$$"
tray_box="ws-landing-tray-$$"
boxes=()
mkdir -m 700 -p -- "$output" "$raw"

cleanup() {
  local code=$?
  trap - EXIT
  for box in "${boxes[@]}"; do
    omabox down "$box" >&2 || code=1
  done
  exit "$code"
}
trap cleanup EXIT

capture_still() {
  local name=$1 source=$2
  magick "$source" -strip -resize '1600x1600>' -quality 78 "$output/$name.webp"
  magick "$source" -strip -resize '1600x1600>' -quality 48 "$output/$name.avif"
}

float_window() {
  local box=$1 width=$2 height=$3
  omabox hyprctl -b "$box" dispatch 'hl.dsp.window.float()' >/dev/null
  omabox hyprctl -b "$box" dispatch "hl.dsp.window.resize({x=$width,y=$height})" >/dev/null
  omabox hyprctl -b "$box" dispatch 'hl.dsp.window.center()' >/dev/null
}

"$root/ops" build

boxes+=("$main_box")
omabox up "$main_box" --stock-bar --net isolated --size 1600x1000@60 \
  --seed "$binary:.local/bin/ws" --json > "$raw/main-box.json"
omabox run -b "$main_box" --env WS_GUI_FIXTURE=setup -d -- sh -c 'exec "$HOME/.local/bin/ws"'
omabox wait -b "$main_box" --timeout 30s window class:ws >/dev/null
float_window "$main_box" 1120 780
sleep 3
omabox shot -b "$main_box" -o "$raw/native-welcome.png" >/dev/null
omabox shot -b "$main_box" --window class:ws -o "$raw/native-welcome-window.png" >/dev/null
magick "$raw/native-welcome.png" -strip "$raw/native-welcome-full.png"
magick "$raw/native-welcome-window.png" -strip "$raw/native-welcome-tight.png"

omabox run -b "$main_box" --env WS_GUI_FIXTURE=status -d --replace -- sh -c 'exec "$HOME/.local/bin/ws"'
omabox wait -b "$main_box" --timeout 30s window class:ws >/dev/null
float_window "$main_box" 1160 820
sleep 3
omabox shot -b "$main_box" -o "$raw/native-overview.png" >/dev/null
omabox shot -b "$main_box" --window class:ws -o "$raw/native-overview-window.png" >/dev/null
magick "$raw/native-overview-window.png" -strip "$raw/native-overview-tight.png"

omabox run -b "$main_box" -- sh -c '"$HOME/.local/bin/ws" setup plugin' >/dev/null 2>&1 || true
omabox run -b "$main_box" -- omarchy-shell shell rescanPlugins >/dev/null
omabox run -b "$main_box" -- omarchy plugin enable noel.window-side >/dev/null
omabox restart-shell -b "$main_box" >/dev/null
omabox run -b "$main_box" -- sh -c 'mv "$HOME/.local/bin/ws" "$HOME/.local/bin/ws.real"; cat > "$HOME/.local/bin/ws" <<'"'"'EOF'"'"'
#!/bin/sh
if [ "${1-}" = metrics ]; then
  printf "%s\\n" '\''{"status":"running","bootProfile":"gpu","backend":"libvirt","available":true,"source":"windows-guest-agent","cpu":18,"memory":"10.2 / 16 GiB","memoryPercent":64,"network":"↓ 1.4 MiB/s\n↑ 218 KiB/s","pids":142,"uptimeSeconds":3349,"sampledAt":1792523760,"storage":{"status":"ready","error":""}}'\''
else
  exec "$HOME/.local/bin/ws.real" "$@"
fi
EOF
chmod +x "$HOME/.local/bin/ws"'
omabox run -b "$main_box" -- omarchy-shell shell summon noel.window-side >/dev/null
sleep 2
omabox shot -b "$main_box" -o "$raw/panel.png" >/dev/null
omabox shot -b "$main_box" -g '0,0 370x390' -o "$raw/panel-tight.png" >/dev/null
magick "$raw/panel.png" -strip "$raw/panel-full.png"
magick "$raw/panel-tight.png" -crop 360x350+5+31 +repage "$raw/panel-tight-crop.png"

omabox run -b "$main_box" -- omarchy-shell shell hide noel.window-side >/dev/null
omabox pointer -b "$main_box" -- move 780 970
omabox run -b "$main_box" -- sh -c 'setsid wf-recorder -y -f "$HOME/overview.mkv" >"$HOME/wf-recorder.log" 2>&1 < /dev/null &'
sleep 7
omabox run -b "$main_box" -- sh -c 'pkill -INT -x wf-recorder || true'
omabox wait -b "$main_box" --timeout 15s cmd -- sh -c '! pgrep -x wf-recorder' >/dev/null
cp -- "$(omabox path -b "$main_box")/home/overview.mkv" "$raw/overview-clip.mkv"

boxes+=("$tray_box")
omabox up "$tray_box" --stock-bar --net isolated --size 1600x1000@60 \
  --seed "$binary:.local/bin/ws" --json > "$raw/tray-box.json"
omabox run -b "$tray_box" -d -- sh -c 'exec "$HOME/.local/bin/ws"'
omabox wait -b "$tray_box" --timeout 30s window class:ws >/dev/null
omabox wait -b "$tray_box" --timeout 20s cmd -- sh -c 'busctl --user --json=short get-property org.kde.StatusNotifierWatcher /StatusNotifierWatcher org.kde.StatusNotifierWatcher RegisteredStatusNotifierItems | grep -q StatusNotifierItem' >/dev/null
sleep 3
omabox keys -b "$tray_box" --window class:ws super+w
omabox wait -b "$tray_box" --timeout 15s window class:ws --gone >/dev/null
omabox pointer -b "$tray_box" -- move 1471 12 sleep 800 move 1440 12 sleep 100
omabox shot -b "$tray_box" -o "$raw/tray.png" >/dev/null
omabox pointer -b "$tray_box" -- move 1510 12 click right
omabox pointer -b "$tray_box" -- move 900 700
sleep 1
omabox shot -b "$tray_box" -o "$raw/tray-menu.png" >/dev/null
omabox shot -b "$tray_box" -g '1360,0 250x190' -o "$raw/tray-menu-tight.png" >/dev/null

for name in native-welcome native-welcome-window native-overview native-overview-window panel panel-tight tray-icon tray-menu tray-menu-tight; do
  case "$name" in
    native-welcome) source="$raw/native-welcome-full.png" ;;
    native-welcome-window) source="$raw/native-welcome-tight.png" ;;
    native-overview) source="$raw/native-overview.png" ;;
    native-overview-window) source="$raw/native-overview-tight.png" ;;
    panel) source="$raw/panel-full.png" ;;
    panel-tight) source="$raw/panel-tight-crop.png" ;;
    tray-icon) source="$raw/tray.png" ;;
    tray-menu) source="$raw/tray-menu.png" ;;
    tray-menu-tight) source="$raw/tray-menu-tight.png" ;;
  esac
  capture_still "$name" "$source"
done

ffmpeg -hide_banner -loglevel error -y -i "$raw/overview-clip.mkv" \
  -vf fps=30 -c:v libsvtav1 -crf 44 -preset 5 -pix_fmt yuv420p -an "$output/overview-clip.webm"
ffmpeg -hide_banner -loglevel error -y -i "$raw/overview-clip.mkv" \
  -vf fps=30 -c:v libx264 -crf 26 -preset slow -pix_fmt yuv420p -movflags +faststart -an "$output/overview-clip.mp4"
ffmpeg -hide_banner -loglevel error -y -ss 3 -i "$raw/overview-clip.mkv" -frames:v 1 "$raw/overview-clip-poster.png"
capture_still overview-clip "$raw/overview-clip-poster.png"

echo "Captured native media to $output"
