//! CPU, memory, boot loader and tool facts that shape the VM and the GPU boot profile.
use super::distro::{self, Family};
use crate::display::hyprland;
use crate::process::{has_command, text};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct System {
    pub cpus: Vec<u32>,
    /// Intel hybrid CPUs: performance and efficiency threads.
    pub performance_cpus: Vec<u32>,
    pub efficiency_cpus: Vec<u32>,
    pub threads_per_core: u32,
    pub memory_mib: u64,
    pub iommu_enabled: bool,
    pub iommu_flag: &'static str,
    /// Kernel package of the running kernel, when the boot loader can get a GPU entry.
    pub limine_kernel: Option<String>,
    pub refresh_hz: String,
    pub user: String,
    pub omarchy: bool,
    /// Package family from os-release; `None` when ws cannot install packages here.
    pub distro: Option<Family>,
    pub missing: Vec<Requirement>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Requirement {
    pub command: &'static str,
    /// Empty on distros ws does not know.
    pub packages: &'static [&'static str],
}

/// Parses kernel CPU lists such as `0-3,8,10-11`.
pub fn cpu_list(list: &str) -> Vec<u32> {
    list.trim()
        .split(',')
        .filter(|part| !part.is_empty())
        .flat_map(|part| match part.split_once('-') {
            Some((start, end)) => match (start.parse::<u32>(), end.parse::<u32>()) {
                (Ok(start), Ok(end)) => (start..=end).collect(),
                _ => Vec::new(),
            },
            None => part.parse().into_iter().collect(),
        })
        .collect()
}

fn read_list(path: &str) -> Vec<u32> {
    fs::read_to_string(path).map(|list| cpu_list(&list)).unwrap_or_default()
}

fn memory_mib() -> u64 {
    let meminfo = fs::read_to_string("/proc/meminfo").unwrap_or_default();
    meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))
        .and_then(|value| value.split_whitespace().next()?.parse::<u64>().ok())
        .map_or(0, |kib| kib / 1024)
}

fn limine_kernel() -> Option<String> {
    // /boot is root-only on Omarchy, so look for Limine's readable config instead.
    if !has_command("limine-entry-tool") || !Path::new("/etc/default/limine").exists() {
        return None;
    }
    let release = fs::read_to_string("/proc/sys/kernel/osrelease").ok()?;
    let vmlinuz = format!("/usr/lib/modules/{}/vmlinuz", release.trim());
    // Limine GPU profiles use mkinitcpio, so the kernel comes from pacman.
    text("pacman", &["-Qoq", &vmlinuz]).ok().map(|package| package.trim().to_owned()).filter(|package| !package.is_empty())
}

pub fn detect() -> System {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let family = distro::detect();
    let siblings = read_list("/sys/devices/system/cpu/cpu0/topology/thread_siblings_list");
    System {
        cpus: read_list("/sys/devices/system/cpu/online"),
        performance_cpus: read_list("/sys/devices/cpu_core/cpus"),
        efficiency_cpus: read_list("/sys/devices/cpu_atom/cpus"),
        threads_per_core: u32::try_from(siblings.len()).unwrap_or(1).max(1),
        memory_mib: memory_mib(),
        iommu_enabled: fs::read_dir("/sys/kernel/iommu_groups").is_ok_and(|mut groups| groups.next().is_some()),
        iommu_flag: if cpuinfo.contains("AuthenticAMD") { "amd_iommu=on" } else { "intel_iommu=on" },
        limine_kernel: limine_kernel(),
        refresh_hz: hyprland::refresh_rate().unwrap_or_else(|| "60".to_owned()),
        user: std::env::var("USER").unwrap_or_default(),
        omarchy: has_command("omarchy"),
        distro: family,
        missing: distro::commands()
            .filter(|command| !has_command(command))
            .map(|command| Requirement { command, packages: family.map_or(&[], |family| family.packages(command)) })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_lists_expand_ranges_and_singles() {
        assert_eq!(cpu_list("0-3,8,10-11\n"), [0, 1, 2, 3, 8, 10, 11]);
        assert_eq!(cpu_list(""), Vec::<u32>::new());
    }
}
