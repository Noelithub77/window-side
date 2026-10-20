//! One window-side app per session: launching `ws` again shows the running window.
use super::tray::Command;
use anyhow::Result;
use std::sync::mpsc::Sender;
use zbus::blocking::{Connection, Proxy, connection::Builder};

const NAME: &str = "io.github.Noelithub77.WindowSide";
const PATH: &str = "/io/github/Noelithub77/WindowSide";

struct Instance {
    sender: Sender<Command>,
}

#[zbus::interface(name = "io.github.Noelithub77.WindowSide")]
impl Instance {
    fn show(&self) {
        let _ = self.sender.send(Command::Window);
    }
}

/// The connection that owns the session name, or `None` after asking the running app to show itself.
pub(super) fn claim(sender: Sender<Command>) -> Result<Option<Connection>> {
    // zbus requests names with replacement on by default; a second launch must not take over.
    let builder = Builder::session()?.serve_at(PATH, Instance { sender })?.name(NAME)?;
    match builder.allow_name_replacements(false).replace_existing_names(false).build() {
        Ok(connection) => Ok(Some(connection)),
        Err(zbus::Error::NameTaken) => {
            let connection = Connection::session()?;
            Proxy::new(&connection, NAME, PATH, NAME)?.call_method("Show", &())?;
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}
