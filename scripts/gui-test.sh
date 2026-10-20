#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
"$root/ops" ui:test
exec bash "$root/scripts/gui-smoke.sh" "$@"
