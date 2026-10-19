use crate::config::Gpu;
use anyhow::{Result, bail};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

/// Which Limine entry the host booted: `gpu` hands the GPU to vfio-pci for Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Normal,
    Gpu,
    Unknown,
}

impl Profile {
    pub fn name(self) -> &'static str {
        match self {
            Profile::Normal => "normal",
            Profile::Gpu => "gpu",
            Profile::Unknown => "unknown",
        }
    }
}

pub fn boot_profile(gpu: Option<&Gpu>) -> Profile {
    fs::read_to_string("/proc/cmdline").map_or(Profile::Unknown, |cmdline| profile_from_cmdline(&cmdline, gpu))
}

fn profile_from_cmdline(cmdline: &str, gpu: Option<&Gpu>) -> Profile {
    if cmdline.trim().is_empty() {
        return Profile::Unknown;
    }
    let Some(gpu) = gpu else { return Profile::Normal };
    let claimed: BTreeSet<&str> = cmdline
        .split_whitespace()
        .filter_map(|token| token.strip_prefix("vfio-pci.ids="))
        .flat_map(|ids| ids.split(','))
        .collect();
    if !gpu.ids.is_empty() && gpu.ids.iter().all(|id| claimed.contains(id.as_str())) {
        Profile::Gpu
    } else {
        Profile::Normal
    }
}

fn pci_device(address: &str) -> std::path::PathBuf {
    Path::new("/sys/bus/pci/devices").join(address)
}

pub fn pci_driver(address: &str) -> Option<String> {
    let target = fs::read_link(pci_device(address).join("driver")).ok()?;
    target.file_name().map(|name| name.to_string_lossy().into_owned())
}

/// The GPU can only be passed through when its IOMMU group holds nothing else.
pub fn verify_gpu_topology(gpu: &Gpu) -> Result<()> {
    let group = pci_device(&gpu.address).join("iommu_group/devices");
    let Ok(entries) = fs::read_dir(&group) else { bail!("The GPU does not have an available IOMMU group.") };
    let members: BTreeSet<String> = entries.flatten().map(|entry| entry.file_name().to_string_lossy().into_owned()).collect();
    let expected: BTreeSet<String> = [gpu.address.clone(), gpu.audio_address.clone()].into();
    if members != expected {
        bail!("The GPU IOMMU group is not isolated to its graphics and audio functions.");
    }
    if !pci_device(&gpu.address).join("reset").exists() {
        bail!("The GPU does not expose a PCI reset interface.");
    }
    Ok(())
}

pub fn verify_gpu_claimed(gpu: &Gpu) -> Result<()> {
    for (address, function) in [(&gpu.address, "graphics"), (&gpu.audio_address, "audio")] {
        if pci_driver(address).as_deref() != Some("vfio-pci") {
            bail!("The GPU boot profile is active, but the GPU {function} function is not bound to vfio-pci.");
        }
    }
    Ok(())
}

/// Hostdev XML that libvirt attaches for one PCI function, e.g. `0000:01:00.0`.
pub fn hostdev_xml(address: &str) -> Result<String> {
    let parts: Vec<&str> = address.split([':', '.']).collect();
    let [domain, bus, slot, function] = parts.as_slice() else { bail!("Invalid PCI address: {address}") };
    Ok(format!(
        "<hostdev mode=\"subsystem\" type=\"pci\" managed=\"yes\">\n  <source>\n    \
         <address domain=\"0x{domain}\" bus=\"0x{bus}\" slot=\"0x{slot}\" function=\"0x{function}\"/>\n  \
         </source>\n</hostdev>\n"
    ))
}

/// Limine entry titles look like `Omarchy/linux-omarchy`; nothing else reaches the root edit.
fn valid_entry(entry: &str) -> bool {
    !entry.is_empty() && entry.chars().all(|c| c.is_ascii_alphanumeric() || " /._-()".contains(c))
}

/// Root side of `ws reboot`: makes `entry` the default Limine entry, then reboots.
pub fn reboot_into(entry: &str) -> Result<()> {
    if !valid_entry(entry) {
        bail!("Invalid boot entry: {entry}");
    }
    let path = Path::new("/boot/limine.conf");
    let config = fs::read_to_string(path).map_err(|_| anyhow::anyhow!("Limine configuration is missing at /boot/limine.conf."))?;
    let updated = set_default_entry(&config, entry);
    fs::write(path, &updated)?;
    if !fs::read_to_string(path)?.lines().any(|line| line == format!("default_entry: {entry}")) {
        bail!("Could not select the boot entry {entry}.");
    }
    crate::process::text("systemctl", &["reboot"]).map(drop)
}

fn set_default_entry(config: &str, entry: &str) -> String {
    let line = format!("default_entry: {entry}");
    let mut found = false;
    let mut lines: Vec<String> = config
        .lines()
        .map(|current| {
            if current.starts_with("default_entry:") {
                found = true;
                line.clone()
            } else {
                current.to_owned()
            }
        })
        .collect();
    if !found {
        lines.insert(0, line);
    }
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rtx() -> Gpu {
        Gpu {
            address: "0000:01:00.0".into(),
            audio_address: "0000:01:00.1".into(),
            ids: vec!["10de:28a0".into(), "10de:22be".into()],
        }
    }

    #[test]
    fn gpu_profile_needs_every_gpu_function_claimed() {
        let gpu = rtx();
        assert_eq!(profile_from_cmdline("root=x vfio-pci.ids=10de:28a0,10de:22be quiet", Some(&gpu)), Profile::Gpu);
        assert_eq!(profile_from_cmdline("root=x vfio-pci.ids=10de:28a0", Some(&gpu)), Profile::Normal);
        assert_eq!(profile_from_cmdline("root=x", None), Profile::Normal);
        assert_eq!(profile_from_cmdline("", Some(&gpu)), Profile::Unknown);
    }

    #[test]
    fn default_entry_is_replaced_or_added() {
        assert_eq!(set_default_entry("timeout: 3\ndefault_entry: A/b\n", "C/d"), "timeout: 3\ndefault_entry: C/d\n");
        assert_eq!(set_default_entry("timeout: 3\n", "C/d"), "default_entry: C/d\ntimeout: 3\n");
        assert!(!valid_entry("A/b; rm -rf /"));
        assert!(valid_entry("Omarchy/window-side-gpu"));
    }

    #[test]
    fn hostdev_xml_splits_the_pci_address() {
        let xml = hostdev_xml("0000:01:00.1").expect("valid address");
        assert!(xml.contains(r#"domain="0x0000" bus="0x01" slot="0x00" function="0x1""#));
        assert!(hostdev_xml("01:00").is_err());
    }
}
