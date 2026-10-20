//! Turns detected hardware and a few choices into the VM, its config and the system files.
pub mod boot;
pub mod domain;
pub mod files;
pub mod root;
pub mod packages;

use crate::config::{Boot, Config, Disk, DiskMode, Gpu, Vm, Windows, home};
use crate::detect::disks::WindowsInstall;
use crate::detect::gpus::{Blocker, Gpu as DetectedGpu};
use crate::detect::system::System;
use crate::detect::{disks, gpus, system};
use crate::process::{has_command, output, text};
use crate::vm::{lifecycle, virsh};
use anyhow::{Context, Result, bail};
use root::{FOOTER_SECTORS, HEADER_SECTORS, RootPlan, VirtualDisk};
use serde::Serialize;
use std::fs;
use std::path::Path;

const PLUGIN_ID: &str = "noel.window-side";
const PLUGIN: &[(&str, &str)] = &[
    ("manifest.json", include_str!("../assets/plugin/manifest.json")),
    ("qmldir", include_str!("../assets/plugin/qmldir")),
    ("Service.qml", include_str!("../assets/plugin/Service.qml")),
    ("ServiceBridge.qml", include_str!("../assets/plugin/ServiceBridge.qml")),
    ("Panel.qml", include_str!("../assets/plugin/Panel.qml")),
    ("BarWidget.qml", include_str!("../assets/plugin/BarWidget.qml")),
];

#[derive(Debug, Clone, Serialize)]
pub struct Inventory {
    pub windows: Vec<WindowsInstall>,
    pub gpus: Vec<DetectedGpu>,
    pub system: System,
}

