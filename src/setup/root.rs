//! Root side of setup, run once through pkexec. It only writes allowlisted files,
//! installs known packages and adds the invoking user to the VM groups.
use crate::detect::distro::Family;
use crate::process::{output, text, text_with_input};
use crate::setup::files::{self, GPU_ENTRY, SystemFile};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub const STATE: &str = "/var/lib/window-side";
pub const ESP_START: u64 = 2048;
pub const ESP_SECTORS: u64 = 260 * 2048;
pub const HEADER_SECTORS: u64 = ESP_START + ESP_SECTORS;
pub const FOOTER_SECTORS: u64 = 2048;
const ESP_TYPE: &str = "C12A7328-F81F-11D2-BA4B-00A0C93EC93B";
const BASIC_DATA_TYPE: &str = "EBD0A0A2-B9E5-4433-87C0-68B6B72699C7";

const GROUPS: &[&str] = &["libvirt", "kvm"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VirtualDisk {
    pub disk_guid: String,
    pub esp_part_uuid: String,
    pub windows_part_uuid: String,
    pub windows_sectors: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootPlan {
    pub user: String,
    pub distro: Option<Family>,
    pub packages: Vec<String>,
    pub groups: Vec<String>,
    pub files: Vec<SystemFile>,
    pub virtual_disk: Option<VirtualDisk>,
    /// Kernel package whose image gets a GPU passthrough twin in Limine.
    pub gpu_kernel: Option<String>,
    pub boot_profile: Option<crate::detect::boot::BootProfile>,
}

impl RootPlan {
    /// Rejects anything outside what setup can legitimately ask for.
    pub fn validate(&self) -> Result<()> {
        if let Some(file) = self.files.iter().find(|file| !files::allowed_path(&file.path, self.boot_profile.as_ref())) {
            bail!("Refusing to write {}", file.path);
        }
        if let Some(boot) = &self.boot_profile {
            super::boot::validate(boot)?;
            if self.gpu_kernel.is_some() != (boot.loader == crate::detect::boot::Loader::Limine) {
                bail!("Inconsistent boot profile");
            }
        }
        if self.files.iter().any(|file| !matches!(file.mode, 0o644 | 0o755)) { bail!("Invalid system file mode"); }
        if let Some(package) = self.packages.iter().find(|package| !self.distro.is_some_and(|family| family.known(package))) {
            bail!("Refusing to install {package}");
        }
        if let Some(group) = self.groups.iter().find(|group| !GROUPS.contains(&group.as_str())) {
            bail!("Refusing to add the user to {group}");
        }
        let safe = |value: &str| !value.is_empty() && value.chars().all(|c| c.is_ascii_alphanumeric() || "-_.".contains(c));
        if !safe(&self.user) || self.gpu_kernel.as_deref().is_some_and(|kernel| !safe(kernel)) {
            bail!("Invalid user or kernel name");
        }
        if let Some(disk) = &self.virtual_disk {
            let guids = [&disk.disk_guid, &disk.esp_part_uuid, &disk.windows_part_uuid];
            if guids.iter().any(|guid| !guid.chars().all(|c| c.is_ascii_hexdigit() || c == '-') || guid.len() != 36) {
                bail!("Invalid partition identifiers");
            }
        }
        Ok(())
    }
}

/// pkexec records who asked; the plan may only act for that user.
fn verify_caller(user: &str) -> Result<()> {
    let caller = std::env::var("PKEXEC_UID").context("Run setup through ws setup apply")?;
    let uid = text("id", &["-u", user])?;
    if uid.trim() != caller {
        bail!("The setup plan belongs to another user");
    }
    Ok(())
}

pub fn apply(plan_path: &Path) -> Result<()> {
    let plan: RootPlan = serde_json::from_str(&fs::read_to_string(plan_path)?).context("Invalid setup plan")?;
    plan.validate()?;
    verify_caller(&plan.user)?;
    if let Some(boot) = &plan.boot_profile
        && crate::detect::boot::detect().as_ref() != Some(boot) {
            bail!("Boot configuration changed or is unreadable; create a new setup plan");
    }
    if let Some(family) = plan.distro.filter(|_| !plan.packages.is_empty()) {
        install(family, &plan.packages)?;
    }
    for file in &plan.files {
        write(file)?;
    }
    if !plan.groups.is_empty() {
        text("usermod", &["-aG", &plan.groups.join(","), &plan.user])?;
    }
    let units = plan.distro.map_or(&["libvirtd.service"][..], Family::libvirt_units);
    text("systemctl", &[&["enable", "--now"][..], units].concat()).context("Could not start libvirt")?;
    text("systemctl", &["daemon-reload"])?;
    if let Some(disk) = &plan.virtual_disk {
        let _ = output("systemctl", &["stop", "window-side-disk.service"]);
        build_virtual_disk(disk)?;
        text("systemctl", &["enable", "--now", "window-side-disk.service"]).context("Could not assemble the virtual disk")?;
    }
    let _ = output("udevadm", &["control", "--reload"]);
    let _ = output("udevadm", &["trigger", "--subsystem-match=block"]);
    let _ = output("systemd-tmpfiles", &["--create", files::TMPFILES_CONF]);
    let entries = match &plan.gpu_kernel {
        Some(kernel) => Some(add_gpu_entry(kernel)?),
        None => plan.boot_profile.as_ref().map(super::boot::apply).transpose()?,
    };
    println!("{}", json!({"normalEntry": entries.as_ref().map(|e| &e.0), "gpuEntry": entries.as_ref().map(|e| &e.1)}));
    Ok(())
}

fn install(family: Family, packages: &[String]) -> Result<()> {
    let (program, flags) = family.installer();
    let args: Vec<&str> = flags.iter().copied().chain(packages.iter().map(String::as_str)).collect();
    let mut result = text(program, &args);
    // A fresh Ubuntu install may have no package lists yet.
    if result.is_err() && family == Family::Debian {
        text("apt-get", &["update"]).context("Could not refresh the package lists")?;
        result = text(program, &args);
    }
    result.context("Could not install the required packages")?;
    Ok(())
}

fn write(file: &SystemFile) -> Result<()> {
    let path = Path::new(&file.path);
    if path.ancestors().any(|parent| parent.symlink_metadata().is_ok_and(|meta| meta.file_type().is_symlink())) {
        bail!("Refusing to write through a symlink: {}", file.path);
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, &file.contents).with_context(|| format!("Could not write {}", file.path))?;
    fs::set_permissions(path, fs::Permissions::from_mode(file.mode))?;
    Ok(())
}

/// Rebuilds the initramfs with vfio, adds the GPU twin of the kernel entry and returns both entry paths.
fn add_gpu_entry(kernel: &str) -> Result<(String, String)> {
    text("limine-mkinitcpio", &[]).context("Could not rebuild the boot images")?;
    let uki = fs::read_dir("/boot/EFI/Linux")?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| path.file_name().is_some_and(|name| name.to_string_lossy().ends_with(&format!("_{kernel}.efi"))))
        .context("No unified kernel image found for the running kernel")?;
    let comment = format!("{kernel} with GPU passthrough");
    text("limine-entry-tool", &["--add-uki", GPU_ENTRY, &uki.display().to_string(), "--comment", &comment])?;
    let config = fs::read_to_string("/boot/limine.conf")?;
    let normal = entry_path(&config, kernel).context("The kernel entry is missing from limine.conf")?;
    let gpu = entry_path(&config, GPU_ENTRY).context("The GPU entry is missing from limine.conf")?;
    Ok((normal, gpu))
}

/// `Parent/child` path of a second-level Limine entry, as `default_entry` expects.
pub fn entry_path(config: &str, name: &str) -> Option<String> {
    let mut parent = None;
    for line in config.lines().map(str::trim) {
        let depth = line.chars().take_while(|c| *c == '/').count();
        let title = line[depth..].trim_start_matches('+').trim();
        match depth {
            1 => parent = Some(title.to_owned()),
            2 if title == name => return parent.map(|parent| format!("{parent}/{title}")),
            _ => {}
        }
    }
    None
}

/// Writes a GPT whose disk and partition GUIDs match the real drive, so Windows' boot configuration
/// finds its partition, then copies the Windows boot files into a private ESP.
fn build_virtual_disk(disk: &VirtualDisk) -> Result<()> {
    fs::create_dir_all(STATE)?;
    let layout = Path::new(STATE).join("layout.img");
    let total = HEADER_SECTORS + disk.windows_sectors + FOOTER_SECTORS;
    fs::File::create(&layout)?.set_len(total * 512)?;
    let table = format!(
        "label: gpt\nlabel-id: {guid}\nunit: sectors\nsector-size: 512\n\
         start={ESP_START}, size={ESP_SECTORS}, type={ESP_TYPE}, uuid={esp}, name=\"EFI system partition\"\n\
         start={HEADER_SECTORS}, size={windows}, type={BASIC_DATA_TYPE}, uuid={win}, name=\"Basic data partition\"\n",
        guid = disk.disk_guid, esp = disk.esp_part_uuid, windows = disk.windows_sectors, win = disk.windows_part_uuid,
    );
    text_with_input("sfdisk", &["--quiet", "--no-reread", "--no-tell-kernel", &layout.display().to_string()], &table)
        .context("Could not write the virtual partition table")?;
    let header = Path::new(STATE).join("header.img");
    let footer = Path::new(STATE).join("footer.img");
    copy_range(&layout, &header, 0, HEADER_SECTORS * 512)?;
    copy_range(&layout, &footer, (total - FOOTER_SECTORS) * 512, FOOTER_SECTORS * 512)?;
    fs::remove_file(&layout)?;
    let header_path = header.display().to_string();
    text("mkfs.fat", &["-F", "32", "-n", "SYSTEM", "--offset", &ESP_START.to_string(), &header_path, &(ESP_SECTORS / 2).to_string()])?;
    copy_boot_files(&header_path, &disk.esp_part_uuid)
}

fn copy_range(source: &Path, target: &Path, start: u64, length: u64) -> Result<()> {
    let mut input = fs::File::open(source)?;
    input.seek(SeekFrom::Start(start))?;
    let mut out = fs::File::create(target)?;
    std::io::copy(&mut input.take(length), &mut out)?;
    out.flush()?;
    Ok(())
}

fn copy_boot_files(header: &str, esp_part_uuid: &str) -> Result<()> {
    let target = Path::new("/run/window-side-esp");
    let source = Path::new("/run/window-side-source");
    fs::create_dir_all(target)?;
    fs::create_dir_all(source)?;
    let offset = format!("loop,offset={},sizelimit={}", ESP_START * 512, ESP_SECTORS * 512);
    text("mount", &["-o", &offset, header, &target.display().to_string()])?;
    let device = format!("PARTUUID={esp_part_uuid}");
    // On a shared drive the ESP is usually already mounted at /boot; read it there.
    let mounted = text("findmnt", &["-n", "-o", "TARGET", "-S", &device]).ok().and_then(|targets| targets.lines().next().map(str::to_owned));
    let result = match mounted {
        Some(mountpoint) => copy_windows_boot(Path::new(&mountpoint), target),
        None => text("mount", &["-o", "ro", &device, &source.display().to_string()])
            .context("Could not read the Windows EFI partition")
            .and_then(|_| {
                let copied = copy_windows_boot(source, target);
                let _ = output("umount", &[&source.display().to_string()]);
                copied
            }),
    };
    let _ = output("umount", &[&target.display().to_string()]);
    result
}

fn copy_windows_boot(source: &Path, target: &Path) -> Result<()> {
    let microsoft = source.join("EFI/Microsoft");
    let manager = microsoft.join("Boot/bootmgfw.efi");
    if !manager.exists() {
        bail!("The EFI partition has no Windows boot manager");
    }
    copy_tree(&microsoft, &target.join("EFI/Microsoft"))?;
    // Firmware without boot entries starts the fallback path.
    fs::create_dir_all(target.join("EFI/Boot"))?;
    fs::copy(&manager, target.join("EFI/Boot/bootx64.efi"))?;
    Ok(())
}

fn copy_tree(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target)?;
    for entry in fs::read_dir(source)?.flatten() {
        let path = entry.path();
        let destination = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&path, &destination)?;
        } else {
            fs::copy(&path, &destination)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_paths_follow_limine_nesting() {
        let config = "timeout: 3\n/+Omarchy\n  comment: x\n  //linux-omarchy\n  //window-side-gpu\n/Windows\n";
        assert_eq!(entry_path(config, "linux-omarchy").as_deref(), Some("Omarchy/linux-omarchy"));
        assert_eq!(entry_path(config, "window-side-gpu").as_deref(), Some("Omarchy/window-side-gpu"));
        assert_eq!(entry_path(config, "missing"), None);
    }

    #[test]
    fn plans_outside_the_allowlist_are_rejected() {
        let plan = RootPlan {
            user: "noel".into(), distro: Some(Family::Arch), packages: vec!["libvirt".into()], groups: vec!["libvirt".into()],
            files: vec![SystemFile { path: files::TMPFILES_CONF.into(), contents: String::new(), mode: 0o644 }],
            virtual_disk: None, gpu_kernel: Some("linux-omarchy".into()), boot_profile: None,
        };
        assert!(plan.validate().is_ok());
        let bad_file = RootPlan { files: vec![SystemFile { path: "/etc/sudoers".into(), contents: String::new(), mode: 0o644 }], ..plan.clone() };
        assert!(bad_file.validate().is_err());
        assert!(RootPlan { packages: vec!["evil".into()], ..plan.clone() }.validate().is_err());
        assert!(RootPlan { packages: vec!["qemu-system-x86".into()], ..plan.clone() }.validate().is_err());
        assert!(RootPlan { distro: Some(Family::Debian), packages: vec!["qemu-system-x86".into()], ..plan.clone() }.validate().is_ok());
        assert!(RootPlan { distro: None, ..plan.clone() }.validate().is_err());
        assert!(RootPlan { groups: vec!["wheel".into()], ..plan.clone() }.validate().is_err());
        assert!(RootPlan { user: "x; rm".into(), ..plan }.validate().is_err());
    }
}
