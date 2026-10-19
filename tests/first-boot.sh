#!/usr/bin/env bash
# Only fake guest replies and temporary files; never connects to libvirt.
set -euo pipefail
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
fixture=$(mktemp -d)
trap 'rm -rf -- "$fixture"' EXIT
mkdir -p "$fixture/bin" "$fixture/config/window-side" "$fixture/state/window-side"
export HOME="$fixture" XDG_CONFIG_HOME="$fixture/config" XDG_STATE_HOME="$fixture/state"
export PATH="$fixture/bin:$PATH" WS_TEST_LOG="$fixture/virsh.log"
unset DBUS_SESSION_BUS_ADDRESS DISPLAY WAYLAND_DISPLAY HYPRLAND_INSTANCE_SIGNATURE
for notifier in omarchy notify-send; do
  printf '#!/usr/bin/env bash\nexit 0\n' > "$fixture/bin/$notifier"
  chmod +x "$fixture/bin/$notifier"
done
export WS_TEST_STATE='shut off' WS_TEST_AGENT=0
export WS_TEST_REPORT='{"user":"PC\\tester","storageReady":false,"networkReady":true,"applied":false}'
cat > "$fixture/bin/virsh" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$3" >> "$WS_TEST_LOG"
case "$3" in
  domstate) echo "$WS_TEST_STATE" ;;
  domblklist) printf 'file disk vdz /tmp/window-side-virtio-%s.img\n' "$(id -u)" ;;
  qemu-agent-command)
    case "$(jq -r .execute <<< "$5")" in
      guest-ping) [[ "$WS_TEST_AGENT" == 1 ]]; echo '{"return":{}}' ;;
      guest-exec) echo '{"return":{"pid":123}}' ;;
      guest-exec-status)
        encoded=$(printf %s "$WS_TEST_REPORT" | base64 -w0)
        jq -nc --arg data "$encoded" '{return:{exited:true,exitcode:0,"out-data":$data}}' ;;
      *) exit 95 ;;
    esac ;;
  define)
    if [[ "${WS_TEST_DEFINE_FAIL:-0}" == 1 && ! -f "$WS_TEST_LOG.failed" ]]; then
      touch "$WS_TEST_LOG.failed"; echo 'Rejected fixture definition' >&2; exit 1
    fi ;;
  *) echo "Unexpected mutation: $3" >&2; exit 96 ;;
esac
MOCK
chmod +x "$fixture/bin/virsh"
cat > "$fixture/config/window-side/config.toml" <<'CONFIG'
[vm]
name = "fixture"
memory_mib = 8192
vcpus = 4
virtio = false
[disk]
id = "fixture-disk"
windows_partition_uuid = "fixture-uuid"
[windows]
user = "tester"
refresh_hz = "60"
provisioned = false
CONFIG
binary="${WS_TEST_BINARY:-$root/target/debug/ws}"
status() { "$binary" setup first-boot status; }
status | jq -e '.stage == "start" and .environment == "window-side-libvirt"' >/dev/null
export WS_TEST_STATE=running
status | jq -e '.stage == "drivers"' >/dev/null
export WS_TEST_AGENT=1
status | jq -e '.stage == "drivers" and .networkReady and (.storageReady | not)' >/dev/null
if "$binary" setup first-boot finish >"$fixture/out" 2>"$fixture/error"; then echo 'Missing drivers allowed finish' >&2; exit 1; fi
rg -q 'Install the VirtIO guest tools' "$fixture/error"
! rg -q 'shutdown|define|attach-disk' "$WS_TEST_LOG"
export WS_TEST_REPORT='{"user":"","storageReady":true,"networkReady":true,"applied":false}'
status | jq -e '.stage == "sign-in"' >/dev/null
export WS_TEST_REPORT='{"user":"PC\\tester","storageReady":true,"networkReady":true,"applied":false}'
status | jq -e '.stage == "ready"' >/dev/null
export WS_TEST_REPORT='{"user":"tester"}'
if status >"$fixture/out" 2>"$fixture/error"; then echo 'Malformed readiness was accepted' >&2; exit 1; fi
rg -q 'invalid first-boot report' "$fixture/error"
if "$binary" setup first-boot recover >"$fixture/out" 2>"$fixture/error"; then echo 'Running VM allowed recovery' >&2; exit 1; fi
rg -q 'Stop Windows' "$fixture/error"
export WS_TEST_STATE='shut off'
printf 'sata fixture\n' > "$fixture/state/window-side/first-boot-domain-before.xml"
cp "$fixture/config/window-side/config.toml" "$fixture/state/window-side/first-boot-config-before.toml"
sed -i 's/virtio = false/virtio = true/' "$fixture/config/window-side/config.toml"
printf 'virtio fixture\n' > "$fixture/config/window-side/domain.xml"
cp "$fixture/config/window-side/config.toml" "$fixture/current.toml"
export WS_TEST_DEFINE_FAIL=1
if "$binary" setup first-boot recover >"$fixture/out" 2>"$fixture/error"; then echo 'Rejected definition reported success' >&2; exit 1; fi
cmp "$fixture/current.toml" "$fixture/config/window-side/config.toml"
rg -q '^virtio fixture$' "$fixture/config/window-side/domain.xml"
export WS_TEST_DEFINE_FAIL=0
"$binary" setup first-boot recover | jq -e '.stage == "start"' >/dev/null
rg -q '^virtio = false$' "$fixture/config/window-side/config.toml"
rg -q '^sata fixture$' "$fixture/config/window-side/domain.xml"
printf '%s\n' 'First-boot fixture checks passed: readiness, refusal before mutation, recovery and rollback'
# Plugin repair also runs without hardware detection or a Windows config.
mkdir -p "$fixture/.config/omarchy/plugins" "$fixture/external-plugin"
printf 'keep linked plugin\n' > "$fixture/external-plugin/manifest.json"
ln -s "$fixture/external-plugin" "$fixture/.config/omarchy/plugins/noel.window-side"
"$binary" setup plugin | jq -e '.status == "managed-externally"' >/dev/null
rg -q '^keep linked plugin$' "$fixture/external-plugin/manifest.json"
rm "$fixture/.config/omarchy/plugins/noel.window-side"
cat > "$fixture/bin/omarchy" <<'MOCK'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == 'plugin enable noel.window-side' ]]
MOCK
chmod +x "$fixture/bin/omarchy"
"$binary" setup plugin | jq -e '.status == "installed"' >/dev/null
jq -e '.id == "noel.window-side"' "$fixture/.config/omarchy/plugins/noel.window-side/manifest.json" >/dev/null
rm "$fixture/.config/omarchy/plugins/noel.window-side/manifest.json"
ln -s "$fixture/external-plugin/manifest.json" "$fixture/.config/omarchy/plugins/noel.window-side/manifest.json"
"$binary" setup plugin | jq -e '.status == "managed-externally"' >/dev/null
rg -q '^keep linked plugin$' "$fixture/external-plugin/manifest.json"
printf '%s\n' 'Plugin fixture checks passed: install, enable, folder and file symlink preservation'
