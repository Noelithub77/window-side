use crate::display::hyprland::{class_is, focus};
use crate::process::require;
use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

fn is_console(client: &Value, domain: &str) -> bool {
    (class_is(client, "window-side") || class_is(client, "virt-viewer"))
        && client["title"].as_str().is_some_and(|title| title.contains(domain))
}

/// Opens or focuses the SPICE console used when Linux keeps the GPU.
pub fn open(domain: &str) -> Result<()> {
    require("virt-viewer")?;
    if !virsh::is_running(domain) {
        bail!("Windows is not running.");
    }
    if focus(|client| is_console(client, domain)) {
        return Ok(());
    }
    // Shift+F12 reliably releases virt-viewer's pointer grab on Wayland.
    Command::new("setsid")
        .args(["-f", "virt-viewer", "--connect", virsh::URI, "--attach", "--class=window-side", "--name=window-side"])
        .args(["--auto-resize=never", "--hotkeys=release-cursor=shift+f12", "--reconnect", "--wait", "--zoom=100", domain])
        .status()
        .context("Could not start virt-viewer")?;
    for _ in 0..30 {
        if focus(|client| is_console(client, domain)) {
            break;
        }
        sleep(Duration::from_millis(100));
    }
    Ok(())
}
