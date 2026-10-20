#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
if [[ "${1:-}" == --help || "${1:-}" == -h || "${2:-}" == --help || "${2:-}" == -h ]]; then
  cat <<'HELP'
Usage: ./ops icons | ./ops icons:check

Generate tracked application and tray PNGs from icons/source/window-side.svg.
icons:check compares a temporary render and does not change tracked files.
Requires the development tools rsvg-convert (librsvg) and magick (ImageMagick).
HELP
  exit 0
fi
mode=${1:-generate}
[[ $# -le 1 && ( "$mode" == generate || "$mode" == check ) ]] || { echo 'Usage: ./ops icons[:check]' >&2; exit 2; }
command -v rsvg-convert >/dev/null || { echo 'Icon generation/checking needs the dev tool rsvg-convert (librsvg)' >&2; exit 1; }
command -v magick >/dev/null || { echo 'Icon generation/checking needs the dev tool magick (ImageMagick)' >&2; exit 1; }
stage=$(mktemp -d)
trap 'rm -rf -- "$stage"' EXIT
for size in 16 22 24 32 48 64 128 256 512; do
  directory="$stage/hicolor/${size}x${size}/apps"
  mkdir -p "$directory"
  # Tauri requires RGBA even when the artwork is fully opaque.
  rsvg-convert --width "$size" --height "$size" "$root/icons/source/window-side.svg" |
    magick png:- -strip "PNG32:$directory/window-side.png"
done
mkdir -p "$stage/hicolor/scalable/apps"
cp "$root/icons/source/window-side.svg" "$stage/hicolor/scalable/apps/window-side.svg"
cp "$stage/hicolor/512x512/apps/window-side.png" "$stage/icon.png"
if [[ "$mode" == check ]]; then
  diff -qr "$stage/hicolor" "$root/icons/hicolor"
  cmp "$stage/icon.png" "$root/icons/icon.png"
  echo 'Generated icons match the SVG sources'
else
  cp -R "$stage/." "$root/icons/"
  echo 'Regenerated application and tray icons'
fi
