#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
case "${1:-}" in
  '') exec pnpm -C "$root/ui" dev --port 5199 --strictPort ;;
  --window)
    echo 'Run the native dev window only inside an isolated OmaBox.' >&2
    [[ -n "${OMABOX_BOX:-}" ]] || { echo 'Use omabox run -b NAME -- ./ops ui:dev --window' >&2; exit 1; }
    cd "$root"
    exec cargo run --no-default-features --features tauri/devtools --bin ws ;;
  *) echo 'Usage: ./ops ui:dev [--window] (window requires OmaBox)' >&2; exit 2 ;;
esac
