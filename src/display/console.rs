use crate::display::desktop::{self, Desktop, Target};
use crate::process::require;
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn console(domain: &str) -> Target<'_> {
    Target { program: "virt-viewer", classes: &["window-side", "virt-viewer"], name: Some(domain) }
}

/// Opens or focuses the SPICE console used when Linux keeps the GPU.
pub fn open(domain: &str) -> Result<()> {
    require("virt-viewer")?;
    if !virsh::is_running(domain) {
        bail!("Windows is not running.");
    }
    let target = console(domain);
    if desktop::raise(&target) {
        return Ok(());
    }
    desktop::close(&target);
    // Shift+F12 reliably releases virt-viewer's pointer grab on Wayland.
    Command::new("setsid")
        .args(["-f", "virt-viewer", "--connect", virsh::URI, "--attach", "--class=window-side", "--name=window-side"])
        .args(["--auto-resize=never", "--hotkeys=release-cursor=shift+f12", "--reconnect", "--wait", "--zoom=100", domain])
        .status()
        .context("Could not start virt-viewer")?;
    // Hyprland may open it on another workspace and tiled; bring it here full screen.
    if desktop::current() == Desktop::Hyprland {
        for _ in 0..30 {
            if desktop::raise(&target) {
                break;
            }
            sleep(Duration::from_millis(100));
        }
    }
    Ok(())
}
