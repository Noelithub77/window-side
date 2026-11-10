#!/usr/bin/env bash
# Links the installed Omarchy panel to this checkout so edits load without reinstalling.
# `ws setup plugin` and the installer leave a linked folder alone.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
id=noel.window-side
plugins="$HOME/.config/omarchy/plugins"
target="$plugins/$id"
source="$root/assets/plugin"
mkdir -p "$plugins"
backup=""
if [[ -e "$target" && ! -L "$target" ]]; then
  # Hidden names are skipped by the plugin registry.
  backup="$plugins/.$id.bak.$(date +%Y%m%d%H%M%S)"
  mv "$target" "$backup"
fi
# Replacing the link is a create/move event, which makes the shell reload the panel.
ln -sfn "$source" "$target"
# The shell IPC can time out while it reloads, so only enable when needed.
if ! omarchy plugin list --json | jq -e --arg id "$id" '.[] | select(.id == $id and .enabled)' >/dev/null; then
  omarchy plugin enable "$id" >/dev/null
fi
jq -n --arg target "$target" --arg source "$source" --arg backup "$backup" \
  '{plugin: "noel.window-side", status: "linked", target: $target, source: $source, backup: (if $backup == "" then null else $backup end)}'
