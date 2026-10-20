---
name: window-side-setup
description: Set up or change window-side, which runs a dual-boot Windows install as a VM on Linux. Use when the user wants to install or configure window-side, pick the Windows partition, GPU passthrough, memory or CPUs, or finish its first boot.
---

# window-side setup

Setup reads first and changes nothing until the user approves. All commands
print JSON.

## Read

```sh
ws --version
ws setup detect          # Windows installs, BitLocker, GPUs, CPUs, distro
ws setup packages        # missing VM packages
ws config path
ws config show           # only if a config exists
ws config validate
```

Also read `/etc/os-release`, `$XDG_CURRENT_DESKTOP`, CPU virtualization
support and whether IOMMU is on.

## Recommend

Summarise what you found in a few lines. Recommend the Windows partition, the
GPU to pass through (or `none`), memory and CPUs, with short pros and cons.
GPU passthrough needs a second GPU or a GPU boot entry; while Windows holds
the card, Linux cannot use it until a normal boot.

```sh
ws setup plan --gpu none --memory-gib 12 --vcpus 8
```

Explain the files and changes the plan lists.

## Apply

Ask before `ws setup apply` with the same options. It asks for the admin
password once and writes system files. Then guide first boot:

```sh
ws setup first-boot status
ws setup first-boot start    # boots Windows on SATA and opens the console
# The user runs virtio-win-guest-tools.exe from the driver CD, then signs in.
ws setup first-boot finish   # verifies drivers and restarts on VirtIO
```

If the switch fails, stop Windows and run `ws setup first-boot recover`.
`ws sign-in setup` saves the Windows password in the Linux keyring so later
starts sign in by themselves.

## Rules

- Later changes go through `ws config set`; disk and GPU changes go through
  `ws setup plan` and `ws setup apply`.
- Never edit `/etc` or the window-side config by hand, and never mount the
  Windows partition.
- Ask before shutting down, force-stopping or rebooting anything.
