#!/usr/bin/env bash
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)
case "${1:---help}" in
  --help|-h) echo 'Usage: ./ops disk:integration --preview|--apply'; echo 'Install the guarded Dolphin NTFS helper on an Arch host; --apply needs sudo.'; exit 0 ;;
  --preview|--apply) action=$1 ;;
  *) echo 'Expected --preview or --apply' >&2; exit 2 ;;
esac
[[ $(readlink -f /sbin) == /usr/bin ]] || { echo 'This helper installer currently supports merged /usr/bin hosts only.' >&2; exit 1; }
binary="$root/target/debug/ws"
[[ -x $binary ]] || { echo 'Run ./ops build first.' >&2; exit 1; }
if [[ -e /usr/bin/mount.ntfs3 ]] && ! grep -q 'Installed by ops disk:integration' /usr/bin/mount.ntfs3; then
  echo 'Refusing to replace an NTFS helper owned by another installer.' >&2; exit 1
fi
[[ $# == 1 ]] || { echo 'Expected one action' >&2; exit 2; }
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
"$binary" setup plan --gpu none --plugin false > "$work/setup.json"
domain=$(jq -er '.config.vm.name' "$work/setup.json")
uuid=$(jq -er '.config.disk.windows_partition_uuid' "$work/setup.json")
[[ $domain =~ ^[a-zA-Z0-9._-]+$ && $uuid =~ ^[a-zA-Z0-9-]+$ ]] || { echo 'Invalid Windows identity' >&2; exit 1; }
sed -e "s/@DOMAIN@/$domain/g" -e "s/@UUID@/$uuid/g" "$root/assets/host/mount.ntfs3" > "$work/helper"
jq --rawfile helper "$work/helper" '.root | .files |= map(select(.path == "/etc/libvirt/hooks/qemu.d/window-side-dirty")) | if (.files | length) != 1 then error("Separate-drive Windows setup required") else . end | .files += [{path:"/usr/bin/mount.ntfs3",contents:$helper,mode:493}] | .distro = null | .packages = [] | .groups = []' "$work/setup.json" > "$work/plan.json"
if [[ $action == --preview ]]; then cat "$work/plan.json"; exit 0; fi
# The root side restarts libvirt to discover the installed hooks.
[[ -z $(virsh -c qemu:///system list --name) ]] || { echo 'Shut down all VMs before installing mount integration.' >&2; exit 1; }
sudo env PKEXEC_UID="$(id -u)" "$binary" setup root "$work/plan.json"
jq -n --arg uuid "$uuid" --arg domain "$domain" '{environment:"local",status:"installed",windowsUuid:$uuid,domain:$domain,helper:"/usr/bin/mount.ntfs3"}'
