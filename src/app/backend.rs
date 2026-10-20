use super::types::*;
use crate::{
    agents,
    config::{Config, DiskMode as CoreDiskMode},
    detect, display, setup,
};
use anyhow::Result;

pub fn detection() -> Result<Detection> {
    let inventory = setup::detect()?;
    let config = Config::load().ok();
    let s = &inventory.system;
    let choices = inventory.default_choices(config.as_ref()).map(Into::into);
    let windows = inventory
        .windows
        .iter()
        .map(|w| WindowsInstall {
            partition: w.partition.clone(),
            disk_model: w.disk_model.clone(),
            size: w.size as f64,
            disk_size: w.disk_size as f64,
            dedicated_drive: w.mode == CoreDiskMode::Whole,
        })
        .collect();
    let gpus = inventory
        .gpus
        .iter()
        .map(|g| {
            let vendor = match g.ids.first().and_then(|id| id.split(':').next()) {
                Some("10de") => Vendor::Nvidia,
                Some("1002") => Vendor::Amd,
                Some("8086") => Vendor::Intel,
                _ => Vendor::Other,
            };
            let blocker = g.blocker.map(|b| match b {
                detect::gpus::Blocker::HostDisplay => GpuBlocker::HostDisplay,
                detect::gpus::Blocker::IommuOff => GpuBlocker::IommuOff,
                detect::gpus::Blocker::SharedGroup => GpuBlocker::SharedGroup,
            });
            GpuOption {
                address: g.address.clone(),
                name: g.name.clone(),
                vendor,
                blocker,
                usable: inventory
                    .usable_gpus()
                    .any(|usable| usable.address == g.address),
            }
        })
        .collect();
    let host = Host {
        threads: s.cpus.len() as u32,
        performance_threads: s.performance_cpus.len() as u32,
        memory_mib: s.memory_mib as f64,
        memory_limit_mib: inventory.memory_limit_mib(),
        vcpu_limit: inventory.vcpu_limit(),
        iommu: s.iommu_enabled,
        missing_packages: s
            .missing
            .iter()
            .flat_map(|m| m.packages.iter().map(|p| (*p).to_owned()))
            .collect(),
        boot_loader: s.boot_profile.as_ref().map(|p| match p.loader {
            detect::boot::Loader::Limine => BootLoader::Limine,
            detect::boot::Loader::Grub => BootLoader::Grub,
            detect::boot::Loader::SystemdBoot => BootLoader::SystemdBoot,
        }),
        distro: s.distro.map(|d| match d {
            detect::distro::Family::Arch => Distro::Arch,
            detect::distro::Family::Debian => Distro::Debian,
            detect::distro::Family::Fedora => Distro::Fedora,
        }),
        desktop: match display::desktop::current() {
            display::desktop::Desktop::Hyprland => Desktop::Hyprland,
            display::desktop::Desktop::Kde => Desktop::Kde,
            display::desktop::Desktop::Gnome => Desktop::Gnome,
            display::desktop::Desktop::Other => Desktop::Other,
        },
        omarchy: s.omarchy,
    };
    Ok(Detection {
        windows,
        gpus,
        host,
        choices,
        configured: config.is_some(),
    })
}

pub fn core_plan(choices: Choices) -> Result<setup::Plan> {
    setup::plan(
        &setup::detect()?,
        &choices.into(),
        Config::load().ok().as_ref(),
    )
}
pub fn plan(choices: Choices) -> Result<Plan> {
    let inventory = setup::detect()?;
    let p = setup::plan(
        &inventory,
        &choices.clone().into(),
        Config::load().ok().as_ref(),
    )?;
    let install = inventory
        .windows
        .iter()
        .find(|w| w.partition == choices.windows)
        .ok_or_else(|| anyhow::anyhow!("Choose a detected Windows install"))?;
    let gpu = choices
        .gpu
        .as_ref()
        .and_then(|address| inventory.gpus.iter().find(|g| &g.address == address));
    Ok(Plan {
        facts: facts(
            &install.disk_model,
            &install.partition,
            install.mode == CoreDiskMode::Whole,
            gpu.map(|g| g.name.as_str()),
            p.config.vm.memory_mib,
            p.config.vm.vcpus,
        ),
        notes: p.notes,
        packages: p.root.packages,
        groups: p.root.groups,
        files: p
            .root
            .files
            .into_iter()
            .map(|f| PlanFile {
                path: f.path,
                contents: f.contents,
            })
            .collect(),
        domain_xml: p.domain_xml,
    })
}