pub fn detect() -> Result<Inventory> {
    Ok(Inventory { windows: disks::detect()?, gpus: gpus::detect(), system: system::detect() })
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Choices {
    /// Partition path of the chosen Windows install.
    pub windows: String,
    pub gpu: Option<String>,
    pub memory_mib: u32,
    pub vcpus: u32,
    pub plugin: bool,
}

impl Inventory {
    pub fn usable_gpus(&self) -> impl Iterator<Item = &DetectedGpu> {
        let bootable = self.system.boot_profile.is_some();
        self.gpus.iter().filter(move |gpu| bootable && matches!(gpu.blocker, None | Some(Blocker::IommuOff)))
    }

    pub fn memory_limit_mib(&self) -> u32 {
        u32::try_from(self.system.memory_mib.saturating_sub(4096)).unwrap_or(u32::MAX).max(2048)
    }

    pub fn vcpu_limit(&self) -> u32 {
        u32::try_from(self.system.cpus.len()).unwrap_or(2).saturating_sub(2).max(2)
    }

    /// The current setup when there is one; otherwise half the memory and CPUs,
    /// the largest Windows install and the first usable GPU.
    pub fn default_choices(&self, existing: Option<&Config>) -> Option<Choices> {
        if let Some(config) = existing {
            let windows = self.windows.iter().find(|install| install.filesystem_uuid == config.disk.windows_partition_uuid);
            if let Some(windows) = windows {
                return Some(Choices {
                    windows: windows.partition.clone(),
                    gpu: config.gpu.as_ref().map(|gpu| gpu.address.clone()),
                    memory_mib: config.vm.memory_mib,
                    vcpus: config.vm.vcpus,
                    plugin: self.system.omarchy,
                });
            }
        }
        let windows = self.windows.first()?.partition.clone();
        let half_gib = u32::try_from(self.system.memory_mib / 2048).unwrap_or(4).max(4);
        let half_cpus = (u32::try_from(self.system.cpus.len()).unwrap_or(4) / 2) & !1;
        Some(Choices {
            windows,
            gpu: self.usable_gpus().next().map(|gpu| gpu.address.clone()),
            memory_mib: (half_gib * 1024).min(self.memory_limit_mib()),
            vcpus: half_cpus.clamp(2, self.vcpu_limit()),
            plugin: self.system.omarchy,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub config: Config,
    pub domain_xml: String,
    pub root: RootPlan,
    pub plugin: bool,
    /// Short facts the user should know before applying.
    pub notes: Vec<String>,
}

/// On hybrid Intel CPUs Windows gets whole performance cores and QEMU's own threads the efficiency cores.
fn pins(system: &System, vcpus: u32) -> (Vec<u32>, Vec<u32>) {
    let count = vcpus as usize;
    if system.efficiency_cpus.is_empty() || system.performance_cpus.len() < count {
        return (Vec::new(), Vec::new());
    }
    let start = system.performance_cpus.len() - count;
    (system.performance_cpus[start..].to_vec(), system.efficiency_cpus.clone())
}

fn random_uuid() -> Result<String> {
    Ok(fs::read_to_string("/proc/sys/kernel/random/uuid")?.trim().to_owned())
}

fn udev_property(device: &str, name: &str) -> Option<String> {
    let properties = text("udevadm", &["info", "-q", "property", "-n", device]).ok()?;
    properties.lines().find_map(|line| line.strip_prefix(&format!("{name}="))).map(str::to_owned)
}

pub fn plan(inventory: &Inventory, choices: &Choices, existing: Option<&Config>) -> Result<Plan> {
    let install = inventory.windows.iter().find(|install| install.partition == choices.windows).context("Choose a detected Windows install")?;
    let gpu = match &choices.gpu {
        Some(address) => Some(inventory.usable_gpus().find(|gpu| &gpu.address == address).context("That GPU cannot be passed through")?),
        None => None,
    };
    if choices.memory_mib < 2048 || choices.memory_mib > inventory.memory_limit_mib() {
        bail!("Give Windows between 2 GiB and {} GiB", inventory.memory_limit_mib() / 1024);
    }
    if choices.vcpus < 2 || choices.vcpus > inventory.vcpu_limit() {
        bail!("Give Windows between 2 and {} CPUs", inventory.vcpu_limit());
    }
    let system = &inventory.system;
    let mut notes = Vec::new();
    if system.boot_profile.is_none() {
        notes.push("No GPU profile on this boot loader: the running entry or initramfs tool is unavailable.".into());
    }
    let disk = match install.mode {
        DiskMode::Whole => Disk {
            mode: DiskMode::Whole,
            id: install.disk_id.clone().context("The Windows drive has no stable /dev/disk/by-id name")?,
            windows_partition_uuid: install.filesystem_uuid.clone(),
        },
        DiskMode::Partition => {
            notes.push("Shares a drive with Linux: Windows gets only its partition and a copy of its boot files.".into());
            Disk { mode: DiskMode::Partition, id: install.part_uuid.clone(), windows_partition_uuid: install.filesystem_uuid.clone() }
        }
    };
    if install.bitlocker {
        notes.push("BitLocker is on: Windows asks for the recovery key in the VM, whose TPM differs.".into());
    }
    let same_disk = existing.is_some_and(|config| config.disk.id == disk.id);
    let config = Config {
        vm: Vm {
            name: existing.map_or_else(|| "window-side".to_owned(), |config| config.vm.name.clone()),
            memory_mib: choices.memory_mib,
            vcpus: choices.vcpus,
            virtio: same_disk && existing.is_some_and(|config| config.vm.virtio),
        },
        disk,
        gpu: gpu.map(|gpu| Gpu { address: gpu.address.clone(), audio_address: gpu.audio_address.clone(), ids: gpu.ids.clone() }),
        windows: Windows {
            provisioned: same_disk && existing.is_some_and(|config| config.windows.provisioned),
            user: existing.map_or_else(|| system.user.clone(), |config| config.windows.user.clone()),
            refresh_hz: system.refresh_hz.clone(),
        },
        boot: existing.map(|config| config.boot.clone()).unwrap_or_default(),
        hooks: existing.map(|config| config.hooks.clone()).unwrap_or_default(),
    };

    let old_xml = fs::read_to_string(lifecycle::domain_file()).ok();
    let (uuid, mac) = old_xml.as_deref().map(domain::identity).unwrap_or_default();
    let uuid = match uuid { Some(uuid) => uuid, None => random_uuid()? };
    let mac = mac.unwrap_or_else(|| format!("52:54:00:{}:{}:{}", &uuid[0..2], &uuid[2..4], &uuid[4..6]));
    let serial = old_xml.as_deref().filter(|_| same_disk).and_then(domain::existing_serial);
    let (vcpu_pins, emulator_pins) = pins(system, choices.vcpus);
    let disk_path = config.disk_link().display().to_string();
    let domain_xml = domain::render(&domain::Domain {
        name: &config.vm.name, uuid: &uuid, mac: &mac, memory_mib: config.vm.memory_mib, vcpus: config.vm.vcpus,
        threads_per_core: system.threads_per_core, vcpu_pins: &vcpu_pins, emulator_pins: &emulator_pins,
        disk: &disk_path, serial: serial.as_deref(), virtio: config.vm.virtio, looking_glass: gpu.is_some(),
    });
    if !config.vm.virtio {
        notes.push("First boot uses SATA and e1000e until the VirtIO drivers are installed.".into());
    }

    let mut files = Vec::new();
    let mut gpu_kernel = None;
    if let (Some(gpu), Some(profile)) = (gpu, &system.boot_profile) {
        let current = fs::read_to_string("/proc/cmdline").context("Could not read the kernel command line")?;
        let cmdline = files::gpu_cmdline(&current, system.iommu_flag, &gpu.ids, gpu.driver.as_deref());
        let modules = match fs::read_to_string(boot::INITRAMFS_MODULES) {
            Ok(modules) => modules,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error).context("Could not preserve the initramfs module list"),
        };
        files.extend(boot::generate(profile, &cmdline, system.iommu_flag, &modules)?);
        files.push(files::looking_glass_file(&system.user));
        gpu_kernel = (profile.loader == crate::detect::boot::Loader::Limine).then(|| profile.kernel.clone());
        if gpu.blocker == Some(Blocker::IommuOff) {
            notes.push("IOMMU turns on after the next reboot; the GPU is checked again then.".into());
        }
        if !has_command("looking-glass-client") {
            notes.push(format!("Install the Looking Glass client {} to see Windows on the GPU.", crate::display::looking_glass::TAG));
        }
    }
    let virtual_disk = match install.mode {
        DiskMode::Whole => {
            let serial = udev_property(&install.disk, "ID_SERIAL").context("Could not read the drive serial")?;
            let group = if text("getent", &["group", "libvirt-qemu"]).is_ok() { "libvirt-qemu" } else { "kvm" };
            files.push(files::udev_file(&serial, group));
            None
        }
        DiskMode::Partition => {
            let esp = install.esp.as_ref().context("The drive has no EFI partition with Windows boot files")?;
            let windows_sectors = install.size / 512;
            files.extend(files::virtual_disk_files(&install.part_uuid, HEADER_SECTORS, windows_sectors, FOOTER_SECTORS));
            Some(VirtualDisk {
                disk_guid: install.disk_guid.clone(),
                esp_part_uuid: esp.part_uuid.clone(),
                windows_part_uuid: install.part_uuid.clone(),
                windows_sectors,
            })
        }
    };
    let packages = packages(system)?;
    let member_of = text("id", &["-nG", &system.user]).unwrap_or_default();
    let groups = ["libvirt", "kvm"].iter().filter(|group| !member_of.split_whitespace().any(|name| name == **group)).map(|group| (*group).to_owned()).collect();
    let root = RootPlan { user: system.user.clone(), distro: system.distro, packages, groups, files, virtual_disk, gpu_kernel, boot_profile: gpu.and(system.boot_profile.clone()) };
    root.validate()?;
    Ok(Plan { config, domain_xml, root, plugin: choices.plugin, notes })
}

/// Packages providing the missing commands and the UEFI firmware.
fn packages(system: &System) -> Result<Vec<String>> {
    let Some(family) = system.distro else {
        if system.missing.is_empty() {
            return Ok(Vec::new());
        }
        let commands: Vec<_> = system.missing.iter().map(|missing| missing.command).collect();
        bail!("Install the packages that provide {} with your package manager, then run setup again.", commands.join(", "));
    };
    let mut packages: Vec<String> = Vec::new();
    let firmware = (!family.firmware_present()).then(|| family.firmware_package());
    for package in system.missing.iter().flat_map(|missing| missing.packages.iter().copied()).chain(firmware) {
        if !packages.iter().any(|known| known == package) {
            packages.push(package.to_owned());
        }
    }
    Ok(packages)
}

/// Writes everything: one pkexec prompt for the system side, then the user config and the VM.
pub fn apply(plan: &Plan) -> Result<()> {
    if virsh::is_running(&plan.config.vm.name) {
        bail!("Stop Windows before changing its setup.");
    }
    let mut config = plan.config.clone();
    let runtime = std::env::var_os("XDG_RUNTIME_DIR").map_or_else(std::env::temp_dir, std::path::PathBuf::from);
    let plan_file = runtime.join("window-side-setup.json");
    crate::guest::commands::write_private(&plan_file, &serde_json::to_string(&plan.root)?)?;
    let program = std::env::current_exe().context("Could not find the ws binary")?;
    let result = output("pkexec", &[&program.display().to_string(), "setup", "root", &plan_file.display().to_string()]);
    let _ = fs::remove_file(&plan_file);
    let result = result?;
    if !result.status.success() {
        bail!("System setup failed: {}", String::from_utf8_lossy(&result.stderr).trim());
    }
    let entries: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap_or_default();
    if let (Some(normal), Some(gpu)) = (entries["normalEntry"].as_str(), entries["gpuEntry"].as_str()) {
        config.boot = Boot { normal_entry: normal.to_owned(), gpu_entry: gpu.to_owned() };
    }
    config.save()?;
    fs::write(lifecycle::domain_file(), &plan.domain_xml)?;
    virsh::run(&["define", "--validate", &lifecycle::domain_file().display().to_string()]).context("libvirt rejected the VM definition")?;
    if plan.plugin {
        install_plugin()?;
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginInstall {
    Installed,
    ManagedExternally,
}

/// Installs the bundled panel; a plugin folder linked from elsewhere (dotfiles) is left alone.
pub fn install_plugin() -> Result<PluginInstall> {
    let folder = home().join(".config/omarchy/plugins").join(PLUGIN_ID);
    let linked = |path: &Path| path.symlink_metadata().is_ok_and(|meta| meta.file_type().is_symlink());
    if linked(&folder) || PLUGIN.iter().any(|(name, _)| linked(&folder.join(name))) {
        return Ok(PluginInstall::ManagedExternally);
    }
    fs::create_dir_all(&folder)?;
    for (name, contents) in PLUGIN {
        fs::write(folder.join(name), contents)?;
    }
    text("omarchy", &["plugin", "enable", PLUGIN_ID]).context("Could not enable the Omarchy plugin")?;
    Ok(PluginInstall::Installed)
}
