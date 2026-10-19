//! Finds Windows installs and decides whether the VM can take their whole drive.
use crate::config::DiskMode;
use crate::process::text;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

const ESP_TYPE: &str = "c12a7328-f81f-11d2-ba4b-00a0c93ec93b";
const BASIC_DATA_TYPE: &str = "ebd0a0a2-b9e5-4433-87c0-68b6b72699c7";
const LINUX_FILESYSTEMS: &[&str] = &[
    "ext2", "ext3", "ext4", "btrfs", "xfs", "f2fs", "bcachefs", "crypto_LUKS", "swap", "LVM2_member", "zfs_member",
];

#[derive(Debug, Deserialize)]
struct Lsblk {
    blockdevices: Vec<Device>,
}

#[derive(Debug, Clone, Deserialize)]
struct Device {
    path: String,
    #[serde(rename = "type")]
    kind: String,
    size: u64,
    model: Option<String>,
    ptuuid: Option<String>,
    parttype: Option<String>,
    partuuid: Option<String>,
    fstype: Option<String>,
    uuid: Option<String>,
    #[serde(default)]
    mountpoints: Vec<Option<String>>,
    #[serde(default)]
    children: Vec<Device>,
}

impl Device {
    fn mounts(&self) -> Vec<String> {
        self.mountpoints.iter().flatten().cloned().collect()
    }

    /// Mount points of this device and everything stacked on it (LUKS, LVM).
    fn all_mounts(&self) -> Vec<String> {
        let mut mounts = self.mounts();
        mounts.extend(self.children.iter().flat_map(Device::all_mounts));
        mounts
    }

