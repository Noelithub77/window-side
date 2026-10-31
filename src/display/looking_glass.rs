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
use std::path::{Path, PathBuf};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::time::Duration;

/// The IDD ships only in bleeding-edge builds; the client must be built from the same tag.
pub const TAG: &str = "B7-829-cdba29f2";
const SHARED_MEMORY: &str = "/dev/shm/looking-glass";
const INSTALL_SCRIPT: &str = include_str!("../../assets/guest/install-looking-glass.ps1");
const CLIENT_OPTIONS: &[&str] = &[
    "win:appId=window-side-viewer",
    "input:escapeKey=KEY_SCROLLLOCK",
    "input:overlayKey=KEY_INSERT",
    "input:ctrlAltCapture=yes",
    "win:softOverlay=yes",
    "win:uiFont=DejaVu Sans",
    "win:uiSize=16",
    // LGMP carries video and input; the domain exposes no SPICE port, so the fallback only retries.
    "spice:enable=no",
];

pub fn open(config: &Config, profile: Profile) -> Result<()> {
    if profile != Profile::Gpu {
        bail!("Looking Glass is available only in the GPU boot profile.");
    }
    if !virsh::is_running(&config.vm.name) {
        bail!("Start Windows before opening Looking Glass.");
    }
    let client = client()?;
    let target = Target { program: "window-side-viewer", classes: &["window-side-viewer"], name: None };
    if desktop::raise(&target) {
        return Ok(());
    }
    desktop::close(&target);
    start_client(&client, Path::new("systemd-run"))
}

pub fn client() -> Result<PathBuf> {
    client_path(&std::env::current_exe().context("Could not find Window Side")?)
}

fn client_path(app: &Path) -> Result<PathBuf> {
    let prefix = app.parent().and_then(Path::parent).context("Window Side has no installation prefix")?;
    let client = prefix.join("libexec/window-side/window-side-viewer");
    if !client.metadata().is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0) {
        bail!("Window Side's viewer is missing at {}. Reinstall Window Side to restore it.", client.display());
    }
    Ok(client)
}

fn start_client(client: &Path, launcher: &Path) -> Result<()> {
    let output = Command::new(launcher)
        .args(["--user", "--collect", "--quiet", "--service-type=exec"])
        .arg(client).args(["-f", SHARED_MEMORY]).args(CLIENT_OPTIONS)
        .stdin(Stdio::null()).output().context("Could not start Window Side's viewer")?;
    if !output.status.success() {
        bail!("Window Side's viewer could not start ({}): {}", output.status, String::from_utf8_lossy(&output.stderr).trim());
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static SERIAL: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Result<Self> {
            let path = std::env::temp_dir().join(format!("ws-viewer-{}-{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed)));
            fs::create_dir(&path)?;
            Ok(Self(path))
        }
        fn executable(&self, relative: &str, script: &str) -> Result<PathBuf> {
            let path = self.0.join(relative);
            if let Some(parent) = path.parent() { fs::create_dir_all(parent)?; }
            fs::write(&path, script)?;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755))?;
            Ok(path)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
    }

    #[test]
    fn viewer_resolution_requires_the_app_owned_executable() -> Result<()> {
        let fixture = Fixture::new()?;
        let app = fixture.0.join("prefix with spaces/bin/ws");
        assert!(client_path(&app).is_err());
        let viewer = fixture.executable("prefix with spaces/libexec/window-side/window-side-viewer", "#!/bin/sh\nexit 0\n")?;
        assert_eq!(client_path(&app)?, viewer);
        fs::set_permissions(&viewer, fs::Permissions::from_mode(0o644))?;
        assert!(client_path(&app).is_err());
        assert!(client_path(Path::new("ws")).is_err());
        Ok(())
    }

    #[test]
    fn app_launch_uses_private_viewer_and_overrides_user_key_settings() -> Result<()> {
        let fixture = Fixture::new()?;
        let client = fixture.0.join("prefix with spaces/libexec/window-side/window-side-viewer");
        let record = fixture.0.join("arguments");
        let launcher = fixture.executable("launcher", &format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\n", record.display()))?;
        start_client(&client, &launcher)?;
        let args = fs::read_to_string(record)?.lines().map(str::to_owned).collect::<Vec<_>>();
        assert_eq!(args.get(4).map(String::as_str), client.to_str());
        assert!(args.iter().any(|arg| arg == "input:overlayKey=KEY_INSERT"));
        assert!(args.iter().any(|arg| arg == "input:ctrlAltCapture=yes"));
        assert!(args.iter().any(|arg| arg == "win:softOverlay=yes"));
        assert!(args.iter().any(|arg| arg == "win:appId=window-side-viewer"));
        assert!(args.iter().any(|arg| arg == "win:uiFont=DejaVu Sans"));
        Ok(())
    }

    #[test]
    fn app_launch_reports_service_failures() -> Result<()> {
        let fixture = Fixture::new()?;
        let launcher = fixture.executable("launcher", "#!/bin/sh\necho 'fixture launch rejected' >&2\nexit 23\n")?;
        let error = start_client(Path::new("/missing-viewer"), &launcher).err().context("A failed launch was reported as success")?;
        assert!(error.to_string().contains("fixture launch rejected"));
        assert!(error.to_string().contains("23"));
        Ok(())
    }
}
