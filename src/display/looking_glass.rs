//! Looking Glass shows Windows from the passed-through GPU through shared memory.
use crate::config::{Config, cache_dir};
use crate::display::desktop::{self, Target};
use crate::guest::agent::{GUEST_DIR, powershell_ok, write_file};
use crate::host::Profile;
use crate::notify::{self, Urgency};
use crate::process::{require, text};
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// The IDD ships only in bleeding-edge builds; the client must be built from the same tag.
pub const TAG: &str = "B7-829-cdba29f2";
const SHARED_MEMORY: &str = "/dev/shm/looking-glass";
const INSTALL_SCRIPT: &str = include_str!("../../assets/guest/install-looking-glass.ps1");

pub fn open(config: &Config, profile: Profile) -> Result<()> {
    if profile != Profile::Gpu {
        bail!("Looking Glass is available only in the GPU boot profile.");
    }
    if !virsh::is_running(&config.vm.name) {
        bail!("Start Windows before opening Looking Glass.");
    }
    require("looking-glass-client")?;
    let target = Target { program: "looking-glass-client", classes: &["looking-glass-client"], name: None };
    if desktop::raise(&target) {
        return Ok(());
    }
    desktop::close(&target);
    Command::new("systemd-run")
        .args(["--user", "--collect", "--quiet", "--service-type=exec", "looking-glass-client", "-f", SHARED_MEMORY])
        .status()
        .context("Could not start the Looking Glass client")?;
    Ok(())
}

/// libvirt recreates the shared memory file as root on each start; let this user read frames.
pub fn grant_shared_memory() {
    let path = Path::new(SHARED_MEMORY);
    let is_file = path.symlink_metadata().is_ok_and(|meta| meta.file_type().is_file());
    let Ok(uid) = fs::metadata("/proc/self").map(|meta| meta.uid()) else { return };
    if is_file {
        let granted = Command::new("sudo")
            .args(["-n", "setfacl", "-m", &format!("u:{uid}:rw"), SHARED_MEMORY])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !granted {
            notify::send(Urgency::Normal, "Looking Glass shared-memory access could not be granted.");
        }
    }
}

/// Downloads the pinned IDD installer and installs it in running Windows.
pub fn install(config: &Config) -> Result<serde_json::Value> {
    let domain = config.vm.name.as_str();
    if !virsh::is_running(domain) {
        bail!("Start Windows before installing Looking Glass.");
    }
    require("curl")?;
    require("bsdtar")?;
    let folder = cache_dir().join(format!("looking-glass-{TAG}"));
    fs::create_dir_all(&folder)?;
    let installer = folder.join("looking-glass-idd-setup.exe");
    if !installer.exists() {
        let archive = folder.join("idd.zip");
        let archive_path = archive.display().to_string();
        // The artifact host rejects curl's default user agent.
        text("curl", &[
            "--fail", "--location", "--max-time", "90", "-A", "Mozilla/5.0",
            &format!("https://looking-glass.io/artifact/{TAG}/idd"), "-o", &archive_path,
        ])
        .context("Could not download the Looking Glass display driver")?;
        text("bsdtar", &["-xf", &archive_path, "-C", &folder.display().to_string(), "looking-glass-idd-setup.exe"])?;
    }
    powershell_ok(domain, &format!("New-Item -ItemType Directory -Force '{GUEST_DIR}' | Out-Null"), "", Duration::from_secs(15))?;
    let target = format!(r"{GUEST_DIR}\looking-glass-idd-{TAG}-setup.exe");
    write_file(domain, &target, &fs::read(&installer)?)?;
    let refresh = config.windows.refresh_hz.replace('\'', "");
    let arguments = format!("-Installer '{target}' -Refresh '{refresh}'");
    let report = powershell_ok(domain, INSTALL_SCRIPT, &arguments, Duration::from_secs(300))?;
    let report = serde_json::from_str(report.trim()).context("Looking Glass returned an invalid setup report")?;
    grant_shared_memory();
    Ok(report)
}
