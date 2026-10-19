//! Compatibility boot, driver readiness and the cold transition to VirtIO.
use crate::config::{Config, config_dir, state_dir};
use crate::display::{console, looking_glass};
use crate::guest::{agent, commands};
use crate::host::{self, Profile};
use crate::setup;
use crate::vm::{drivers, lifecycle, virsh};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::time::Duration;

const PROVISION: &str = include_str!("../assets/guest/first-boot.ps1");
const DISPLAY_STARTUP: &str = include_str!("../assets/guest/display-startup.ps1");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    Start,
    Drivers,
    SignIn,
    Ready,
    GpuBoot,
    Complete,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Status {
    pub environment: String,
    pub domain: String,
    pub stage: Stage,
    pub storage_ready: Option<bool>,
    pub network_ready: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Guest {
    user: String,
    storage_ready: bool,
    network_ready: bool,
    applied: bool,
}

impl Guest {
    fn stage(&self) -> Stage {
        if !self.storage_ready || !self.network_ready { Stage::Drivers }
        else if self.user.trim().is_empty() { Stage::SignIn }
        else { Stage::Ready }
    }
}

fn read_guest(domain: &str, apply: bool) -> Result<Guest> {
    let json = agent::powershell_ok(domain, PROVISION, if apply { "-Apply" } else { "" }, Duration::from_secs(90))?;
    serde_json::from_str(json.trim()).context("Windows returned an invalid first-boot report")
}

pub fn status(config: &Config) -> Result<Status> {
    let mut status = Status {
        environment: "window-side-libvirt".into(), domain: config.vm.name.clone(),
        stage: Stage::Start, storage_ready: None, network_ready: None,
    };
    if !config.needs_first_boot() {
        status.stage = Stage::Complete;
    } else if config.gpu.is_some() && host::boot_profile(config.gpu.as_ref()) != Profile::Gpu {
        status.stage = Stage::GpuBoot;
    } else if virsh::is_running(&config.vm.name) {
        status.stage = Stage::Drivers;
        if agent::ping(&config.vm.name) {
            let guest = read_guest(&config.vm.name, false)?;
            status.stage = guest.stage();
            status.storage_ready = Some(guest.storage_ready);
            status.network_ready = Some(guest.network_ready);
        }
    }
    Ok(status)
}

fn workflow_lock() -> Result<File> {
    fs::create_dir_all(state_dir())?;
    let lock = File::create(state_dir().join("first-boot.lock"))?;
    lock.try_lock().context("Another first-boot operation is in progress")?;
    Ok(lock)
}

/// Starts only the compatibility boot and returns without waiting for a missing guest agent.
pub fn start(config: &Config) -> Result<()> {
    let _lock = workflow_lock()?;
    ensure!(config.needs_first_boot(), "First boot is already complete; use ws start");
    let profile = host::boot_profile(config.gpu.as_ref());
    lifecycle::start(config, profile)?;
    drivers::prime_storage(&config.vm.name)?;
    drivers::attach(&config.vm.name)?;
    console::open(&config.vm.name)
}

/// On failure after shutdown, restore compatibility settings and leave Windows stopped for inspection.
fn save_transition(config: &Config, xml: &str, old_config: &str, old_xml: &str) -> Result<()> {
    let config_path = config_dir().join("config.toml");
    let domain_path = lifecycle::domain_file();
    let result = (|| {
        fs::write(&domain_path, xml)?;
        virsh::run(&["define", "--validate", &domain_path.display().to_string()])?;
        fs::write(&config_path, toml::to_string(config)?)?;
        Ok(())
    })();
    if let Err(error) = result {
        fs::write(&domain_path, old_xml).context("Transition failed and the SATA domain could not be restored")?;
        fs::write(config_path, old_config).context("Transition failed and the old configuration could not be restored")?;
        virsh::run(&["define", "--validate", &domain_path.display().to_string()])
            .context("Transition failed and libvirt could not restore the SATA definition")?;
        return Err(error);
    }
    Ok(())
}

/// Explicitly provisions Windows, closes its apps cleanly, switches buses while stopped and boots again.
pub fn finish(config: &Config) -> Result<Status> {
    let _lock = workflow_lock()?;
    if !config.needs_first_boot() { return status(config) }
    let domain = config.vm.name.as_str();
    ensure!(virsh::is_running(domain), "Start first boot before finishing setup");
    ensure!(agent::ping(domain), "Install the guest tools from the driver CD, then retry");
    let profile = host::boot_profile(config.gpu.as_ref());
    if config.gpu.is_some() && profile != Profile::Gpu {
        bail!("Finish graphics setup in the GPU boot profile; shut down Windows, then use ws reboot gpu");
    }
    drivers::prime_storage(domain)?;
    let guest = read_guest(domain, false)?;
    ensure!(guest.stage() == Stage::Ready, "Install the VirtIO guest tools and sign in to Windows before finishing setup");
    let mut upgraded = config.clone();
    upgraded.vm.virtio = true;
    upgraded.windows.provisioned = false;
    upgraded.windows.user = guest.user.rsplit('\\').next().context("Windows user name is missing")?.to_owned();
    let inventory = setup::detect()?;
    let choices = inventory.default_choices(Some(&upgraded)).context("The configured Windows install was not detected")?;
    ensure!(choices.windows == inventory.windows.iter().find(|disk| disk.filesystem_uuid == config.disk.windows_partition_uuid)
        .context("The configured Windows partition was not detected")?.partition, "The Windows install changed during setup");
    let plan = setup::plan(&inventory, &choices, Some(&upgraded))?;
    let old_config = fs::read_to_string(config_dir().join("config.toml"))?;
    let old_xml = fs::read_to_string(lifecycle::domain_file())?;
    if !config.vm.virtio {
        commands::write_private(&state_dir().join("first-boot-config-before.toml"), &old_config)?;
        commands::write_private(&state_dir().join("first-boot-domain-before.xml"), &old_xml)?;
    }
    agent::install_script(domain, "display-startup.ps1", DISPLAY_STARTUP)?;
    if config.gpu.is_some() { looking_glass::install(config)?; }
    let applied = read_guest(domain, true)?;
    ensure!(applied.applied && applied.stage() == Stage::Ready, "Windows provisioning did not complete");
    lifecycle::stop(domain)?;
    save_transition(&upgraded, &plan.domain_xml, &old_config, &old_xml)?;
    lifecycle::start(&upgraded, profile).context("VirtIO start failed; SATA recovery files are in the window-side state directory")?;
    lifecycle::wait_for_agent(domain).context("VirtIO boot is unverified; compatibility recovery files are in the window-side state directory")?;
    upgraded.windows.provisioned = true;
    fs::write(config_dir().join("config.toml"), toml::to_string(&upgraded)?)?;
    status(&upgraded)
}

pub fn recover(config: &Config) -> Result<()> {
    let _lock = workflow_lock()?;
    ensure!(!virsh::is_running(&config.vm.name), "Stop Windows before restoring its SATA configuration");
    let old_config = fs::read_to_string(state_dir().join("first-boot-config-before.toml"))
        .context("No first-boot recovery configuration was saved")?;
    let previous: Config = toml::from_str(&old_config)?;
    ensure!(!previous.vm.virtio && previous.vm.name == config.vm.name
        && previous.disk.id == config.disk.id && previous.disk.windows_partition_uuid == config.disk.windows_partition_uuid,
        "The recovery files belong to a different Windows setup");
    let old_xml = fs::read_to_string(state_dir().join("first-boot-domain-before.xml"))?;
    save_transition(&previous, &old_xml,
        &fs::read_to_string(config_dir().join("config.toml"))?,
        &fs::read_to_string(lifecycle::domain_file())?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_drivers_and_missing_login_never_allow_finish() {
        let mut guest = Guest { user: "PC\\tester".into(), storage_ready: false, network_ready: true, applied: false };
        assert_eq!(guest.stage(), Stage::Drivers);
        guest.storage_ready = true;
        guest.network_ready = false;
        assert_eq!(guest.stage(), Stage::Drivers);
        guest.network_ready = true;
        guest.user.clear();
        assert_eq!(guest.stage(), Stage::SignIn);
        guest.user = "PC\\tester".into();
        assert_eq!(guest.stage(), Stage::Ready);
    }

    #[test]
    fn missing_report_fields_fail_closed() {
        assert!(serde_json::from_str::<Guest>(r#"{"user":"tester","storageReady":true}"#).is_err());
        let guest: Guest = serde_json::from_str(r#"{"user":"PC\\tester","storageReady":true,"networkReady":true,"applied":false}"#).unwrap();
        assert_eq!(guest.stage(), Stage::Ready);
        assert!(!guest.applied);
    }
}
