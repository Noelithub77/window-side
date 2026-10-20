//! KDE Plasma: a KWin script raises windows; kscreen-doctor reports refresh rates.
use crate::config::cache_dir;
use crate::process::text;
use anyhow::Result;
use serde_json::Value;
use std::fs;
use zbus::blocking::Connection;

const KWIN: &str = "org.kde.KWin";
const SCRIPTING: &str = "org.kde.kwin.Scripting";
const SCRIPT: &str = "window-side-raise";

/// Restores the first matching window onto the current desktop and activates it (Plasma 5 and 6).
fn script(classes: &[&str], name: Option<&str>) -> Result<String> {
    Ok(format!(
        "const classes = {}; const name = {};\n\
         const plasma6 = typeof workspace.windowList === \"function\";\n\
         const windows = plasma6 ? workspace.windowList() : workspace.clientList();\n\
         for (const window of windows) {{\n\
           if (!classes.includes(String(window.resourceClass)) || (name !== null && !window.caption.includes(name))) continue;\n\
           window.minimized = false;\n\
           if (plasma6) {{ window.desktops = [workspace.currentDesktop]; workspace.activeWindow = window; }}\n\
           else {{ window.desktop = workspace.currentDesktop; workspace.activeClient = window; }}\n\
           break;\n\
         }}\n",
        serde_json::to_string(classes)?,
        serde_json::to_string(&name)?,
    ))
}

pub fn raise(classes: &[&str], name: Option<&str>) -> Result<()> {
    let path = cache_dir().join("kwin-raise.js");
    fs::create_dir_all(cache_dir())?;
    fs::write(&path, script(classes, name)?)?;
    let bus = Connection::session()?;
    // KWin reads the file after `run` returns, so the previous run is unloaded here rather than right after it.
    let _ = bus.call_method(Some(KWIN), "/Scripting", Some(SCRIPTING), "unloadScript", &(SCRIPT,));
    let reply = bus.call_method(Some(KWIN), "/Scripting", Some(SCRIPTING), "loadScript", &(path.display().to_string(), SCRIPT))?;
    let id: i32 = reply.body().deserialize()?;
    bus.call_method(Some(KWIN), format!("/Scripting/Script{id}").as_str(), Some("org.kde.kwin.Script"), "run", &())?;
    Ok(())
}

/// Highest current refresh rate in `kscreen-doctor -j` output.
fn current_rate(outputs: &Value) -> Option<f64> {
    outputs["outputs"]
        .as_array()?
        .iter()
        .filter(|output| output["enabled"].as_bool() == Some(true))
        .filter_map(|output| {
            let current = &output["currentModeId"];
            output["modes"].as_array()?.iter().find(|mode| &mode["id"] == current)?["refreshRate"].as_f64()
        })
        .reduce(f64::max)
}

pub fn refresh_rate() -> Option<f64> {
    let outputs = text("kscreen-doctor", &["-j"]).ok()?;
    current_rate(&serde_json::from_str(&outputs).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn current_modes_of_enabled_outputs_set_the_rate() {
        let outputs = json!({ "outputs": [
            { "enabled": true, "currentModeId": "2", "modes": [{ "id": "1", "refreshRate": 60.0 }, { "id": "2", "refreshRate": 143.98 }] },
            { "enabled": false, "currentModeId": "1", "modes": [{ "id": "1", "refreshRate": 240.0 }] },
        ]});
        assert_eq!(current_rate(&outputs), Some(143.98));
        let script = script(&["window-side", "virt-viewer"], Some("win\"11")).unwrap_or_default();
        assert!(script.contains(r#"const classes = ["window-side","virt-viewer"]; const name = "win\"11";"#));
    }
}
