//! The running desktop: raises windows ws opened and reads the refresh rate on Hyprland, KDE and GNOME.
use super::{gnome, hyprland, kde};
use crate::process::output;
use std::env;
use std::fs;
use std::os::unix::fs::MetadataExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Desktop {
    Hyprland,
    Kde,
    Gnome,
    Other,
}

pub fn current() -> Desktop {
    if env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_some() {
        return Desktop::Hyprland;
    }
    let names = env::var("XDG_CURRENT_DESKTOP").unwrap_or_default().to_ascii_lowercase();
    let has = |name: &str| names.split(':').any(|part| part == name);
    if has("kde") {
        Desktop::Kde
    } else if has("gnome") {
        Desktop::Gnome
    } else {
        Desktop::Other
    }
}

/// A window ws opens: its program, window classes and, for a VM viewer, the VM name in its title and arguments.
pub struct Target<'a> {
    pub program: &'a str,
    pub classes: &'a [&'a str],
    pub name: Option<&'a str>,
}

/// Brings an open window of `target` forward. False when none is open or this desktop cannot raise it.
pub fn raise(target: &Target) -> bool {
    match current() {
        Desktop::Hyprland => hyprland::focus(|client| {
            target.classes.iter().any(|class| hyprland::class_is(client, class))
                && target.name.is_none_or(|name| client["title"].as_str().is_some_and(|title| title.contains(name)))
        }),
        Desktop::Kde => !processes(target).is_empty() && kde::raise(target.classes, target.name).is_ok(),
        Desktop::Gnome | Desktop::Other => false,
    }
}

/// Closes instances that could not be raised, so a fresh window opens in front.
pub fn close(target: &Target) {
    for pid in processes(target) {
        let _ = output("kill", &[&pid]);
    }
}

/// This user's processes running `target`.
fn processes(target: &Target) -> Vec<String> {
    let Ok(uid) = fs::metadata("/proc/self").map(|meta| meta.uid()) else { return Vec::new() };
    let Ok(entries) = fs::read_dir("/proc") else { return Vec::new() };
    entries
        .flatten()
        .filter(|entry| entry.metadata().is_ok_and(|meta| meta.uid() == uid))
        .filter_map(|entry| {
            let pid = entry.file_name().into_string().ok().filter(|pid| pid.chars().all(|c| c.is_ascii_digit()))?;
            let cmdline = fs::read(entry.path().join("cmdline")).ok()?;
            let args: Vec<_> = cmdline.split(|byte| *byte == 0).map(String::from_utf8_lossy).collect();
            let executable = fs::read_link(entry.path().join("exe")).ok();
            (program_matches(executable.as_deref(), args.first()?, target.program)
                && target.name.is_none_or(|name| args.iter().any(|arg| arg == name))).then_some(pid)
        })
        .collect()
}

fn program_matches(executable: Option<&std::path::Path>, command: &str, program: &str) -> bool {
    // Looking Glass rewrites argv[0] to show its connection state.
    executable.and_then(std::path::Path::file_name).is_some_and(|name| name == program)
        || command.rsplit('/').next() == Some(program)
}

/// Highest refresh rate among the connected monitors, formatted as Windows expects (e.g. `165.003`).
pub fn refresh_rate() -> String {
    let rate = match current() {
        Desktop::Hyprland => hyprland::refresh_rate(),
        Desktop::Kde => kde::refresh_rate(),
        Desktop::Gnome => gnome::refresh_rate(),
        Desktop::Other => None,
    };
    format!("{:.3}", rate.filter(|rate| *rate > 0.0).unwrap_or(60.0))
}

/// Windows' scaling steps; the host scale snaps to the nearest one.
const WINDOWS_SCALES: [u32; 12] = [100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500];

/// The host display's scale as the Windows scaling percentage closest to it.
pub fn windows_scale() -> u32 {
    let scale = match current() {
        Desktop::Hyprland => hyprland::scale(),
        Desktop::Kde => kde::scale(),
        Desktop::Gnome => gnome::scale(),
        Desktop::Other => None,
    };
    nearest_windows_scale(scale.filter(|scale| *scale > 0.0).unwrap_or(1.0))
}

fn nearest_windows_scale(scale: f64) -> u32 {
    let wanted = scale * 100.0;
    WINDOWS_SCALES.into_iter().min_by(|a, b| (f64::from(*a) - wanted).abs().total_cmp(&(f64::from(*b) - wanted).abs())).unwrap_or(100)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn host_scale_snaps_to_a_windows_step() {
        assert_eq!(nearest_windows_scale(1.0), 100);
        assert_eq!(nearest_windows_scale(1.25), 125);
        assert_eq!(nearest_windows_scale(1.333), 125);
        assert_eq!(nearest_windows_scale(1.6), 150);
        assert_eq!(nearest_windows_scale(2.0), 200);
        assert_eq!(nearest_windows_scale(8.0), 500);
    }

    #[test]
    fn viewer_identity_survives_process_title_changes() {
        assert!(program_matches(Some(Path::new("/private/window-side-viewer")), "Looking Glass Client B7-829", "window-side-viewer"));
        assert!(!program_matches(Some(Path::new("/usr/bin/looking-glass-client")), "Looking Glass Client B7-829", "window-side-viewer"));
        assert!(program_matches(None, "/usr/bin/virt-viewer", "virt-viewer"));
    }
}