    fn holds_linux(&self) -> bool {
        self.fstype.as_deref().is_some_and(|fstype| LINUX_FILESYSTEMS.contains(&fstype))
            || self.all_mounts().iter().any(|mount| mount == "/" || mount == "[SWAP]")
            || self.children.iter().any(Device::holds_linux)
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Esp {
    pub path: String,
    pub part_uuid: String,
    pub mounts: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowsInstall {
    pub disk: String,
    pub disk_model: String,
    /// Stable /dev/disk/by-id name for the whole drive.
    pub disk_id: Option<String>,
    pub disk_guid: String,
    pub disk_size: u64,
    pub partition: String,
    pub size: u64,
    /// Filesystem UUID, checked before every start.
    pub filesystem_uuid: String,
    /// GPT partition GUID, which Windows' boot configuration refers to.
    pub part_uuid: String,
    pub bitlocker: bool,
    pub esp: Option<Esp>,
    pub mounts: Vec<String>,
    /// The drive also holds Linux, so only the partition can be given to the VM.
    pub shares_linux: bool,
    pub mode: DiskMode,
}

pub fn detect() -> Result<Vec<WindowsInstall>> {
    let json = text("lsblk", &[
        "-J", "-b", "-o", "NAME,PATH,TYPE,SIZE,MODEL,PTUUID,PARTTYPE,PARTUUID,FSTYPE,UUID,MOUNTPOINTS",
    ])?;
    let tree: Lsblk = serde_json::from_str(&json).context("lsblk returned unexpected JSON")?;
    Ok(installs(&tree.blockdevices, &disk_id))
}

fn installs(devices: &[Device], disk_id: &dyn Fn(&str) -> Option<String>) -> Vec<WindowsInstall> {
    let mut found = Vec::new();
    for disk in devices.iter().filter(|device| device.kind == "disk") {
        let esp = disk.children.iter().find(|part| part.parttype.as_deref() == Some(ESP_TYPE)).map(|part| Esp {
            path: part.path.clone(),
            part_uuid: part.partuuid.clone().unwrap_or_default(),
            mounts: part.mounts(),
        });
        let shares_linux = disk.children.iter().any(Device::holds_linux);
        for part in &disk.children {
            let windows_fs = matches!(part.fstype.as_deref(), Some("ntfs" | "BitLocker"));
            if part.parttype.as_deref() != Some(BASIC_DATA_TYPE) || !windows_fs {
                continue;
            }
            found.push(WindowsInstall {
                disk: disk.path.clone(),
                disk_model: disk.model.clone().unwrap_or_default().trim().to_owned(),
                disk_id: disk_id(&disk.path),
                disk_guid: disk.ptuuid.clone().unwrap_or_default(),
                disk_size: disk.size,
                partition: part.path.clone(),
                size: part.size,
                filesystem_uuid: part.uuid.clone().unwrap_or_default(),
                part_uuid: part.partuuid.clone().unwrap_or_default(),
                bitlocker: part.fstype.as_deref() == Some("BitLocker"),
                esp: esp.clone(),
                mounts: part.mounts(),
                shares_linux,
                mode: if shares_linux { DiskMode::Partition } else { DiskMode::Whole },
            });
        }
    }
    // Largest first: the system partition is normally bigger than data or recovery volumes.
    found.sort_by_key(|install| std::cmp::Reverse(install.size));
    found
}

/// Prefers identifiers that survive controller and slot changes.
fn disk_id(device: &str) -> Option<String> {
    let entries = fs::read_dir("/dev/disk/by-id").ok()?;
    let target = fs::canonicalize(device).ok()?;
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| fs::canonicalize(entry.path()).is_ok_and(|resolved| resolved == target))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    let rank = |name: &String| {
        ["nvme-eui.", "wwn-", "nvme-", "ata-", "scsi-"].iter().position(|prefix| name.starts_with(prefix)).unwrap_or(9)
    };
    names.sort_by_key(|name| (rank(name), name.clone()));
    names.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<Device> {
        let json = r#"{"blockdevices":[
          {"path":"/dev/nvme1n1","type":"disk","size":512,"model":"WD","ptuuid":"d1","parttype":null,"partuuid":null,"fstype":null,"uuid":null,"mountpoints":[null],"children":[
            {"path":"/dev/nvme1n1p1","type":"part","size":2,"model":null,"ptuuid":"d1","parttype":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","partuuid":"e1","fstype":"vfat","uuid":"B1","mountpoints":["/boot"]},
            {"path":"/dev/nvme1n1p2","type":"part","size":300,"model":null,"ptuuid":"d1","parttype":"4f68bce3-e8cd-4db1-96e7-fbcaf984b709","partuuid":"l1","fstype":"crypto_LUKS","uuid":"L","mountpoints":[null],"children":[
              {"path":"/dev/mapper/root","type":"crypt","size":300,"model":null,"ptuuid":null,"parttype":null,"partuuid":null,"fstype":"btrfs","uuid":"R","mountpoints":["/"]}]},
            {"path":"/dev/nvme1n1p3","type":"part","size":200,"model":null,"ptuuid":"d1","parttype":"ebd0a0a2-b9e5-4433-87c0-68b6b72699c7","partuuid":"w1","fstype":"BitLocker","uuid":"W1","mountpoints":[null]}]},
          {"path":"/dev/nvme0n1","type":"disk","size":1000,"model":"CT1000P3PSSD8  ","ptuuid":"d0","parttype":null,"partuuid":null,"fstype":null,"uuid":null,"mountpoints":[null],"children":[
            {"path":"/dev/nvme0n1p1","type":"part","size":1,"model":null,"ptuuid":"d0","parttype":"c12a7328-f81f-11d2-ba4b-00a0c93ec93b","partuuid":"e0","fstype":"vfat","uuid":"B0","mountpoints":[null]},
            {"path":"/dev/nvme0n1p3","type":"part","size":900,"model":null,"ptuuid":"d0","parttype":"ebd0a0a2-b9e5-4433-87c0-68b6b72699c7","partuuid":"w0","fstype":"ntfs","uuid":"E07E","mountpoints":["/mnt/c"]},
            {"path":"/dev/nvme0n1p4","type":"part","size":1,"model":null,"ptuuid":"d0","parttype":"de94bba4-06d1-4d40-a16a-bfd50179d6ac","partuuid":"r0","fstype":"ntfs","uuid":"R0","mountpoints":[null]}]}
        ]}"#;
        serde_json::from_str::<Lsblk>(json).expect("valid sample").blockdevices
    }

    #[test]
    fn own_drive_gets_whole_mode_and_shared_drive_gets_partition_mode() {
        let found = installs(&sample(), &|_| None);
        assert_eq!(found.len(), 2);
        let own = &found[0];
        assert_eq!((own.partition.as_str(), own.mode, own.shares_linux), ("/dev/nvme0n1p3", DiskMode::Whole, false));
        assert_eq!(own.disk_model, "CT1000P3PSSD8");
        assert_eq!(own.mounts, ["/mnt/c"]);
        let shared = &found[1];
        assert_eq!((shared.mode, shared.bitlocker), (DiskMode::Partition, true));
        let esp = shared.esp.as_ref().expect("esp on the shared drive");
        assert_eq!((esp.part_uuid.as_str(), esp.mounts.as_slice()), ("e1", ["/boot".to_owned()].as_slice()));
    }

    #[test]
    fn recovery_partitions_are_not_windows_installs() {
        assert!(installs(&sample(), &|_| None).iter().all(|install| install.part_uuid != "r0"));
    }
}
