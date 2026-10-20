//! Status-bar tray: Windows' state, start, open and shut down, the window and quit.
fn state_label(state: &str) -> &str {
    match state {
        "running" => "Running",
        "shut off" => "Stopped",
        "paused" => "Paused",
        "in shutdown" => "Stopping",
        _ => "Unavailable",
    }
}
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
    icons: Vec<ksni::Icon>,
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

    // A missing theme name can override valid pixmaps in tray hosts.
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        self.icons.clone()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "window-side".into(),
            description: format!("Windows · {}", state_label(&self.status)),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(Command::Window);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let running = self.status == "running";
        let header = StandardItem {
            label: format!("Windows · {}", state_label(&self.status)),
            enabled: false,
            ..Default::default()
        };
        let mut items = vec![header.into(), MenuItem::Separator];
        if running {
            items.push(Self::item("Open Windows", true, || {
                Command::Run("Opening Windows", Action::Console)
            }));
            items.push(Self::item("Shut down", true, || {
                Command::Run("Shutting down", Action::Stop)
            }));
        } else {
            let startable = self.configured && self.status == "shut off";
            items.push(Self::item("Start Windows", startable, || {
                Command::Run("Starting Windows", Action::Launch)
            }));
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
    let tray = WindowSideTray {
        status: String::new(),
        configured: false,
        sender,
        online: online.clone(),
        icons: match embedded_icons() {
            Ok(icons) => icons,
            Err(error) => {
                eprintln!("window-side tray: could not decode embedded icons: {error}");
                Vec::new()
            }
        },
    };
    match tray.spawn() {
        Ok(handle) => Some(TrayHandle { handle, online }),
        Err(ksni::Error::Dbus(error)) => {
            eprintln!("window-side tray: {error}");
            None
        }
        Err(_) => None,
    }
}

fn decode_icon(bytes: &[u8]) -> anyhow::Result<ksni::Icon> {
    let image = tauri::image::Image::from_bytes(bytes)?;
    let mut data = image.rgba().to_vec();
    for pixel in data.chunks_exact_mut(4) {
        pixel.rotate_right(1);
    }
    Ok(ksni::Icon {
        width: i32::try_from(image.width())?,
        height: i32::try_from(image.height())?,
        data,
    })
}

fn embedded_icons() -> anyhow::Result<Vec<ksni::Icon>> {
    [
        include_bytes!("../../icons/hicolor/22x22/apps/window-side.png").as_slice(),
        include_bytes!("../../icons/hicolor/32x32/apps/window-side.png").as_slice(),
        include_bytes!("../../icons/hicolor/48x48/apps/window-side.png").as_slice(),
    ]
    .into_iter()
    .map(decode_icon)
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_pixmaps_are_network_order_argb() -> anyhow::Result<()> {
        let icons = embedded_icons()?;
        assert_eq!(icons.iter().map(|i| i.width).collect::<Vec<_>>(), [22, 32, 48]);
        for icon in icons {
            assert_eq!(icon.width, icon.height);
            assert_eq!(icon.data.len(), (icon.width * icon.height * 4) as usize);
            assert!(icon.data.chunks_exact(4).any(|p| p[0] > 0));
        }
        let bytes = include_bytes!("../../icons/hicolor/22x22/apps/window-side.png");
        let source = tauri::image::Image::from_bytes(bytes)?;
        let decoded = decode_icon(bytes)?;
        for (rgba, argb) in source.rgba().chunks_exact(4).zip(decoded.data.chunks_exact(4)) {
            assert_eq!(argb, [rgba[3], rgba[0], rgba[1], rgba[2]]);
        }
        assert!(decode_icon(b"invalid PNG").is_err());
        Ok(())
    }

    #[test]
    fn context_uses_full_window_icon_without_bundling() -> anyhow::Result<()> {
        let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
        let icon = context
            .default_window_icon()
            .ok_or_else(|| anyhow::anyhow!("Missing window icon"))?;
        let expected = tauri::image::Image::from_bytes(include_bytes!("../../icons/icon.png"))?;
        assert_eq!((icon.width(), icon.height()), (512, 512));
        assert_eq!(icon.rgba(), expected.rgba());
        Ok(())
    }
}
