use crate::process::{has_command, output, text};
use serde_json::Value;

/// Moves the first matching window to the current workspace and focuses it.
/// Returns false when Hyprland is absent or no window matches.
pub fn focus(matches: impl Fn(&Value) -> bool) -> bool {
    if !has_command("hyprctl") {
        return false;
    }
    let Ok(clients) = text("hyprctl", &["clients", "-j"]) else { return false };
    let Ok(Value::Array(clients)) = serde_json::from_str::<Value>(&clients) else { return false };
    let Some(address) = clients.iter().find(|client| matches(client)).and_then(|client| client["address"].as_str()) else {
        return false;
    };
    let workspace = text("hyprctl", &["activeworkspace", "-j"])
        .ok()
        .and_then(|active| serde_json::from_str::<Value>(&active).ok())
        .and_then(|active| active["id"].as_i64());
    let window = format!("address:{address}");
    // Bring Windows to the workspace in use rather than sending the user elsewhere.
    if let Some(workspace) = workspace {
        dispatch(&format!("hl.dsp.window.move({{ workspace = \"{workspace}\", window = \"{window}\" }})"));
    }
    dispatch(&format!(
        "hl.dsp.window.fullscreen_state({{ internal = 0, client = 0, action = \"set\", window = \"{window}\" }})"
    ));
    dispatch(&format!("hl.dsp.focus({{ window = \"{window}\" }})"));
    true
}

fn dispatch(action: &str) {
    let _ = output("hyprctl", &["eval", &format!("hl.dispatch({action})")]);
}

pub fn class_is(client: &Value, class: &str) -> bool {
    client["class"].as_str() == Some(class)
}

/// Highest refresh rate among connected monitors.
pub fn refresh_rate() -> Option<f64> {
    let monitors = text("hyprctl", &["monitors", "-j"]).ok()?;
    let Value::Array(monitors) = serde_json::from_str::<Value>(&monitors).ok()? else { return None };
    monitors.iter().filter_map(|monitor| monitor["refreshRate"].as_f64()).reduce(f64::max)
}
