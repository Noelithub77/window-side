---
name: window-side
description: Operate the Windows VM that window-side runs from a dual-boot install, using the `ws` CLI on Linux. Use when the user wants to start, stop or check Windows, run PowerShell in Windows, copy files between Linux and Windows, reach Windows over SSH, or read its CPU, memory and update state.
---

# window-side

`ws` controls the Windows install from the user's dual boot, running as a KVM
virtual machine. Almost every command prints JSON; `ws <command> --help`
lists the options.

## State first

```sh
ws status        # plain text: VM state and boot profile (normal or GPU)
ws metrics       # JSON: status, cpu, memory, network, uptime, pending updates
```

Run commands inside Windows only when `ws status` says it is running. Every
guest command fails with "Start Windows first." otherwise.

## Lifecycle

```sh
ws start         # headless; waits for the guest agent, then signs in
ws stop          # clean Windows shutdown
```

- Ask before `ws start` if the user did not ask for Windows to run: it uses
  several GiB of memory.
- Ask before `ws stop`. Never run `ws force-stop` or `ws reboot` without an
  explicit request: they can lose unsaved work or restart Linux.
- `ws launch` and `ws console` open windows on the user's desktop. Use them
  only when the user wants to see Windows.

## Run PowerShell

```sh
ws exec --json 'Get-ComputerInfo | Select-Object OsName, OsVersion'
ws exec --json --file ./check.ps1 --timeout 120
```

`--json` returns `exitCode`, `stdout` and `stderr`; always use it and check
`exitCode`. This runs as SYSTEM in session 0 through the guest agent, so it
cannot see the signed-in desktop or the user's mapped drives. The default
timeout is 600 seconds.

```sh
ws ssh '$PSVersionTable.PSVersion.ToString()'
```

SSH runs as the configured Windows user with key-only, strict host-key trust.
Use it when a command needs the user's profile instead of SYSTEM. It needs
Windows OpenSSH; if it fails, report that and fall back to `ws exec`.

## Move files

```sh
ws copy ./app.zip 'C:\Users\Public\app.zip'          # Linux to Windows
ws ssh --put ./input.bin 'C:/Users/Public/input.bin'  # Linux to Windows over SSH
ws ssh --get 'C:/Logs/setup.log' ./setup.log          # Windows to Linux
```

`ws copy` goes through the guest agent: it works without SSH but is slow for
large files and does not create folders. Prefer `ws ssh --put` for big files.
Use `C:\Users\Public` or the user's temp folder for scratch files and clean
them up afterwards.

## Updates and display

```sh
ws updates status    # last saved report, no guest call
ws updates check     # asks Windows Update, up to 30 minutes
ws updates install   # installs pending updates; ask first
ws resolution --width 2560 --height 1440 --json
```

## Rules

- Change settings only with `ws config get` and `ws config set`;
  `ws config schema` describes every key. Never edit the config or files under
  `/etc` by hand.
- Never mount the Windows partition on Linux while the VM can use it.
- Do not read or print the stored sign-in password; `ws sign-in` handles it.
