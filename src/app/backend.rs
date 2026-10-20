use super::types::*;
use crate::{agents, config::{Config, DiskMode as CoreDiskMode}, detect, display, setup};
use anyhow::Result;

pub fn detection() -> Result<Detection> {
    let inventory = setup::detect()?;
    let config = Config::load().ok();
    let s = &inventory.system;
    let choices = inventory.default_choices(config.as_ref()).map(Into::into);
    let windows = inventory.windows.iter().map(|w| WindowsInstall { partition: w.partition.clone(), disk_model: w.disk_model.clone(), size: w.size as f64, disk_size: w.disk_size as f64, dedicated_drive: w.mode == CoreDiskMode::Whole }).collect();
    let gpus = inventory.gpus.iter().map(|g| {
        let vendor = match g.ids.first().and_then(|id| id.split(':').next()) { Some("10de") => Vendor::Nvidia, Some("1002") => Vendor::Amd, Some("8086") => Vendor::Intel, _ => Vendor::Other };
        let blocker = g.blocker.map(|b| match b { detect::gpus::Blocker::HostDisplay => GpuBlocker::HostDisplay, detect::gpus::Blocker::IommuOff => GpuBlocker::IommuOff, detect::gpus::Blocker::SharedGroup => GpuBlocker::SharedGroup });
        GpuOption { address: g.address.clone(), name: g.name.clone(), vendor, blocker, usable: inventory.usable_gpus().any(|usable| usable.address == g.address) }
    }).collect();
    let host = Host {
        threads: s.cpus.len() as u32, performance_threads: s.performance_cpus.len() as u32,
        memory_mib: s.memory_mib as f64, memory_limit_mib: inventory.memory_limit_mib(), vcpu_limit: inventory.vcpu_limit(), iommu: s.iommu_enabled,
        missing_packages: s.missing.iter().flat_map(|m| m.packages.iter().map(|p| (*p).to_owned())).collect(),
        boot_loader: s.boot_profile.as_ref().map(|p| match p.loader { detect::boot::Loader::Limine => BootLoader::Limine, detect::boot::Loader::Grub => BootLoader::Grub, detect::boot::Loader::SystemdBoot => BootLoader::SystemdBoot }),
        distro: s.distro.map(|d| match d { detect::distro::Family::Arch => Distro::Arch, detect::distro::Family::Debian => Distro::Debian, detect::distro::Family::Fedora => Distro::Fedora }),
        desktop: match display::desktop::current() { display::desktop::Desktop::Hyprland => Desktop::Hyprland, display::desktop::Desktop::Kde => Desktop::Kde, display::desktop::Desktop::Gnome => Desktop::Gnome, display::desktop::Desktop::Other => Desktop::Other }, omarchy: s.omarchy,
    };
    Ok(Detection { windows, gpus, host, choices, configured: config.is_some() })
}

pub fn core_plan(choices: Choices) -> Result<setup::Plan> {
    setup::plan(&setup::detect()?, &choices.into(), Config::load().ok().as_ref())
}
pub fn plan(p: setup::Plan) -> Plan {
    Plan {
        facts: vec![Fact { label: "Windows".into(), value: p.config.disk.id.clone() }, Fact { label: "Memory".into(), value: format!("{} GiB", p.config.vm.memory_mib / 1024) }, Fact { label: "Processor".into(), value: format!("{} threads", p.config.vm.vcpus) }, Fact { label: "Graphics".into(), value: p.config.gpu.as_ref().map_or_else(|| "Virtual display".into(), |g| g.address.clone()) }],
        notes: p.notes, packages: p.root.packages, groups: p.root.groups, files: p.root.files.into_iter().map(|f| PlanFile { path: f.path, contents: f.contents }).collect(), domain_xml: p.domain_xml,
    }
}

pub fn shell(appearance: Appearance) -> Shell {
    let palette = palette();
    let agents = agents::detect().into_iter().map(|a| Agent { id: match a { agents::Agent::Claude => AgentId::Claude, agents::Agent::Codex => AgentId::Codex, agents::Agent::OpenCode => AgentId::Opencode }, name: a.name().into() }).collect();
    Shell { version: env!("CARGO_PKG_VERSION").into(), fixture: false, omarchy: crate::config::home().join(".config/omarchy").is_dir(), appearance, palette, agents }
}
fn palette() -> Option<Palette> {
    let path = crate::config::home().join(".config/omarchy/current/theme/colors.toml");
    let colors: toml::Table = toml::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    let color = |key: &str| -> Option<String> {
        let value = colors.get(key)?.as_str()?;
        let hex = value.strip_prefix('#')?;
        (hex.len() == 6 && hex.bytes().all(|c| c.is_ascii_hexdigit())).then(|| value.into())
    };
    Some(Palette { background: color("background")?, foreground: color("foreground")?, accent: color("accent")?, colors: (1..=6).map(|i| color(&format!("color{i}"))).collect::<Option<_>>()? })
}

pub fn appearance() -> Appearance {
    let folder = crate::config::config_dir();
    let json = std::fs::read_to_string(folder.join("preferences.json")).ok().and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok());
    let old = || std::fs::read_to_string(folder.join("appearance.toml")).ok().and_then(|text| toml::from_str::<toml::Table>(&text).ok()).and_then(|t| t.get("appearance").and_then(toml::Value::as_str).map(str::to_owned));
    if json.as_ref().and_then(|v| v["appearance"].as_str()).map(str::to_owned).or_else(old).as_deref() == Some("omarchy") { Appearance::Omarchy } else { Appearance::Pastel }
}
pub fn save_appearance(appearance: Appearance) -> Result<()> {
    let folder = crate::config::config_dir();
    std::fs::create_dir_all(&folder)?;
    let path = folder.join("preferences.json");
    let mut value = match std::fs::read_to_string(&path) { Ok(text) => serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&text)?, Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(), Err(e) => return Err(e.into()) };
    value.insert("appearance".into(), serde_json::to_value(appearance)?);
    let temp = folder.join("preferences.json.tmp");
    std::fs::write(&temp, serde_json::to_string_pretty(&value)? + "\n")?;
    std::fs::rename(temp, path)?;
    Ok(())
}

pub fn action(action: Action) -> Result<()> {
    use crate::cli::{self, Action as C, FirstBootAction as F, SetupStep, SignInAction};
    use crate::guest::commands::UpdateAction;
    if let Action::Reboot { profile } = action {
        let result = std::process::Command::new(std::env::current_exe()?).args(["reboot", match profile { BootProfile::Normal => "normal", BootProfile::Gpu => "gpu" }]).output()?;
        anyhow::ensure!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr).trim());
        return Ok(());
    }
    let command = match action {
        Action::Launch => C::Launch, Action::Console => C::Console, Action::Stop => C::Stop, Action::ForceStop => C::ForceStop,
        Action::Resolution { width, height } => C::Resolution { width, height, json: true }, Action::LookingGlassSetup => C::LookingGlassSetup,
        Action::UpdatesCheck => C::Updates { action: UpdateAction::Check }, Action::UpdatesInstall => C::Updates { action: UpdateAction::Install },
        Action::SignIn => C::SignIn { action: SignInAction::Run, wait: 40 }, Action::InstallPlugin => C::Setup { step: SetupStep::Plugin },
        Action::FirstBootStart => C::Setup { step: SetupStep::FirstBoot { action: F::Start } }, Action::FirstBootFinish => C::Setup { step: SetupStep::FirstBoot { action: F::Finish } },
        Action::Reboot { .. } => return Ok(()),
    };
    anyhow::ensure!(cli::run(command)? == 0, "Windows could not complete the action");
    Ok(())
}
