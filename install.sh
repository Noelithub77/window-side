#!/usr/bin/env bash
# Installs or updates the app and VM packages. Machine configuration stays in onboarding.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
prefix="${HOME:?}/.local"
binary=''
replace=false
dry_run=false
yes=false
no_plugin=false
while (( $# )); do
  case "$1" in
    --prefix|--binary)
      (( $# >= 2 )) || { echo "Missing value for $1" >&2; exit 2; }
      if [[ "$1" == --prefix ]]; then prefix=$2; else binary=$2; fi
      shift 2 ;;
    --replace-existing) replace=true; shift ;;
    --dry-run) dry_run=true; shift ;;
    --yes) yes=true; shift ;;
    --no-plugin) no_plugin=true; shift ;;
    --help|-h)
      cat <<'HELP'
Usage: ./install.sh [--binary FILE] [--prefix /absolute/path] [--replace-existing]
                    [--yes] [--no-plugin] [--dry-run]

Builds the locked Rust release when --binary is omitted, then installs or updates
ws, its desktop entry and the VM packages for Arch, Ubuntu/Debian or Fedora.
Package installation uses the installed ws through pkexec or sudo. On Omarchy,
asks whether to install or update the optional panel; declined/skipped panels
are offered again next time. Machine configuration stays in ws setup.

--yes accepts the panel without prompting. --no-plugin skips it even with --yes.
Without a terminal, the panel is skipped unless --yes is supplied. --dry-run
writes nothing, does not build, and only queries packages when a binary exists.
Unknown existing files require --replace-existing; backups preserve symlink
sources. Results are JSON; progress goes to stderr. Does not launch the app.

