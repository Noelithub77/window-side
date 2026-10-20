use crate::process::{has_command, output};
use notify_rust::Notification;

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

impl From<Urgency> for notify_rust::Urgency {
    fn from(urgency: Urgency) -> Self {
        match urgency {
            Urgency::Low => Self::Low,
            Urgency::Normal => Self::Normal,
            Urgency::Critical => Self::Critical,
        }
    }
}

/// Best effort: a missing notifier never stops a VM operation.
pub fn send(urgency: Urgency, message: &str) {
    if has_command("omarchy") {
        let _ = output("omarchy", &[
            "notification", "send", "--app-name", "window-side", "-u", urgency.name(), "-g", "", "window-side", message,
        ]);
    } else {
        let _ = Notification::new().appname("window-side").summary("window-side").body(message).urgency(urgency.into()).show();
    }
}
