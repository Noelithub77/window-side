#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
output=''
allow_dirty=false
while (( $# )); do
  case "$1" in
    --output) (( $# >= 2 )) || exit 2; output=$2; shift 2 ;;
    --allow-dirty) allow_dirty=true; shift ;;
    *) echo 'Usage: ./ops release:package --output /absolute/directory [--allow-dirty]' >&2; exit 2 ;;
  esac
done
[[ "$output" == /* && "$output" != *[$'\n\r\t']* ]] || { echo 'An absolute output directory is required' >&2; exit 2; }
[[ "$(uname -m)" == x86_64 ]] || { echo 'Only Linux x86_64 release artifacts are supported' >&2; exit 1; }
binary="$root/target/release/ws"
[[ -x "$binary" ]] || { echo 'Build the release binary first' >&2; exit 1; }
version=$("$binary" --version); version=${version#ws }
[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][[:alnum:].-]+)?$ ]] || exit 1
revision=$(git -C "$root" rev-parse HEAD)
dirty=false
if [[ -n "$(git -C "$root" status --porcelain)" ]]; then
  dirty=true
  $allow_dirty || { echo 'Commit the source before packaging, or explicitly use --allow-dirty for a local preview' >&2; exit 1; }
fi
name="window-side-$version-linux-x86_64"
mkdir -p -- "$output"
[[ ! -e "$output/$name.tar.gz" && ! -e "$output/$name.tar.gz.sha256" ]] || { echo 'Release output already exists' >&2; exit 1; }
stage=$(mktemp -d)
trap 'rm -rf -- "$stage"' EXIT
mkdir -p "$stage/$name/packaging" "$stage/$name/scripts"
install -m 755 "$binary" "$stage/$name/ws"
install -m 755 "$root/install.sh" "$stage/$name/install.sh"
install -m 755 "$root/scripts/system-deps.sh" "$stage/$name/scripts/system-deps.sh"
install -m 644 "$root/packaging/window-side.desktop" "$stage/$name/packaging/window-side.desktop"
install -m 644 "$root/LICENSE" "$root/README.md" "$stage/$name/"
printf 'version=%s\nrevision=%s\ndirty=%s\nbuildPlatform=Arch Linux x86_64\n' "$version" "$revision" "$dirty" > "$stage/$name/build-info.txt"
epoch=$(git -C "$root" show -s --format=%ct HEAD)
tar --sort=name --mtime="@$epoch" --owner=0 --group=0 --numeric-owner -C "$stage" -cf - "$name" | gzip -n > "$output/$name.tar.gz"
(cd "$output"; sha256sum "$name.tar.gz" > "$name.tar.gz.sha256")
printf 'Packaged %s (%s, dirty=%s)\n' "$output/$name.tar.gz" "$revision" "$dirty"
