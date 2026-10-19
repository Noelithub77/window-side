#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
archive=''; output=''
while (( $# )); do
  case "$1" in
    --source-archive|--output)
      (( $# >= 2 )) || exit 2
      if [[ "$1" == --source-archive ]]; then archive=$2; else output=$2; fi
      shift 2 ;;
    *) echo 'Usage: ./ops aur:prepare --source-archive /reviewed/tag.tar.gz --output /new/directory' >&2; exit 2 ;;
  esac
done
[[ -f "$archive" && "$output" == /* && ! -e "$output" ]] || { echo 'Provide an existing tag source archive and a new absolute output directory' >&2; exit 2; }
version=$(sed -n 's/^pkgver=//p' "$root/packaging/PKGBUILD.in")
listing=$(tar -tzf "$archive")
while IFS= read -r member; do
  [[ "$member" == "window-side-$version/"* ]] || { echo 'Archive does not match the expected versioned source root' >&2; exit 1; }
done <<< "$listing"
checksum=$(sha256sum -- "$archive"); checksum=${checksum%% *}
mkdir -m 700 -p -- "$output"
sed "s/@SHA256@/$checksum/" "$root/packaging/PKGBUILD.in" > "$output/PKGBUILD"
(cd "$output"; makepkg --printsrcinfo > .SRCINFO)
printf 'Prepared PKGBUILD and .SRCINFO with source sha256=%s; nothing was published\n' "$checksum"
