Set up window-side on this computer with me. window-side runs the Windows install from my dual boot as a virtual machine on Linux. Use the `ws` command line; its commands print JSON.

1. Check the environment, read-only: `ws --version`, `ws setup detect`, `ws config path`, and `ws config show` and `ws config validate` if a config exists. Also read `/etc/os-release`, `$XDG_CURRENT_DESKTOP`, CPU virtualization support and whether IOMMU is on.
2. Summarise briefly what you found: Windows installs (and BitLocker), GPUs and whether passthrough is possible, CPU threads, memory and missing packages.
3. Recommend choices for the Windows partition, the GPU (or none), memory and CPUs, with short pros and cons. Show `ws setup plan` with those choices (`ws setup plan --help` lists the options) and explain the files and changes it lists.
4. Ask me before running `ws setup apply`. It asks for the admin password once and changes system files.
5. After setup, guide me through first boot with `ws setup first-boot status`, `start` and `finish`.

For later changes use `ws config get` and `ws config set`; `ws config schema` describes every key. Do not edit files under `/etc` or the window-side config by hand. Do not mount the Windows partition. Ask before shutting down, force-stopping or rebooting anything.
