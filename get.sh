#!/usr/bin/env bash
# The default version advances only after a release is ready for installation.
set -euo pipefail
version=0.1.0
case "${1:-}" in
  --help|-h)
    echo 'Usage: bash get.sh [installer options]'
    echo 'Download and verify the current Linux x86_64 release, then run its installer.'
    echo 'Options are forwarded to install.sh, including --dry-run, --prefix and --no-plugin.'
    exit 0 ;;
esac
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || {
  echo 'window-side currently supports Linux x86_64 only.' >&2
  exit 1
}
for tool in curl tar sha256sum mktemp; do
  command -v "$tool" >/dev/null || { echo "Install $tool before continuing." >&2; exit 1; }
done
temporary=$(mktemp -d)
trap 'rm -rf -- "$temporary"' EXIT
name="window-side-$version-linux-x86_64"
url="https://github.com/Noelithub77/window-side/releases/download/v$version"
echo "Downloading window-side $version…" >&2
curl --fail --silent --show-error --location --retry 3 "$url/$name.tar.gz" -o "$temporary/$name.tar.gz"
curl --fail --silent --show-error --location --retry 3 "$url/$name.tar.gz.sha256" -o "$temporary/checksum"
read -r digest filename < "$temporary/checksum"
[[ "$digest" =~ ^[a-f0-9]{64}$ && "$filename" == "$name.tar.gz" ]] || {
  echo 'Invalid release checksum file.' >&2
  exit 1
}
actual=$(sha256sum "$temporary/$name.tar.gz")
[[ ${actual%% *} == "$digest" ]] || { echo 'Release checksum verification failed.' >&2; exit 1; }
tar -xzf "$temporary/$name.tar.gz" -C "$temporary"
bash "$temporary/$name/install.sh" --binary "$temporary/$name/ws" "$@"
