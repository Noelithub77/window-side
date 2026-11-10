//! GNOME: Mutter's display configuration reports refresh rates. GNOME offers no API to raise another app's window.
use std::collections::HashMap;
use zbus::blocking::Connection;
use zbus::zvariant::OwnedValue;

type Properties = HashMap<String, OwnedValue>;
type Connector = (String, String, String, String);
type Mode = (String, i32, i32, f64, f64, Vec<f64>, Properties);
type Monitor = (Connector, Vec<Mode>, Properties);
type LogicalMonitor = (i32, i32, f64, u32, bool, Vec<Connector>, Properties);

fn current_state() -> Option<(Vec<Monitor>, Vec<LogicalMonitor>)> {
    let bus = Connection::session().ok()?;
    let name = "org.gnome.Mutter.DisplayConfig";
    let reply = bus.call_method(Some(name), "/org/gnome/Mutter/DisplayConfig", Some(name), "GetCurrentState", &()).ok()?;
    let (_, monitors, logical, _): (u32, Vec<Monitor>, Vec<LogicalMonitor>, Properties) = reply.body().deserialize().ok()?;
    Some((monitors, logical))
}

/// Scale of the primary logical monitor.
pub fn scale() -> Option<f64> {
    let (_, logical) = current_state()?;
    logical.iter().find(|monitor| monitor.4).or_else(|| logical.first()).map(|monitor| monitor.2)
}

/// Highest refresh rate among the modes Mutter marks current.
pub fn refresh_rate() -> Option<f64> {
    let (monitors, _) = current_state()?;
    monitors
        .iter()
        .flat_map(|(_, modes, _)| modes)
        .filter(|(.., properties)| properties.get("is-current").is_some_and(|current| current.downcast_ref::<bool>().unwrap_or(false)))
        .map(|mode| mode.3)
        .reduce(f64::max)
}
