---
name: windows-app-testing
description: Build, install and test Windows software from Linux on the user's real Windows install through window-side (`ws`). Use when the user wants to check a Windows build, run tests or an installer on Windows, reproduce a Windows-only bug, or collect Windows logs, without rebooting.
---

# Windows app testing

Test on the user's real Windows, running as a VM beside Linux. Read the
`window-side` skill for command details and safety rules.

## Loop

1. Check state: `ws status`. If Windows is stopped, ask before `ws start`.
2. Build on Linux when the project can cross-compile; otherwise copy the
   source and build in Windows.
3. Copy inputs in, into a fresh folder per run:

   ```sh
   run="C:/Users/Public/ws-test/$(date +%s)"
   ws exec --json "New-Item -ItemType Directory -Force '$run' | Out-Null"
   ws copy ./dist/app.zip "$run/app.zip"
   ```

4. Run the work and keep the JSON result:

   ```sh
   ws exec --json --timeout 1800 "Set-Location '$run'; Expand-Archive app.zip -Force; .\app\run-tests.ps1"
   ```

   A non-zero `exitCode` is a failed run. Read `stderr`, then fetch logs.
5. Bring results back: `ws ssh --get "$run/results.xml" ./results.xml`, or
   print small files with `ws exec --json "Get-Content '$run\log.txt' -Tail 200"`.
6. Remove the run folder unless the user wants it kept.

## Installers

- Silent installs run fine as SYSTEM:
  `ws exec --json "Start-Process '$run\setup.exe' -ArgumentList '/S' -Wait -PassThru | Select-Object ExitCode"`.
- Check the result in the registry or on disk, not only the exit code.
- Ask before installing anything outside the run folder, and offer to
  uninstall afterwards. This is the user's own Windows, not a throwaway VM.

## Limits

- `ws exec` runs in session 0 and cannot drive the desktop. For GUI tests, ask
  the user to open Windows (`ws launch`) or use SSH as the signed-in user with
  a tool that supports it.
- Per-user state (HKCU, AppData) differs between SYSTEM and the user; use
  `ws ssh` when it matters.
- `ws metrics` shows CPU and memory during long runs; Windows Update can slow
  runs, so check `ws updates status` when timings look wrong.