pub(super) fn facts(
    model: &str,
    partition: &str,
    whole: bool,
    gpu: Option<&str>,
    memory_mib: u32,
    vcpus: u32,
) -> Vec<Fact> {
    let name = if model.trim().is_empty() {
        partition
    } else {
        model.trim()
    };
    let mode = if whole {
        "whole drive".to_owned()
    } else {
        let number = partition.trim_end_matches(|c: char| c.is_ascii_digit());
        let number = &partition[number.len()..];
        if number.is_empty() {
            "partition".into()
        } else {
            format!("partition {number}")
        }
    };
    let gib = f64::from(memory_mib) / 1024.;
    [
        ("Windows", format!("{name} · {mode}")),
        ("Graphics", gpu.unwrap_or("Virtual display").to_owned()),
        ("Memory", format!("{gib} GiB")),
        ("Processor", format!("{vcpus} threads")),
    ]
    .into_iter()
    .map(|(label, value)| Fact {
        label: label.into(),
        value,
    })
    .collect()
}

pub fn shell(appearance: Appearance) -> Shell {
    let palette = palette();
    let agents = agents::detect()
        .into_iter()
        .map(|a| Agent {
            id: match a {
                agents::Agent::Claude => AgentId::Claude,
                agents::Agent::Codex => AgentId::Codex,
                agents::Agent::OpenCode => AgentId::Opencode,
            },
            name: a.name().into(),
        })
        .collect();
    let home = crate::config::home();
    Shell {
        panel_installed: panel_installed(&home),
        version: env!("CARGO_PKG_VERSION").into(),
        fixture: false,
        omarchy: crate::config::home().join(".config/omarchy").is_dir(),
        appearance,
        palette,
        agents,
    }
}
fn palette() -> Option<Palette> {
    let path = crate::config::home().join(".config/omarchy/current/theme/colors.toml");
    let colors: toml::Table = toml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let color = |key: &str| -> Option<String> {
        let value = colors.get(key)?.as_str()?;
        let hex = value.strip_prefix('#')?;
        (hex.len() == 6 && hex.bytes().all(|c| c.is_ascii_hexdigit())).then(|| value.into())
    };
    Some(Palette {
        background: color("background")?,
        foreground: color("foreground")?,
        accent: color("accent")?,
        colors: (1..=6)
            .map(|i| color(&format!("color{i}")))
            .collect::<Option<_>>()?,
    })
}

pub fn appearance() -> Appearance {
    let folder = crate::config::config_dir();
    let json = std::fs::read_to_string(folder.join("preferences.json"))
        .ok()
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
    let old = || {
        std::fs::read_to_string(folder.join("appearance.toml"))
            .ok()
            .and_then(|text| toml::from_str::<toml::Table>(&text).ok())
            .and_then(|t| {
                t.get("appearance")
                    .and_then(toml::Value::as_str)
                    .map(str::to_owned)
            })
    };
    if json
        .as_ref()
        .and_then(|v| v["appearance"].as_str())
        .map(str::to_owned)
        .or_else(old)
        .as_deref()
        == Some("omarchy")
    {
        Appearance::Omarchy
    } else {
        Appearance::Pastel
    }
}
pub fn save_appearance(appearance: Appearance) -> Result<()> {
    let folder = crate::config::config_dir();
    std::fs::create_dir_all(&folder)?;
    let path = folder.join("preferences.json");
    let mut value = match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(),
        Err(e) => return Err(e.into()),
    };
    value.insert("appearance".into(), serde_json::to_value(appearance)?);
    let temp = folder.join("preferences.json.tmp");
    std::fs::write(&temp, serde_json::to_string_pretty(&value)? + "\n")?;
    std::fs::rename(temp, path)?;
    Ok(())
}

