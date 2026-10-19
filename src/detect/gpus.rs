//! Finds GPUs that can be handed to Windows: discrete, not driving the host display, isolated.
use crate::process::{has_command, output};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    pub address: String,
    pub name: String,
    /// `vendor:device` of every function vfio-pci must claim, graphics first.
    pub ids: Vec<String>,
    pub audio_address: Option<String>,
    pub driver: Option<String>,
    /// Why passthrough is unavailable; `None` when it is ready.
    pub blocker: Option<Blocker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Blocker {
    /// Linux draws its own screen with this GPU.
    HostDisplay,
    /// IOMMU is off; setup turns it on and the GPU is rechecked after a reboot.
    IommuOff,
    /// Other devices share its IOMMU group and would have to go with it.
    SharedGroup,
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).map(|value| value.trim().to_owned()).unwrap_or_default()
}

fn id(device: &Path) -> String {
    let strip = |name: &str| read(&device.join(name)).trim_start_matches("0x").to_owned();
    format!("{}:{}", strip("vendor"), strip("device"))
}

fn name(address: &str) -> Option<String> {
    if !has_command("lspci") {
        return None;
    }
    let result = output("lspci", &["-mm", "-s", address]).ok()?;
    // Fields: slot "class" "vendor" "device" ...
    let line = String::from_utf8_lossy(&result.stdout).into_owned();
    let fields: Vec<&str> = line.split('"').filter(|field| !field.trim().is_empty()).collect();
    let vendor = fields.get(2)?.replace(" Corporation", "");
    Some(format!("{vendor} {}", fields.get(3)?))
}

pub fn detect() -> Vec<Gpu> {
    scan(Path::new("/sys/bus/pci/devices"), Path::new("/sys/kernel/iommu_groups"))
        .into_iter()
        .map(|gpu| Gpu { name: name(&gpu.address).unwrap_or(gpu.name), ..gpu })
        .collect()
}

fn scan(devices: &Path, groups: &Path) -> Vec<Gpu> {
    let iommu = fs::read_dir(groups).is_ok_and(|mut entries| entries.next().is_some());
    let Ok(entries) = fs::read_dir(devices) else { return Vec::new() };
    let mut gpus: Vec<Gpu> = entries
        .flatten()
        .filter(|entry| read(&entry.path().join("class")).starts_with("0x03"))
        .map(|entry| gpu(devices, &entry.path(), iommu))
        .collect();
    gpus.sort_by(|a, b| a.address.cmp(&b.address));
    gpus
}

fn gpu(devices: &Path, device: &Path, iommu: bool) -> Gpu {
    let address = device.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    // Sibling functions share everything before the final `.function`.
    let slot = address.rsplit_once('.').map_or(address.as_str(), |(slot, _)| slot).to_owned();
    let siblings: Vec<String> = fs::read_dir(devices)
        .map(|entries| {
            let mut found: Vec<String> = entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.starts_with(&format!("{slot}.")))
                .collect();
            found.sort();
            found
        })
        .unwrap_or_default();
    let audio_address = siblings.iter().find(|name| read(&devices.join(name).join("class")).starts_with("0x0403")).cloned();
    let functions: Vec<&String> = std::iter::once(&address).chain(siblings.iter().filter(|name| **name != address)).collect();
    let group: Vec<String> = fs::read_dir(device.join("iommu_group/devices"))
        .map(|entries| entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    // PCIe bridges may share the group; vfio leaves them to the host.
    let isolated = group.iter().all(|member| siblings.contains(member) || read(&devices.join(member).join("class")).starts_with("0x0604"));
    let blocker = if read(&device.join("boot_vga")) == "1" {
        Some(Blocker::HostDisplay)
    } else if !iommu {
        Some(Blocker::IommuOff)
    } else if !isolated {
        Some(Blocker::SharedGroup)
    } else {
        None
    };
    let driver = fs::read_link(device.join("driver")).ok().and_then(|link| link.file_name().map(|name| name.to_string_lossy().into_owned()));
    Gpu {
        name: id(device),
        ids: functions.iter().map(|function| id(&devices.join(function))).collect(),
        address,
        audio_address,
        driver,
        blocker,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    fn device(root: &Path, address: &str, class: &str, vendor: &str, product: &str, group: &str) {
        let path = root.join("devices").join(address);
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("class"), class).unwrap();
        fs::write(path.join("vendor"), vendor).unwrap();
        fs::write(path.join("device"), product).unwrap();
        let members = root.join("groups").join(group).join("devices");
        fs::create_dir_all(&members).unwrap();
        symlink(&path, members.join(address)).unwrap();
        symlink(root.join("groups").join(group), path.join("iommu_group")).unwrap();
    }

    #[test]
    fn laptop_dgpu_is_ready_and_igpu_drives_the_host() {
        let root = std::env::temp_dir().join(format!("window-side-gpus-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        device(&root, "0000:00:02.0", "0x030000", "0x8086", "0xa7a8", "0");
        fs::write(root.join("devices/0000:00:02.0/boot_vga"), "1").unwrap();
        device(&root, "0000:00:01.0", "0x060400", "0x8086", "0xa70d", "15");
        device(&root, "0000:01:00.0", "0x030000", "0x10de", "0x28a0", "15");
        device(&root, "0000:01:00.1", "0x040300", "0x10de", "0x22be", "15");
        device(&root, "0000:02:00.0", "0x030000", "0x1002", "0x73df", "16");
        device(&root, "0000:03:00.0", "0x020000", "0x8086", "0x1533", "16");

        let gpus = scan(&root.join("devices"), &root.join("groups"));
        fs::remove_dir_all(&root).unwrap();
        let find = |address: &str| gpus.iter().find(|gpu| gpu.address == address).unwrap();
        assert_eq!(find("0000:00:02.0").blocker, Some(Blocker::HostDisplay));
        let rtx = find("0000:01:00.0");
        assert_eq!(rtx.blocker, None);
        assert_eq!(rtx.ids, ["10de:28a0", "10de:22be"]);
        assert_eq!(rtx.audio_address.as_deref(), Some("0000:01:00.1"));
        assert_eq!(find("0000:02:00.0").blocker, Some(Blocker::SharedGroup));
    }
}
