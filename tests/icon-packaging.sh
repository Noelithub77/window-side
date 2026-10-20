#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
source_root="$fixture/source"
mkdir -p "$source_root/target/release" "$source_root/scripts" "$source_root/packaging" "$fixture/bin"
cp "$root/install.sh" "$root/LICENSE" "$root/README.md" "$source_root/"
cp "$root/scripts/package-release.sh" "$root/scripts/system-deps.sh" "$source_root/scripts/"
cp "$root/packaging/window-side.desktop" "$source_root/packaging/"
cp -R "$root/icons" "$source_root/"
printf '#!/usr/bin/env bash\nprintf "ws 0.1.0\\n"\n' > "$source_root/target/release/ws"
chmod +x "$source_root/target/release/ws"
git -C "$source_root" init -q
git -C "$source_root" add .
git -C "$source_root" -c user.name=Fixture -c user.email=fixture@example.invalid commit -qm 'Packaging fixture'
bash "$source_root/scripts/package-release.sh" --output "$fixture/release"
tar -xzf "$fixture/release/window-side-0.1.0-linux-x86_64.tar.gz" -C "$fixture"
diff -qr "$root/icons/hicolor" "$fixture/window-side-0.1.0-linux-x86_64/icons/hicolor"
# Run just the package function, without building or installing system packages.
cat > "$fixture/bin/gtk-update-icon-cache" <<'CACHE'
#!/usr/bin/env bash
printf '%s\n' "$*" > "$WS_ICON_CACHE_LOG"
exit 1
CACHE
chmod +x "$fixture/bin/gtk-update-icon-cache"
export PATH="$fixture/bin:$PATH" WS_ICON_CACHE_LOG="$fixture/cache.log"
(
  source "$root/packaging/PKGBUILD.in"
  pkgdir="$fixture/package"
  mkdir -p "$fixture/aur"
  ln -s "$source_root" "$fixture/aur/window-side-$pkgver"
  cd "$fixture/aur"
  package
)
diff -qr "$root/icons/hicolor" "$fixture/package/usr/share/icons/hicolor"
grep -Fxq -- "-f -t $fixture/package/usr/share/icons/hicolor" "$fixture/cache.log"
echo 'Icon packaging fixtures passed: release tree, package tree and optional cache failure'
