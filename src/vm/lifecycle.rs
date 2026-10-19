use crate::config::{Config, DiskMode, config_dir, state_dir};
use crate::guest::agent::ping;
use crate::host::{self, Profile};
use crate::notify::{self, Urgency};
use crate::process::{hook, output, require, text};
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use std::fs::{self, File};
use std::path::Path;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn domain_file() -> std::path::PathBuf {
    config_dir().join("domain.xml")
}

fn verify_host(config: &Config, profile: Profile) -> Result<()> {
    if !domain_file().exists() {
        bail!("Missing configuration: {}", domain_file().display());
    }
    let kvm = File::options().read(true).write(true).open("/dev/kvm");
    if kvm.is_err() {
        bail!("KVM is unavailable to the current user.");
    }
    let disk = config.disk_link();
    let resolved = fs::canonicalize(&disk).with_context(|| match config.disk.mode {
        DiskMode::Whole => format!("The configured Windows disk {} was not found.", config.disk.id),
        DiskMode::Partition => "The Windows virtual disk is not assembled. Check window-side-disk.service.".to_owned(),
    })?;
    if !resolved.starts_with("/dev/") {
        bail!("The Windows disk resolved to an unexpected device: {}", resolved.display());
    }
    let host = fs::canonicalize(config.host_device()).context("The Windows partition was not found.")?;
    if !windows_partition_matches(&host, &config.disk.windows_partition_uuid)? {
        bail!("The Windows partition identity does not match the configured disk.");
    }
    if let Some(gpu) = &config.gpu {
        host::verify_gpu_topology(gpu)?;
        if profile == Profile::Gpu {
            host::verify_gpu_claimed(gpu)?;
        }
    }
    if profile == Profile::Unknown {
        bail!("The current boot profile could not be identified. Reboot Linux before starting Windows.");
    }
    Ok(())
}

/// Checks the configured Windows partition is on the configured device.
fn windows_partition_matches(disk: &Path, uuid: &str) -> Result<bool> {
    let disk = disk.display().to_string();
    let uuids = text("lsblk", &["-nrpo", "UUID", &disk])?;
    Ok(uuids.lines().any(|line| line.trim().eq_ignore_ascii_case(uuid)))
}

fn ensure_network() -> Result<()> {
    let info = virsh::run(&["net-info", "default"]).context("The default libvirt network is missing. Reinstall libvirt to restore it.")?;
    let field = |name: &str| {
        info.lines().find_map(|line| line.strip_prefix(name)).map(|value| value.trim_start_matches(':').trim() == "yes")
    };
    if field("Active") != Some(true) {
        virsh::run(&["net-start", "default"]).context("Could not start the default VM network.")?;
    }
    if field("Autostart") != Some(true) {
        virsh::run(&["net-autostart", "default"]).context("Could not enable the default VM network.")?;
    }
    Ok(())
}

/// Validates the host and redefines the domain from its configuration file.
pub fn setup_domain(config: &Config, profile: Profile) -> Result<()> {
    require("virsh")?;
    verify_host(config, profile)?;
    ensure_network()?;
    let file = domain_file().display().to_string();
    virsh::run(&["define", "--validate", &file]).context("The libvirt domain configuration is invalid.")?;
    virsh::run(&["autostart", &config.vm.name, "--disable"]).context("Could not disable automatic Windows startup.")?;
    Ok(())
}

/// Linux must not hold any Windows partition open while the VM writes to it.
fn unmount_windows_disk(config: &Config) -> Result<()> {
    let disk = fs::canonicalize(config.host_device())?.display().to_string();
    let mounted = || -> Result<Vec<(String, String)>> {
        let devices = text("lsblk", &["-nrpo", "NAME", &disk])?;
        Ok(devices
            .lines()
            .filter_map(|device| {
                let target = output("findmnt", &["-rn", "-S", device, "-o", "TARGET"]).ok()?;
                let target = String::from_utf8_lossy(&target.stdout).lines().next()?.to_owned();
                Some((device.to_owned(), target))
            })
            .collect())
    };
    for (device, target) in mounted()? {
        eprintln!("Unmounting {device} from {target}");
        text("udisksctl", &["unmount", "-b", &device])
            .with_context(|| format!("Could not unmount {target}. Close files using the Windows drive and try again."))?;
    }
    if let Some((_, target)) = mounted()?.first() {
        bail!("The Windows disk is still mounted at {target}.");
    }
    Ok(())
}

