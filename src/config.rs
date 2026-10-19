use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

/// Machine-specific choices written by `ws setup`; nothing here is committed to the repo.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub vm: Vm,
    pub disk: Disk,
    pub gpu: Option<Gpu>,
    pub windows: Windows,
    #[serde(default)]
    pub boot: Boot,
    #[serde(default)]
    pub hooks: Hooks,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vm {
    pub name: String,
    pub memory_mib: u32,
    pub vcpus: u32,
    /// Windows has the VirtIO drivers, so the disk and network use VirtIO instead of SATA and e1000e.
    #[serde(default = "enabled")]
    pub virtio: bool,
}

fn enabled() -> bool {
    true
}

/// How Windows reaches the VM: its own drive, or only its partition on a drive shared with Linux.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiskMode {
    #[default]
    Whole,
    Partition,
}

/// device-mapper name of the virtual disk assembled in partition mode.
pub const VIRTUAL_DISK: &str = "window-side";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    #[serde(default)]
    pub mode: DiskMode,
    /// Whole mode: /dev/disk/by-id name of the drive. Partition mode: GPT GUID of the Windows partition.
    pub id: String,
    /// Filesystem UUID of the Windows system partition, checked before every start.
    pub windows_partition_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gpu {
    pub address: String,
    /// Some laptops hide the GPU's HDMI audio function.
    pub audio_address: Option<String>,
    /// vendor:device IDs claimed by vfio-pci in the GPU boot profile.
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Windows {
    pub user: String,
    /// Refresh rate given to the Looking Glass virtual display, as Windows expects it.
    pub refresh_hz: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Boot {
    pub normal_entry: String,
    pub gpu_entry: String,
}

impl Default for Boot {
    fn default() -> Self {
        Self {
            normal_entry: "Omarchy/linux-omarchy".into(),
            gpu_entry: "Omarchy/window-side-gpu".into(),
        }
    }
}

/// Optional commands for machine-specific power or storage policies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hooks {
    /// Runs before every start; a non-zero exit stops the start.
    #[serde(default)]
    pub before_start: Vec<String>,
    /// Prints `{"ready": bool}` JSON describing the Windows disk.
    #[serde(default)]
    pub storage_status: Vec<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_dir().join("config.toml");
        let text = fs::read_to_string(&path)
            .with_context(|| format!("Run `ws setup` first; {} is missing", path.display()))?;
        toml::from_str(&text).with_context(|| format!("{} is invalid", path.display()))
    }

    /// The block device libvirt gives to Windows.
    pub fn disk_link(&self) -> PathBuf {
        match self.disk.mode {
            DiskMode::Whole => PathBuf::from("/dev/disk/by-id").join(&self.disk.id),
            DiskMode::Partition => PathBuf::from("/dev/mapper").join(VIRTUAL_DISK),
        }
    }

    /// Host devices Linux must not have mounted while Windows runs.
    pub fn host_device(&self) -> PathBuf {
        match self.disk.mode {
            DiskMode::Whole => self.disk_link(),
            DiskMode::Partition => PathBuf::from("/dev/disk/by-partuuid").join(&self.disk.id),
        }
    }
}

fn xdg(variable: &str, fallback: &str) -> PathBuf {
    env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(fallback))
        .join("window-side")
}

pub fn home() -> PathBuf {
    env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("/"))
}

pub fn config_dir() -> PathBuf {
    xdg("XDG_CONFIG_HOME", ".config")
}

pub fn state_dir() -> PathBuf {
    xdg("XDG_STATE_HOME", ".local/state")
}

pub fn cache_dir() -> PathBuf {
    xdg("XDG_CACHE_HOME", ".cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_config_uses_default_boot_entries_and_no_hooks() {
        let config: Config = toml::from_str(
            r#"
            [vm]
            name = "window-side"
            memory_mib = 16384
            vcpus = 8
            [disk]
            id = "nvme-eui.1"
            windows_partition_uuid = "ABCD"
            [windows]
            user = "Noel"
            refresh_hz = "60"
            "#,
        )
        .expect("valid config");
        assert!(config.gpu.is_none());
        assert_eq!(config.boot.gpu_entry, "Omarchy/window-side-gpu");
        assert!(config.hooks.before_start.is_empty());
        assert_eq!(config.disk_link(), PathBuf::from("/dev/disk/by-id/nvme-eui.1"));
    }
}
