#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd "$root"
case "${1:-}" in
  --help|-h)
    echo 'Usage: ./ops dev'
    echo 'Build with two jobs, then replace this checkout’s previous dev app.'
    echo 'Re-run after edits; Rust does not hot-reload. Build failures keep the old app running.'
    exit 0 ;;
  '') ;;
  *) echo 'Use ./ops dev [--help]' >&2; exit 2 ;;
esac
if (( $# > 1 )); then
  echo 'Use ./ops dev [--help]' >&2
  exit 2
fi
mkdir -p "$root/target"
exec 9>"$root/target/dev.lock"
flock 9
# Use a predictable checkout-local binary even when the shell sets a target directory.
pnpm -C ui build
cargo build --locked --jobs 2 --bin ws --target-dir "$root/target"
binary="$root/target/debug/ws"
pid_file="$root/target/dev.pid"
matches() {
  local executable
  [[ "$dev_pid" =~ ^[0-9]+$ && -O "/proc/$dev_pid" ]] || return 1
  [[ $(readlink "/proc/$dev_pid/cwd") == "$root" ]] || return 1
  executable=$(readlink "/proc/$dev_pid/exe") || return 1
  [[ ${executable% (deleted)} == "$binary" ]] || return 1
  local stat
  stat=$(cat "/proc/$dev_pid/stat") || return 1
  stat=${stat##*) }
  [[ $(awk '{print $20}' <<<"$stat") == "$dev_start" ]]
}
if [[ -f "$pid_file" ]]; then
  dev_pid=''
  dev_start=''
  read -r dev_pid dev_start < "$pid_file" || true
  if matches; then
    kill -TERM "$dev_pid"
    for ((attempt=0; attempt<120; attempt++)); do
      matches || break
      sleep 0.025
    done
    if matches; then
      echo "Dev PID $dev_pid did not exit; replacement was not launched." >&2
      exit 1
    fi
  fi
fi
"$binary" 9>&- &
dev_pid=$!
stat=$(cat "/proc/$dev_pid/stat")
stat=${stat##*) }
dev_start=$(awk '{print $20}' <<<"$stat")
printf '%s %s\n' "$dev_pid" "$dev_start" > "$pid_file"
flock -u 9
exec 9>&-
trap 'kill -TERM "$dev_pid" 2>/dev/null || true' INT TERM
echo "Dev started: PID $dev_pid" >&2
wait "$dev_pid"
