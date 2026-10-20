//! Status-bar tray: Windows' state, start, open and shut down, the window and quit.
use super::model::state_label;
use crate::cli::Action;
use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::StandardItem;
use ksni::{MenuItem, OfflineReason, ToolTip, Tray};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

/// Requests from the tray and from a second `ws` launch, handled on the app thread.
pub(super) enum Command {
    Window,
    Run(&'static str, Action),
    Quit,
}

struct WindowSideTray {
    status: String,
    configured: bool,
    sender: Sender<Command>,
    online: Arc<AtomicBool>,
}

impl WindowSideTray {
    fn item(label: &str, enabled: bool, command: fn() -> Command) -> MenuItem<Self> {
        StandardItem {
            label: label.into(),
            enabled,
            activate: Box::new(move |tray: &mut Self| {
                let _ = tray.sender.send(command());
            }),
            ..Default::default()
        }
        .into()
    }
}

impl Tray for WindowSideTray {
    fn id(&self) -> String {
        "window-side".into()
    }

    fn title(&self) -> String {
        "window-side".into()
    }

    fn icon_name(&self) -> String {
        "computer".into()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip { title: "window-side".into(), description: format!("Windows · {}", state_label(&self.status)), ..Default::default() }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(Command::Window);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let running = self.status == "running";
        let header = StandardItem { label: format!("Windows · {}", state_label(&self.status)), enabled: false, ..Default::default() };
        let mut items = vec![header.into(), MenuItem::Separator];
        if running {
            items.push(Self::item("Open Windows", true, || Command::Run("Opening Windows", Action::Console)));
            items.push(Self::item("Shut down", true, || Command::Run("Shutting down", Action::Stop)));
        } else {
            let startable = self.configured && self.status == "shut off";
            items.push(Self::item("Start Windows", startable, || Command::Run("Starting Windows", Action::Launch)));
        }
        items.extend([
            MenuItem::Separator,
            Self::item("Open window-side", true, || Command::Window),
            Self::item("Quit", true, || Command::Quit),
        ]);
        items
    }

    fn watcher_online(&self) {
        self.online.store(true, Ordering::Relaxed);
    }

    /// Keeps the item registered for when the bar comes back; meanwhile closing the window quits.
    fn watcher_offline(&self, _reason: OfflineReason) -> bool {
        self.online.store(false, Ordering::Relaxed);
        true
    }
}

pub(super) struct TrayHandle {
    handle: Handle<WindowSideTray>,
    /// Whether a bar shows the tray, so closing the window can leave the app running.
    pub(super) online: Arc<AtomicBool>,
}

impl TrayHandle {
    pub(super) fn update(&self, status: String, configured: bool) {
        self.handle.update(|tray| {
            tray.status = status;
            tray.configured = configured;
        });
    }
}

/// `None` when no bar hosts tray items, such as GNOME without the AppIndicator extension.
pub(super) fn start(sender: Sender<Command>) -> Option<TrayHandle> {
    let online = Arc::new(AtomicBool::new(true));
    let tray = WindowSideTray { status: String::new(), configured: false, sender, online: online.clone() };
    match tray.spawn() {
        Ok(handle) => Some(TrayHandle { handle, online }),
        Err(ksni::Error::Dbus(error)) => {
            eprintln!("window-side tray: {error}");
            None
        }
        Err(_) => None,
    }
}