HELP
      exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ "$prefix" == /* && "$prefix" != / && "$prefix" != *[$'\n\r\t']* ]] || { echo 'Prefix must be an absolute directory without control characters' >&2; exit 2; }
prefix=${prefix%/}
[[ ! -L "$prefix/bin" && ! -L "$prefix/share/applications" ]] || { echo 'Installation directories must not be symlinks' >&2; exit 1; }
if [[ -z "$binary" && -x "$root/ws" ]]; then binary="$root/ws"; fi
if [[ -z "$binary" ]]; then
  binary="$root/target/release/ws"
  if ! $dry_run; then
    command -v cargo >/dev/null || { echo 'Install Rust and the documented build tools first, or pass --binary' >&2; exit 1; }
    command -v node >/dev/null && command -v pnpm >/dev/null || { echo 'Source builds require Node.js and pnpm. See scripts/system-deps.sh --build' >&2; exit 1; }
    if ! pkg-config --exists webkit2gtk-4.1 gtk+-3.0; then
      echo 'Missing build dependencies. Install with:' >&2
      bash "$root/scripts/system-deps.sh" --build >&2
      exit 1
    fi
    (cd "$root"; pnpm -C ui install --frozen-lockfile; pnpm -C ui build; cargo build --release --locked -j 2) >&2
  fi
fi
[[ "$binary" != *[$'\n\r\t']* ]] || { echo 'Binary path must not contain control characters' >&2; exit 2; }
if [[ -e "$binary" ]]; then
  binary=$(realpath -- "$binary")
  [[ -f "$binary" && -x "$binary" ]] || { echo 'The binary must be an executable regular file' >&2; exit 1; }
  if ldd "$binary" 2>/dev/null | grep -q 'not found'; then
    if $dry_run; then
      echo 'Missing runtime libraries. Install with:' >&2
      bash "$root/scripts/system-deps.sh" --runtime >&2
      exit 1
    fi
    bash "$root/scripts/system-deps.sh" --runtime --install >&2
  fi
  version=$("$binary" --version)
  [[ "$version" =~ ^ws\ [0-9]+\.[0-9]+\.[0-9]+([-+][[:alnum:].-]+)?$ ]] || { echo 'The supplied executable is not a native ws build' >&2; exit 1; }
  digest=$(sha256sum -- "$binary"); digest=${digest%% *}
else
  $dry_run || { echo "Binary not found: $binary" >&2; exit 1; }
  version='not built'
  digest=''
fi
receipt="$prefix/share/window-side/install-receipt"
owned=false
if [[ -f "$receipt" && ! -L "$receipt" && -f "$prefix/bin/ws" && ! -L "$prefix/bin/ws" ]]; then
  previous=$(sha256sum -- "$prefix/bin/ws"); previous=${previous%% *}
  if grep -Fxq "sha256=$previous" "$receipt"; then owned=true; fi
fi
files=(bin/ws share/applications/window-side.desktop share/licenses/window-side/LICENSE share/window-side/install-receipt)
for relative in "${files[@]}"; do
  target="$prefix/$relative"
  if [[ -d "$target" && ! -L "$target" ]]; then echo "Refusing to replace a directory: $target" >&2; exit 1; fi
  if [[ -e "$target" || -L "$target" ]] && ! $owned && ! $replace; then
    echo "Existing unmanaged file: $target. Review it, then use --replace-existing to back it up and replace it." >&2
    exit 1
  fi
done
json_string() {
  local value=$1
  value=${value//\\/\\\\}; value=${value//\"/\\\"}
  printf '"%s"' "$value"
}
backup=''
if ! $dry_run; then
  backup_candidate="${XDG_STATE_HOME:-$HOME/.local/state}/window-side/install-backups/$(date -u +%Y%m%dT%H%M%SZ)-$$"
  for relative in "${files[@]}"; do
    target="$prefix/$relative"
    if [[ -e "$target" || -L "$target" ]]; then
      backup=$backup_candidate
      mkdir -p -m 700 -- "$backup"
      mkdir -p -- "$backup/$(dirname -- "$relative")"
      cp -P -- "$target" "$backup/$relative"
      # Keep the old launcher bytes as well as its link, in case the source moves.
      if [[ "$relative" == bin/ws && -L "$target" && -f "$target" ]]; then cp -L -- "$target" "$backup/ws-before"; fi
    fi
  done
  mkdir -p -- "$prefix/bin" "$prefix/share/applications" "$prefix/share/licenses/window-side" "$prefix/share/window-side"
  temporary=$(mktemp "$prefix/bin/.ws.XXXXXXXX")
  trap 'rm -f -- "${temporary:-}" "${desktop_temporary:-}" "${license_temporary:-}" "${receipt_temporary:-}"' EXIT
  install -m 755 -- "$binary" "$temporary"
  desktop_temporary=$(mktemp --suffix=.desktop "$prefix/share/applications/.window-side.XXXXXXXX")
  executable="$prefix/bin/ws"
  escaped=$(printf '%s' "$executable" | sed 's/[\\"`$]/\\&/g; s/%/%%/g')
  while IFS= read -r line; do
    case "$line" in
      Exec=ws*) printf 'Exec="%s"%s\n' "$escaped" "${line#Exec=ws}" ;;
      *) printf '%s\n' "$line" ;;
    esac
  done < "$root/packaging/window-side.desktop" > "$desktop_temporary"
  chmod 644 "$desktop_temporary"
  if command -v desktop-file-validate >/dev/null; then desktop-file-validate "$desktop_temporary"; fi
  mv -Tf -- "$temporary" "$prefix/bin/ws"
  mv -Tf -- "$desktop_temporary" "$prefix/share/applications/window-side.desktop"
  # Replace license symlinks instead of writing through them.
  license_temporary=$(mktemp "$prefix/share/licenses/window-side/.license.XXXXXXXX")
  install -m 644 -- "$root/LICENSE" "$license_temporary"
  mv -Tf -- "$license_temporary" "$prefix/share/licenses/window-side/LICENSE"
  receipt_temporary=$(mktemp "$prefix/share/window-side/.receipt.XXXXXXXX")
  printf 'version=%s\nsha256=%s\n' "$version" "$digest" > "$receipt_temporary"
  mv -Tf -- "$receipt_temporary" "$receipt"
  if command -v update-desktop-database >/dev/null; then
    update-desktop-database "$prefix/share/applications" >&2 || echo 'Installed; refreshing the optional desktop cache failed' >&2
  fi
fi
packages=null
plugin='{"status":"not-applicable"}'
if $dry_run; then
  if [[ -x "$binary" ]]; then
    packages=$("$binary" setup packages) || { echo 'Package preview failed' >&2; exit 1; }
  else
    echo 'Environment and package preview require a built ws binary; nothing was changed' >&2
  fi
  if command -v omarchy >/dev/null; then plugin='{"status":"would-ask"}'; fi
  if $no_plugin; then plugin='{"status":"skipped"}'; fi
else
  packages=$("$prefix/bin/ws" setup packages --install) || {
    echo 'ws was installed, but VM package setup failed. Rerun this command to retry.' >&2
    exit 1
  }
  if command -v omarchy >/dev/null; then
    choice_file="${XDG_STATE_HOME:-$HOME/.local/state}/window-side/install-plugin-choice"
    previous_choice=''
    if [[ -f "$choice_file" && ! -L "$choice_file" ]]; then read -r previous_choice < "$choice_file" || true; fi
    install_plugin=$yes
    choice=skipped
    if $no_plugin; then
      install_plugin=false
    elif ! $yes && [[ -t 0 ]]; then
      if [[ "$previous_choice" == declined || "$previous_choice" == skipped ]]; then
        printf 'Omarchy panel was %s last time. Install it now? [y/N] ' "$previous_choice" >&2
      else
        printf 'Install the optional Omarchy panel? [y/N] ' >&2
      fi
      reply=''
      read -r reply || true
      case "$reply" in y|Y|yes|Yes) install_plugin=true ;; *) choice=declined ;; esac
    elif ! $yes; then
      echo 'Omarchy panel skipped without a terminal; rerun with --yes to install it.' >&2
    fi
    if $install_plugin; then
      plugin=$("$prefix/bin/ws" setup plugin) || { echo 'Omarchy panel setup failed; rerun this command to retry.' >&2; exit 1; }
      choice=installed
    else
      plugin="{\"status\":\"$choice\"}"
    fi
    choice_dir=$(dirname -- "$choice_file")
    mkdir -p -m 700 -- "$choice_dir"
    choice_temporary=$(mktemp "$choice_dir/.plugin-choice.XXXXXXXX")
    printf '%s\n' "$choice" > "$choice_temporary"
    mv -Tf -- "$choice_temporary" "$choice_file"
  fi
fi
printf '{"environment":"local-install","applied":%s,"prefix":' "$(! $dry_run && echo true || echo false)"
json_string "$prefix"
printf ',"binary":'; json_string "$binary"
printf ',"version":'; json_string "$version"
printf ',"backup":'; json_string "$backup"
printf ',"packages":%s,"plugin":%s}\n' "$packages" "$plugin"
