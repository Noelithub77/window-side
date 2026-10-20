//! Distro families from `/etc/os-release`: packages, firmware and libvirt services for each.
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Family {
    Arch,
    Debian,
    Fedora,
}

/// Commands ws calls and the packages that provide them: Arch, Ubuntu/Debian, Fedora.
const REQUIREMENTS: &[(&str, [&[&str]; 3])] = &[
    ("virsh", [&["libvirt"], &["libvirt-daemon-system", "libvirt-clients"], &["libvirt"]]),
    ("qemu-system-x86_64", [&["qemu-base"], &["qemu-system-x86"], &["qemu-kvm"]]),
    ("swtpm", [&["swtpm"], &["swtpm"], &["swtpm"]]),
    ("swtpm_setup", [&["swtpm"], &["swtpm-tools"], &["swtpm-tools"]]),
    ("dnsmasq", [&["dnsmasq"], &["dnsmasq-base"], &["dnsmasq"]]),
    ("virt-viewer", [&["virt-viewer"], &["virt-viewer"], &["virt-viewer"]]),
    ("secret-tool", [&["libsecret"], &["libsecret-tools"], &["libsecret"]]),
    ("xorriso", [&["libisoburn"], &["xorriso"], &["xorriso"]]),
    ("bsdtar", [&["libarchive"], &["libarchive-tools"], &["bsdtar"]]),
    ("udisksctl", [&["udisks2"], &["udisks2"], &["udisks2"]]),
    ("mkfs.fat", [&["dosfstools"], &["dosfstools"], &["dosfstools"]]),
    ("dmsetup", [&["device-mapper"], &["dmsetup"], &["device-mapper"]]),
    ("lspci", [&["pciutils"], &["pciutils"], &["pciutils"]]),
];

/// UEFI firmware folders and their packages; libvirt picks the image through its firmware descriptors.
const FIRMWARE: [(&str, &str); 3] =
    [("/usr/share/edk2/x64", "edk2-ovmf"), ("/usr/share/OVMF", "ovmf"), ("/usr/share/edk2/ovmf", "edk2-ovmf")];

pub fn parse(os_release: &str) -> Option<Family> {
    let ids: Vec<&str> = os_release
        .lines()
        .filter_map(|line| line.strip_prefix("ID=").or_else(|| line.strip_prefix("ID_LIKE=")))
        .flat_map(|value| value.trim_matches('"').split_whitespace())
        .collect();
    // ID comes first in the file, so a distro's own family wins over what it is like.
    ids.iter().find_map(|id| match *id {
        "arch" => Some(Family::Arch),
        "debian" | "ubuntu" => Some(Family::Debian),
        "fedora" | "rhel" | "centos" => Some(Family::Fedora),
        _ => None,
    })
}

pub fn detect() -> Option<Family> {
    ["/etc/os-release", "/usr/lib/os-release"]
        .iter()
        .find_map(|path| fs::read_to_string(path).ok())
        .and_then(|text| parse(&text))
}

impl Family {
    fn index(self) -> usize {
        match self {
            Self::Arch => 0,
            Self::Debian => 1,
            Self::Fedora => 2,
        }
    }

    /// Packages providing `command`.
    pub fn packages(self, command: &str) -> &'static [&'static str] {
        REQUIREMENTS
            .iter()
            .find(|(name, _)| *name == command)
            .map_or(&[], |(_, packages)| packages[self.index()])
    }

    /// Packages this family may install; the root side refuses anything else.
    pub fn known(self, package: &str) -> bool {
        REQUIREMENTS.iter().any(|(_, packages)| packages[self.index()].contains(&package)) || self.firmware_package() == package
    }

    pub fn firmware_present(self) -> bool {
        Path::new(FIRMWARE[self.index()].0).exists()
    }

    pub fn firmware_package(self) -> &'static str {
        FIRMWARE[self.index()].1
    }

    /// Program and arguments that install packages without prompting.
    pub fn installer(self) -> (&'static str, &'static [&'static str]) {
        match self {
            Self::Arch => ("pacman", &["-S", "--needed", "--noconfirm"]),
            Self::Debian => ("apt-get", &["install", "-y", "--no-install-recommends"]),
            Self::Fedora => ("dnf", &["install", "-y"]),
        }
    }

    /// Fedora runs libvirt as per-driver socket-activated daemons.
    pub fn libvirt_units(self) -> &'static [&'static str] {
        match self {
            Self::Arch | Self::Debian => &["libvirtd.service"],
            Self::Fedora => &["virtqemud.socket", "virtnetworkd.socket", "virtstoraged.socket", "virtnodedevd.socket", "virtsecretd.socket"],
        }
    }
}

pub fn commands() -> impl Iterator<Item = &'static str> {
    REQUIREMENTS.iter().map(|(command, _)| *command)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn families_follow_id_then_id_like() {
        assert_eq!(parse("NAME=\"Ubuntu\"\nID=ubuntu\nID_LIKE=debian\n"), Some(Family::Debian));
        assert_eq!(parse("ID=neon\nID_LIKE=\"ubuntu debian\"\n"), Some(Family::Debian));
        assert_eq!(parse("ID=fedora\n"), Some(Family::Fedora));
        assert_eq!(parse("ID=arch\n"), Some(Family::Arch));
        assert_eq!(parse("ID=endeavouros\nID_LIKE=arch\n"), Some(Family::Arch));
        assert_eq!(parse("ID=opensuse-tumbleweed\nID_LIKE=\"opensuse suse\"\n"), None);
    }

    #[test]
    fn every_family_has_packages_for_every_command() {
        for family in [Family::Arch, Family::Debian, Family::Fedora] {
            for command in commands() {
                assert!(!family.packages(command).is_empty(), "{family:?} lacks {command}");
                assert!(family.packages(command).iter().all(|package| family.known(package)));
            }
            assert!(family.known(family.firmware_package()));
            assert!(!family.known("sudo"));
        }
        assert!(!Family::Fedora.known("qemu-base"));
    }
}
