#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
[[ $# == 4 && "$1" == --binary && "$2" == /* && "$3" == --output ]] || { echo 'Usage: ./ops gui:installed --binary /absolute/ws --output /absolute/new-directory' >&2; exit 2; }
export WS_GUI_BINARY=$2
exec bash "$root/scripts/gui-smoke.sh" --output "$4"
