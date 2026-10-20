#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
export WS_BOOTSTRAP_FIXTURE=$fixture
name=window-side-0.1.0-linux-x86_64
mkdir -p "$fixture/bin" "$fixture/package/$name"
cat > "$fixture/package/$name/install.sh" <<'INSTALL'
#!/usr/bin/env bash
printf '%s\n' "$@" > "$WS_BOOTSTRAP_FIXTURE/arguments"
exit "${WS_BOOTSTRAP_EXIT:-0}"
INSTALL
touch "$fixture/package/$name/ws"
tar -czf "$fixture/$name.tar.gz" -C "$fixture/package" "$name"
(cd "$fixture"; sha256sum "$name.tar.gz" > checksum)
cat > "$fixture/bin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
url=''
output=''
while (( $# )); do
  case "$1" in
    -o) output=$2; shift 2 ;;
    --retry) shift 2 ;;
    https://*) url=$1; shift ;;
    *) shift ;;
  esac
done
[[ "$url" == https://github.com/Noelithub77/window-side/releases/download/v0.1.0/* ]] || exit 1
[[ "${WS_BOOTSTRAP_DOWNLOAD_FAIL:-0}" == 0 ]] || exit 22
case "$url" in
  *.sha256) cp "$WS_BOOTSTRAP_FIXTURE/checksum" "$output" ;;
  *.tar.gz) cp "$WS_BOOTSTRAP_FIXTURE/window-side-0.1.0-linux-x86_64.tar.gz" "$output" ;;
  *) exit 1 ;;
esac
CURL
chmod +x "$fixture/bin/curl"
export PATH="$fixture/bin:$PATH"
bash "$root/get.sh" --prefix "$fixture/prefix with spaces" --no-plugin --dry-run
grep -Fxq -- "$fixture/prefix with spaces" "$fixture/arguments"
grep -Fxq -- --no-plugin "$fixture/arguments"
grep -Fxq -- --dry-run "$fixture/arguments"
if WS_BOOTSTRAP_EXIT=7 bash "$root/get.sh"; then echo 'Ignored installer failure' >&2; exit 1; fi
rm "$fixture/arguments"
if WS_BOOTSTRAP_DOWNLOAD_FAIL=1 bash "$root/get.sh"; then echo 'Ignored download failure' >&2; exit 1; fi
[[ ! -e "$fixture/arguments" ]]
printf 'corruption' >> "$fixture/$name.tar.gz"
if bash "$root/get.sh"; then echo 'Installed a corrupt archive' >&2; exit 1; fi
[[ ! -e "$fixture/arguments" ]]
printf 'invalid checksum\n' > "$fixture/checksum"
if bash "$root/get.sh"; then echo 'Accepted malformed checksum' >&2; exit 1; fi
[[ ! -e "$fixture/arguments" ]]
echo 'Bootstrap fixtures passed: forwarding, installer errors, download errors and checksum rejection'
