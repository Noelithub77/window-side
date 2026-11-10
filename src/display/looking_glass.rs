//! Looking Glass shows Windows through shared memory, from the passed-through GPU or CPU rendering.
use crate::config::{Config, cache_dir};
use crate::display::desktop::{self, Target};
use crate::guest::agent::{self, GUEST_DIR, powershell_ok, write_file};
use crate::host::Profile;
use crate::notify::{self, Urgency};
use crate::process::{require, text};
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

/// The IDD ships only in bleeding-edge builds; the client must be built from the same tag.
pub const TAG: &str = "B7-829-cdba29f2";
const SHARED_MEMORY: &str = "/dev/shm/looking-glass";
/// libvirt keeps SPICE off the network; only the granted user can reach this socket.
pub const SPICE_SOCKET: &str = "/dev/shm/window-side-spice.sock";
pub const DISPLAY_STARTUP: &str = include_str!("../../assets/guest/display-startup.ps1");
const INSTALL_SCRIPT: &str = include_str!("../../assets/guest/install-looking-glass.ps1");
const SCALE_SCRIPT: &str = include_str!("../../assets/guest/set-display-scale.ps1");
const VIEWER: Target<'static> = Target { program: "window-side-viewer", classes: &["window-side-viewer"], name: None };
const CLIENT_OPTIONS: &[&str] = &[
    "win:appId=window-side-viewer",
    // Fullscreen hands Windows the panel's native resolution; it follows the window size.
    "win:fullScreen=yes",
    "input:escapeKey=KEY_SCROLLLOCK",
    "input:overlayKey=KEY_INSERT",
    "input:ctrlAltCapture=yes",
    "win:softOverlay=yes",
    "win:uiFont=DejaVu Sans",
    "win:uiSize=16",
];

/// LGMP carries video, input and clipboard; SPICE adds only the audio LGMP lacks.
fn spice_options(reachable: bool) -> Vec<String> {
    if !reachable {
        return vec!["spice:enable=no".into()];
    }
    vec![
        format!("spice:host={SPICE_SOCKET}"), "spice:port=0".into(), "spice:input=no".into(),
        "spice:clipboard=no".into(), "spice:usbAudio=no".into(),
    ]
}

fn viewer_options(profile: Profile, spice_reachable: bool) -> Vec<String> {
    let mut options: Vec<String> = CLIENT_OPTIONS.iter().map(|option| (*option).to_owned()).collect();
    options.extend(spice_options(spice_reachable));
    // CPU-rendered frames gain nothing from multisampling, which costs integrated GPUs the most.
    if profile != Profile::Gpu {
        options.push("egl:multisample=no".into());
    }
    options
}

pub fn open(config: &Config, profile: Profile) -> Result<()> {
    if !virsh::is_running(&config.vm.name) {
        bail!("Start Windows before opening Looking Glass.");
    }
    let client = client()?;
    if desktop::raise(&VIEWER) {
        return Ok(());
    }
    desktop::close(&VIEWER);
    let options = viewer_options(profile, UnixStream::connect(SPICE_SOCKET).is_ok());
    start_client(&client, Path::new("systemd-run"), &options)?;
    match_scale(&config.vm.name);
    Ok(())
}

/// Sets Windows' scaling to the host display's so text keeps the size it has on Linux.
/// Windows stores scale relative to a recommended value that depends on resolution,
/// so this waits for the fullscreen viewer to have resized the display first.
fn match_scale(domain: &str) {
    if !agent::ping(domain) {
        return;
    }
    std::thread::sleep(Duration::from_secs(3));
    let percent = desktop::windows_scale();
    let applied = agent::install_script(domain, "set-display-scale.ps1", SCALE_SCRIPT)
        .and_then(|target| powershell_ok(domain, &format!("& '{target}' -Percent {percent}"), "", Duration::from_secs(45)));
    if let Err(e) = applied {
        eprintln!("window-side: Windows scaling was left as it was: {e:#}");
    }
}

