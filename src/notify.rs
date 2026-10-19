use crate::process::{has_command, output};

#[derive(Clone, Copy)]
pub enum Urgency {
    Low,
    Normal,
    Critical,
}

impl Urgency {
    fn name(self) -> &'static str {
        match self {
            Urgency::Low => "low",
            Urgency::Normal => "normal",
            Urgency::Critical => "critical",
        }
    }
}

/// Best effort: a missing notifier never stops a VM operation.
pub fn send(urgency: Urgency, message: &str) {
    let level = urgency.name();
    if has_command("omarchy") {
        let _ = output("omarchy", &[
            "notification", "send", "--app-name", "window-side", "-u", level, "-g", "", "window-side", message,
        ]);
    } else if has_command("notify-send") {
        let _ = output("notify-send", &["-u", level, "-a", "window-side", "window-side", message]);
    }
}
