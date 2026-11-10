"""Regression cases for the root mount helper's repair boundary; no real disks."""
from collections import Counter
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class MountGuard(unittest.TestCase):
    def run_helper(self, *, state="shut off", dirty=True, hibernated=False,
                   busy=False, mounted=False, source="/fake/windows", fail_repair=False):
        with tempfile.TemporaryDirectory() as directory:
            work = Path(directory)
            commands = work / "bin"
            commands.mkdir()
            runtime = work / "runtime"
            runtime.mkdir()
            if busy:
                (runtime / "busy").touch()
            mock = commands / "mock"
            mock.write_text('''#!/bin/sh
case "$(basename "$0")" in
readlink) case "$2" in /dev/disk/*) echo /fake/windows ;; *) echo "$2" ;; esac ;;
virsh) printf '%s\\n' "$VM_STATE" ;;
mount) echo mount >> "$TRACE"; [ -f "$ATTEMPT" ] && exit 0; touch "$ATTEMPT"; exit 32 ;;
ntfsinfo) printf 'Volume Flags: %s\\n' "$FLAGS" ;;
ntfsls) printf '%s\\n' "$LISTING" ;;
ntfsfix) echo repair >> "$TRACE"; exit "$REPAIR_EXIT" ;;
findmnt) exit "$FIND_EXIT" ;;
logger) : ;;
esac
''')
            mock.chmod(0o755)
            for name in ("readlink", "virsh", "mount", "ntfsinfo", "ntfsls", "ntfsfix", "findmnt", "logger"):
                (commands / name).symlink_to(mock)
            helper = work / "helper"
            helper.write_text((ROOT / "assets/host/mount.ntfs3").read_text()
                              .replace("PATH=/usr/bin:/usr/sbin:/bin:/sbin", f"PATH={commands}:/usr/bin:/bin")
                              .replace("/run/window-side-ntfs-@UUID@", str(runtime)))
            trace = work / "trace"
            result = subprocess.run(["sh", str(helper), source, "/fake/target", "-o", "rw"],
                                    env={**os.environ, "VM_STATE": state, "TRACE": str(trace),
                                         "ATTEMPT": str(work / "attempt"),
                                         "FLAGS": "0x0001 DIRTY" if dirty else "0x0000",
                                         "LISTING": "hiberfil.sys" if hibernated else "Users",
                                         "FIND_EXIT": "0" if mounted else "1",
                                         "REPAIR_EXIT": "1" if fail_repair else "0"},
                                    capture_output=True, text=True, timeout=5)
            return result.returncode, Counter(trace.read_text().splitlines()) if trace.exists() else Counter()

    def test_running_paused_and_unknown_state_refuse_all_disk_access(self):
        for state in ("running", "paused", "in shutdown", "", "unknown"):
            with self.subTest(state=state):
                code, calls = self.run_helper(state=state)
                self.assertNotEqual(code, 0)
                self.assertEqual(calls, {})

    def test_starting_vm_marker_refuses_mount_after_off_state_query(self):
        code, calls = self.run_helper(busy=True)
        self.assertNotEqual(code, 0)
        self.assertEqual(calls, {})

    def test_only_dirty_unmounted_nonhibernated_volume_is_repaired_then_retried(self):
        self.assertEqual(self.run_helper(), (0, {"mount": 2, "repair": 1}))
        for options in ({"dirty": False}, {"hibernated": True}, {"mounted": True}):
            with self.subTest(options=options):
                code, calls = self.run_helper(**options)
                self.assertNotEqual(code, 0)
                self.assertEqual(calls, {"mount": 1})

    def test_failed_repair_does_not_retry_mount(self):
        code, calls = self.run_helper(fail_repair=True)
        self.assertNotEqual(code, 0)
        self.assertEqual(calls, {"mount": 1, "repair": 1})

    def test_unrelated_volume_gets_no_vm_query_or_repair(self):
        code, calls = self.run_helper(source="/fake/other", state="running")
        self.assertEqual(calls, {"mount": 1})
        self.assertNotEqual(code, 0)


if __name__ == "__main__":
    unittest.main()