/// Opens the viewer without a passthrough GPU; false leaves Windows on its console display.
pub fn open_software(config: &Config, profile: Profile) -> Result<bool> {
    if desktop::raise(&VIEWER) {
        return Ok(true);
    }
    if !prepare(&config.vm.name) {
        return Ok(false);
    }
    open(config, profile)?;
    Ok(true)
}

/// Switches Windows to the display this host can show and reports whether that is Looking Glass.
pub fn prepare(domain: &str) -> bool {
    if !agent::ping(domain) {
        return false;
    }
    let usable = client().is_ok() && fs::File::options().read(true).write(true).open(SHARED_MEMORY).is_ok();
    // The startup task only runs at boot, and guests set up earlier hold an older copy of it.
    let _ = agent::install_script(domain, "display-startup.ps1", DISPLAY_STARTUP);
    let arguments = if usable { "" } else { "-UseVirtio" };
    let Ok(report) = powershell_ok(domain, DISPLAY_STARTUP, arguments, Duration::from_secs(60)) else {
        return false;
    };
    let chosen = usable && serde_json::from_str::<serde_json::Value>(report.trim()).is_ok_and(|report| report["lookingGlass"] == true);
    let _ = crate::guest::commands::write_private(&display_file(), if chosen { "looking-glass" } else { "console" });
    chosen
}

fn display_file() -> PathBuf {
    crate::config::state_dir().join("display")
}

/// Whether the last viewer launch moved Windows onto the Looking Glass display.
pub fn last_chosen() -> bool {
    fs::read_to_string(display_file()).is_ok_and(|display| display == "looking-glass")
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

fn start_client(client: &Path, launcher: &Path, options: &[String]) -> Result<()> {
    let output = Command::new(launcher)
        .args(["--user", "--collect", "--quiet", "--service-type=exec"])
        .arg(client).args(["-f", SHARED_MEMORY]).args(options)
        .stdin(Stdio::null()).output().context("Could not start Window Side's viewer")?;
    if !output.status.success() {
        bail!("Window Side's viewer could not start ({}): {}", output.status, String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

/// libvirt recreates the frame buffer and SPICE socket on each start; let this user reach them.
pub fn grant_viewer_access() {
    let Ok(uid) = fs::metadata("/proc/self").map(|meta| meta.uid()) else { return };
    for path in [SHARED_MEMORY, SPICE_SOCKET] {
        let Ok(kind) = Path::new(path).symlink_metadata().map(|meta| meta.file_type()) else { continue };
        if !kind.is_file() && !kind.is_socket() {
            continue;
        }
        let granted = Command::new("sudo")
            .args(["-n", "setfacl", "-m", &format!("u:{uid}:rw"), path])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !granted {
            notify::send(Urgency::Normal, "Looking Glass access could not be granted.");
            return;
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
    agent::install_script(domain, "display-startup.ps1", DISPLAY_STARTUP)?;
    grant_viewer_access();
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
        start_client(&client, &launcher, &viewer_options(Profile::Gpu, false))?;
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
        let error = start_client(Path::new("/missing-viewer"), &launcher, &[]).err().context("A failed launch was reported as success")?;
        assert!(error.to_string().contains("fixture launch rejected"));
        assert!(error.to_string().contains("23"));
        Ok(())
    }

    #[test]
    fn spice_supplies_only_audio_and_stays_off_without_its_socket() {
        let has = |options: &[String], wanted: &str| options.iter().any(|option| option == wanted);
        let reachable = viewer_options(Profile::Gpu, true);
        assert!(has(&reachable, &format!("spice:host={SPICE_SOCKET}")));
        assert!(has(&reachable, "spice:port=0"));
        assert!(has(&reachable, "spice:input=no"));
        assert!(has(&reachable, "spice:clipboard=no"));
        assert!(!has(&reachable, "spice:enable=no"));
        assert!(has(&viewer_options(Profile::Gpu, false), "spice:enable=no"));
    }

    #[test]
    fn only_cpu_rendering_uses_the_low_cost_viewer_options() {
        let low_cost = "egl:multisample=no".to_owned();
        assert!(viewer_options(Profile::Normal, true).contains(&low_cost));
        assert!(!viewer_options(Profile::Gpu, true).contains(&low_cost));
    }
}
