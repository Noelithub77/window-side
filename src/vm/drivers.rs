use crate::config::cache_dir;
use crate::notify::{self, Urgency};
use crate::process::{require, text};
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::PathBuf;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

/// Present an empty readonly VirtIO disk so Windows binds viostor before its boot disk changes.
pub fn prime_storage(domain: &str) -> Result<()> {
    let uid = fs::metadata("/proc/self")?.uid();
    let path = std::env::temp_dir().join(format!("window-side-virtio-{uid}.img"));
    let source = path.display().to_string();
    let blocks = virsh::run(&["domblklist", domain, "--details"])?;
    if let Some(line) = blocks.lines().find(|line| line.split_whitespace().nth(2) == Some("vdz")) {
        if line.split_whitespace().last() != Some(source.as_str()) {
            bail!("The temporary VirtIO target vdz is already in use");
        }
        return Ok(());
    }
    match fs::OpenOptions::new().write(true).create_new(true).mode(0o644).open(&path) {
        Ok(file) => file.set_len(32 * 1024 * 1024)?,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let meta = path.symlink_metadata()?;
            if !meta.is_file() || meta.uid() != uid || meta.nlink() != 1 || meta.len() != 32 * 1024 * 1024 {
                bail!("The temporary VirtIO disk is not an owned regular scratch file");
            }
        },
        Err(error) => return Err(error.into()),
    }
    virsh::run(&["attach-disk", domain, &source, "vdz", "--targetbus", "virtio", "--driver", "qemu",
        "--subdriver", "raw", "--mode", "readonly", "--sourcetype", "file", "--live"])
        .context("Could not present the temporary VirtIO driver disk")?;
    Ok(())
}

const DRIVERS_URL: &str =
    "https://fedorapeople.org/groups/virt/virtio-win/direct-downloads/stable-virtio/virtio-win-guest-tools.exe";

fn runtime_iso() -> PathBuf {
    let uid = fs::metadata("/proc/self").map(|meta| std::os::unix::fs::MetadataExt::uid(&meta)).unwrap_or(0);
    std::env::temp_dir().join(format!("window-side-drivers-{uid}.iso"))
}

/// Attaches a CD holding the stable VirtIO guest tools installer to running Windows.
pub fn attach(domain: &str) -> Result<()> {
    for program in ["curl", "virsh", "xorriso"] {
        require(program)?;
    }
    if !virsh::is_running(domain) {
        bail!("Start Windows before attaching the driver CD.");
    }
    let runtime = runtime_iso();
    let runtime_path = runtime.display().to_string();
    if virsh::run(&["domblklist", domain, "--details"])?.contains(&runtime_path) {
        eprintln!("The window-side driver CD is already attached.");
        return Ok(());
    }
    let cache = cache_dir();
    fs::create_dir_all(&cache)?;
    let installer = cache.join("virtio-win-guest-tools.exe");
    if !installer.exists() {
        notify::send(Urgency::Normal, "Downloading the stable Windows guest drivers.");
        let download = cache.join("virtio-win-guest-tools.exe.download");
        text("curl", &["--fail", "--location", "--show-error", "--output", &download.display().to_string(), DRIVERS_URL])
            .context("Could not download the Windows guest drivers.")?;
        fs::rename(&download, &installer)?;
    }
    let iso = cache.join("window-side-drivers.iso");
    let build = cache.join("window-side-drivers.iso.new");
    let _ = fs::remove_file(&build);
    text("xorriso", &[
        "-as", "mkisofs", "-quiet", "-J", "-r", "-V", "WSDRIVERS", "-graft-points",
        &format!("virtio-win-guest-tools.exe={}", installer.display()), "-o", &build.display().to_string(),
    ])
    .context("Could not build the driver CD.")?;
    fs::rename(&build, &iso)?;
    fs::copy(&iso, &runtime)?;
    let _ = virsh::run(&["detach-device-alias", domain, "redir1", "--live"]);
    virsh::run(&[
        "attach-disk", domain, &runtime_path, "sdb", "--targetbus", "sata", "--type", "cdrom",
        "--mode", "readonly", "--sourcetype", "file", "--live",
    ])
    .context("Could not attach the Windows driver CD.")?;
    notify::send(Urgency::Normal, "Driver CD attached. Run virtio-win-guest-tools.exe inside Windows.");
    Ok(())
}
