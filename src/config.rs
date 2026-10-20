pub mod edit;

use anyhow::{Context, Result, bail, ensure};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};

const FILE: &str = "config.json";
const SCHEMA_FILE: &str = "config.schema.json";
/// Configs written before JSON; migrated once, then kept as `config.toml.migrated`.
const LEGACY_FILE: &str = "config.toml";

/// Machine-specific choices written by `ws setup`; nothing here is committed to the repo.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum DiskMode {
    #[default]
    Whole,
    Partition,
}

/// device-mapper name of the virtual disk assembled in partition mode.
pub const VIRTUAL_DISK: &str = "window-side";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Disk {
    #[serde(default)]
    pub mode: DiskMode,
    /// Whole mode: /dev/disk/by-id name of the drive. Partition mode: GPT GUID of the Windows partition.
    pub id: String,
    /// Filesystem UUID of the Windows system partition, checked before every start.
    pub windows_partition_uuid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Gpu {
    pub address: String,
    /// Some laptops hide the GPU's HDMI audio function.
    pub audio_address: Option<String>,
    /// vendor:device IDs claimed by vfio-pci in the GPU boot profile.
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Windows {
    pub user: String,
    /// False until first-boot provisioning and the VirtIO reboot have been verified.
    #[serde(default = "enabled")]
    pub provisioned: bool,
    /// Refresh rate given to the Looking Glass virtual display, as Windows expects it.
    pub refresh_hz: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
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
    pub fn path() -> PathBuf {
        config_dir().join(FILE)
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        match fs::read_to_string(&path) {
            Ok(text) => Self::parse(&text).with_context(|| format!("{} is invalid", path.display())),
            Err(error) if error.kind() == ErrorKind::NotFound => Self::migrate(),
            Err(error) => Err(error).with_context(|| format!("Could not read {}", path.display())),
        }
    }

    /// Reads config JSON; the `$schema` editor hint is not part of the config.
    pub fn parse(text: &str) -> Result<Self> {
        let mut value: Value = serde_json::from_str(text)?;
        if let Some(object) = value.as_object_mut() {
            object.remove("$schema");
        }
        Ok(serde_json::from_value(value)?)
    }

    pub fn to_json(&self) -> Result<String> {
        let mut value = serde_json::to_value(self)?;
        value["$schema"] = Value::from(format!("./{SCHEMA_FILE}"));
        Ok(serde_json::to_string_pretty(&value)? + "\n")
    }

    /// Validates, then replaces the file atomically beside a current schema.
    pub fn save(&self) -> Result<()> {
        self.validate()?;
        let directory = config_dir();
        fs::create_dir_all(&directory)?;
        write_atomic(&directory.join(SCHEMA_FILE), &(serde_json::to_string_pretty(&schema())? + "\n"))?;
        write_atomic(&Self::path(), &self.to_json()?)
    }

    fn migrate() -> Result<Self> {
        let legacy = config_dir().join(LEGACY_FILE);
        let text = match fs::read_to_string(&legacy) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                bail!("Run `ws setup` first; {} is missing", Self::path().display())
            }
            Err(error) => return Err(error).with_context(|| format!("Could not read {}", legacy.display())),
        };
        let config: Self = toml::from_str(&text).with_context(|| format!("{} is invalid", legacy.display()))?;
        config.save()?;
        fs::rename(&legacy, legacy.with_extension("toml.migrated"))?;
        Ok(config)
    }

    /// Value checks the types alone cannot express.
    pub fn validate(&self) -> Result<()> {
        let name_ok = |name: &str| !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || "._-".contains(c));
        ensure!(name_ok(&self.vm.name), "vm.name may use letters, digits, '.', '_' and '-'");
        ensure!(self.vm.memory_mib >= 2048, "vm.memory_mib must be at least 2048");
        ensure!(self.vm.vcpus >= 2, "vm.vcpus must be at least 2");
        ensure!(!self.disk.id.is_empty() && !self.disk.windows_partition_uuid.is_empty(), "disk.id and disk.windows_partition_uuid are required");
        ensure!(!self.windows.user.trim().is_empty(), "windows.user is required");
        ensure!(
            self.windows.refresh_hz.parse::<f64>().is_ok_and(|rate| rate > 0.),
            "windows.refresh_hz must be a positive number, e.g. \"60\" or \"165.003\""
        );
        for (key, entry) in [("boot.normal_entry", &self.boot.normal_entry), ("boot.gpu_entry", &self.boot.gpu_entry)] {
            ensure!(crate::host::valid_entry(entry), "{key} is not a valid boot entry");
        }
        if let Some(gpu) = &self.gpu {
            for address in std::iter::once(&gpu.address).chain(&gpu.audio_address) {
                ensure!(crate::host::valid_pci_address(address), "{address} is not a PCI address like 0000:01:00.0");
            }
        }
        Ok(())
    }

    pub fn needs_first_boot(&self) -> bool {
        !self.vm.virtio || !self.windows.provisioned
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

/// JSON Schema of `config.json`, generated from these types.
pub fn schema() -> Value {
    let mut schema = serde_json::to_value(schemars::schema_for!(Config)).unwrap_or_default();
    // The file names its schema; that key is the only one outside the config itself.
    schema["properties"]["$schema"] = serde_json::json!({ "type": "string" });
    schema
}

fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let temporary = path.with_extension("json.tmp");
    let mut file = fs::File::create(&temporary).with_context(|| format!("Could not write {}", temporary.display()))?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    fs::rename(&temporary, path).with_context(|| format!("Could not save {}", path.display()))
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
        assert!(!config.needs_first_boot());
        let mut pending = config.clone();
        pending.vm.virtio = false;
        assert!(pending.needs_first_boot());
        pending.vm.virtio = true;
        pending.windows.provisioned = false;
        assert!(pending.needs_first_boot());
    }

    #[test]
    fn json_round_trips_with_its_schema_hint_and_validates() {
        let config = Config::parse(
            r#"{"$schema": "./config.schema.json", "vm": {"name": "window-side", "memory_mib": 16384, "vcpus": 8},
                "disk": {"id": "nvme-eui.1", "windows_partition_uuid": "ABCD"},
                "gpu": {"address": "0000:01:00.0", "audio_address": null, "ids": ["10de:28a0"]},
                "windows": {"user": "Noel", "refresh_hz": "165.003"}}"#,
        )
        .expect("valid config");
        config.validate().expect("valid values");
        let json = config.to_json().expect("serializes");
        assert!(json.contains(r#""$schema": "./config.schema.json""#));
        assert_eq!(Config::parse(&json).expect("round trip").vm.memory_mib, 16384);
        assert!(Config::parse(r#"{"surprise": 1}"#).is_err());
        let mut bad = config.clone();
        bad.gpu = Some(Gpu { address: "1:0".into(), audio_address: None, ids: Vec::new() });
        assert!(bad.validate().is_err());
        assert_eq!(schema()["properties"]["$schema"]["type"], "string");
    }
}