pub fn action(action: Action) -> Result<()> {
    use anyhow::Context;
    let args = action_args(action);
    let executable = std::env::current_exe().context("Could not find the ws binary")?;
    let output = std::process::Command::new(executable)
        .args(args)
        .stdin(std::process::Stdio::null())
        .output()
        .context("Could not run the Windows action")?;
    anyhow::ensure!(
        output.status.success(),
        "{}",
        super::errors::action_message(&output.stderr, &output.stdout)
    );
    Ok(())
}

fn action_args(action: Action) -> Vec<String> {
    let args: Vec<&str> = match action {
        Action::Launch => vec!["launch"],
        Action::Console => vec!["console"],
        Action::Stop => vec!["stop"],
        Action::ForceStop => vec!["force-stop"],
        Action::Reboot { profile } => vec![
            "reboot",
            match profile {
                BootProfile::Normal => "normal",
                BootProfile::Gpu => "gpu",
            },
        ],
        Action::Resolution { width, height } => {
            return vec![
                "resolution".into(),
                "--width".into(),
                width.to_string(),
                "--height".into(),
                height.to_string(),
                "--json".into(),
            ];
        }
        Action::LookingGlassSetup => vec!["looking-glass-setup"],
        Action::UpdatesCheck => vec!["updates", "check"],
        Action::UpdatesInstall => vec!["updates", "install"],
        Action::SignIn => vec!["sign-in", "run", "--wait", "40"],
        Action::InstallPlugin => vec!["setup", "plugin"],
        Action::FirstBootStart => vec!["setup", "first-boot", "start"],
        Action::FirstBootFinish => vec!["setup", "first-boot", "finish"],
    };
    args.into_iter().map(str::to_owned).collect()
}

fn panel_installed(home: &std::path::Path) -> bool {
    let omarchy = home.join(".config/omarchy");
    omarchy.is_dir()
        && ["manifest.json", "Panel.qml", "Service.qml"]
            .iter()
            .all(|file| {
                omarchy
                    .join("plugins/noel.window-side")
                    .join(file)
                    .is_file()
            })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_use_selected_hardware_and_preserve_fractional_memory() {
        let values = facts(
            " Samsung 990 Pro ",
            "/dev/nvme0n1p3",
            true,
            Some("NVIDIA GeForce RTX 4070"),
            16384,
            8,
        );
        assert_eq!(
            values.iter().map(|f| f.label.as_str()).collect::<Vec<_>>(),
            ["Windows", "Graphics", "Memory", "Processor"]
        );
        assert_eq!(
            values.iter().map(|f| f.value.as_str()).collect::<Vec<_>>(),
            [
                "Samsung 990 Pro · whole drive",
                "NVIDIA GeForce RTX 4070",
                "16 GiB",
                "8 threads"
            ]
        );
        let values = facts("Crucial MX500", "/dev/sda12", false, None, 2560, 4);
        assert_eq!(values[0].value, "Crucial MX500 · partition 12");
        assert_eq!(values[1].value, "Virtual display");
        assert_eq!(values[2].value, "2.5 GiB");
        assert_eq!(
            facts(" ", "/dev/nvme0n1p3", false, None, 2048, 2)[0].value,
            "/dev/nvme0n1p3 · partition 3"
        );
    }

    #[test]
    fn panel_detection_checks_installed_files_and_follows_dotfile_links() -> Result<()> {
        use std::os::unix::fs::symlink;
        let root = std::env::temp_dir().join(format!("ws-panel-test-{}", std::process::id()));
        let home = root.join("home");
        let source = root.join("dotfiles");
        let plugin = home.join(".config/omarchy/plugins/noel.window-side");
        std::fs::create_dir_all(&source)?;
        assert!(!panel_installed(&home));
        std::fs::create_dir_all(
            plugin
                .parent()
                .ok_or_else(|| anyhow::anyhow!("Missing plugin parent"))?,
        )?;
        assert!(!panel_installed(&home));
        symlink(&source, &plugin)?;
        std::fs::write(source.join("manifest.json"), "{}")?;
        assert!(!panel_installed(&home));
        for file in ["Panel.qml", "Service.qml"] {
            std::fs::write(source.join(file), "")?;
        }
        assert!(panel_installed(&home));
        std::fs::remove_file(source.join("Panel.qml"))?;
        assert!(!panel_installed(&home));
        std::fs::remove_dir_all(root)?;
        Ok(())
    }

}