fn attach_gpu(config: &Config) -> Result<()> {
    let Some(gpu) = &config.gpu else { return Ok(()) };
    let domain = config.vm.name.as_str();
    let functions = std::iter::once(&gpu.address).chain(gpu.audio_address.as_ref());
    let mut attached: Vec<String> = Vec::new();
    for (index, address) in functions.enumerate() {
        let file = state_dir().join(format!("gpu-hostdev-{index}.xml"));
        fs::write(&file, host::hostdev_xml(address)?)?;
        let file = file.display().to_string();
        if virsh::run(&["attach-device", domain, &file, "--config"]).is_err() {
            for done in &attached {
                let _ = virsh::run(&["detach-device", domain, done, "--config"]);
            }
            bail!("Could not prepare GPU function {address} for passthrough. Linux kept ownership.");
        }
        attached.push(file);
    }
    Ok(())
}

/// Starts Windows without a viewer; returns once QEMU is running.
pub fn start(config: &Config, profile: Profile) -> Result<()> {
    for program in ["findmnt", "lsblk", "udisksctl"] {
        require(program)?;
    }
    let state = state_dir();
    fs::create_dir_all(&state)?;
    let lock = File::create(state.join("backend.lock"))?;
    lock.lock().context("Could not lock the VM backend")?;
    let domain = config.vm.name.as_str();
    match virsh::state(domain).as_str() {
        "running" => return Ok(()),
        "paused" => bail!("Windows is paused. Resume it with virsh before opening it."),
        _ => {}
    }
    if let Some(result) = hook(&config.hooks.before_start)
        && !result.is_ok_and(|output| output.status.success())
    {
        bail!("The Windows disk is unavailable. Check its status before starting Windows.");
    }
    setup_domain(config, profile)?;
    unmount_windows_disk(config)?;
    if profile == Profile::Gpu {
        attach_gpu(config)?;
        let memory = config.vm.memory_mib / 1024;
        notify::send(Urgency::Normal, &format!("Starting Windows with the GPU, {memory} GB RAM, and {} vCPUs.", config.vm.vcpus));
    } else {
        notify::send(Urgency::Normal, "Starting Windows with Linux keeping the GPU.");
    }
    virsh::run(&["start", domain])
        .context("Libvirt could not start Windows. Reboot to the normal profile if the GPU profile is active.")?;
    crate::display::looking_glass::grant_shared_memory();
    let started = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs());
    fs::write(state.join("started-at"), format!("{started}\n"))?;
    Ok(())
}

pub fn wait_for_agent(domain: &str) -> Result<()> {
    for _ in 0..90 {
        if ping(domain) {
            return Ok(());
        }
        if !virsh::is_running(domain) {
            bail!("Windows stopped while booting.");
        }
        sleep(Duration::from_secs(2));
    }
    bail!("Windows is running, but its guest agent did not answer within three minutes.")
}

fn forget_run_state() {
    let _ = fs::remove_file(state_dir().join("started-at"));
}

pub fn stop(domain: &str) -> Result<()> {
    if !virsh::exists(domain) || !virsh::is_running(domain) {
        eprintln!("window-side is already stopped.");
        notify::send(Urgency::Low, "Windows is already stopped.");
        return Ok(());
    }
    notify::send(Urgency::Normal, "Requesting a clean Windows shutdown.");
    virsh::run(&["shutdown", domain, "--mode", "agent"])
        .or_else(|_| virsh::run(&["shutdown", domain, "--mode", "acpi"]))
        .context("Could not request a clean shutdown.")?;
    for _ in 0..120 {
        if !virsh::is_running(domain) {
            forget_run_state();
            notify::send(Urgency::Normal, "Windows has stopped.");
            return Ok(());
        }
        sleep(Duration::from_secs(1));
    }
    bail!("Windows did not stop within two minutes. Use force-stop only if Windows is stuck.")
}

pub fn force_stop(domain: &str) -> Result<()> {
    if !virsh::exists(domain) || !virsh::is_running(domain) {
        println!("window-side is already stopped.");
        return Ok(());
    }
    notify::send(Urgency::Critical, "Force stopping Windows. Unsaved Windows data may be lost.");
    virsh::run(&["destroy", domain]).context("Libvirt could not force stop Windows.")?;
    forget_run_state();
    notify::send(Urgency::Normal, "Windows was force stopped.");
    Ok(())
}
