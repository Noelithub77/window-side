mod backend;
mod bindings;
mod commands;
mod fixture;
mod instance;
mod readings;
mod smoke;
mod tray;
mod types;

use std::{sync::{Arc, Mutex, MutexGuard, atomic::{AtomicBool, Ordering}, mpsc}, time::Duration};
use tauri::{Emitter, Manager};

struct Inner {
    fixture: Option<fixture::Fixture>, appearance: types::Appearance,
    history: readings::History, listeners: u32, credential: Option<bool>,
}
struct State { inner: Mutex<Inner>, operation: Arc<AtomicBool> }
impl State {
    fn lock(&self) -> Result<MutexGuard<'_, Inner>, String> { self.inner.lock().map_err(|_| "The window state is unavailable".into()) }
}

pub fn run() -> bool {
    match launch() { Ok(()) => true, Err(e) => { eprintln!("window-side: {e:#}"); false } }
}
fn launch() -> anyhow::Result<()> {
    let fixture = std::env::var("WS_GUI_FIXTURE").ok().map(|s| fixture::Fixture::new(&s)).transpose()?;
    let isolated = fixture.is_some();
    let (sender, receiver) = mpsc::channel();
    let instance = if isolated { None } else {
        match instance::claim(sender.clone()) {
            Ok(Some(c)) => Some(c), Ok(None) => return Ok(()),
            Err(e) => { eprintln!("window-side: running without single-instance support: {e}"); None }
        }
    };
    let state = State { inner: Mutex::new(Inner { fixture, appearance: if isolated { types::Appearance::Pastel } else { backend::appearance() }, history: readings::History::default(), listeners: 0, credential: None }), operation: Arc::new(AtomicBool::new(false)) };
    let tray = if isolated { None } else { tray::start(sender) };
    let online = tray.as_ref().map(|t| t.online.clone());
    let ipc = commands::builder();
    tauri::Builder::default()
        .manage(state)
        .invoke_handler(ipc.invoke_handler())
        .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event
                && online.as_ref().is_some_and(|o| o.load(Ordering::Relaxed)) {
                    api.prevent_close();
                    if let Err(e) = window.hide() { eprintln!("window-side: could not hide window: {e}"); window.app_handle().exit(1); }
            }
        })
        .setup(move |app| {
            if isolated { smoke::start(app.handle()); }
            let handle = app.handle().clone();
            let polling = handle.clone();
            std::thread::spawn(move || {
                let mut previous = None;
                while polling.get_webview_window("main").is_some() {
                    let started = std::time::Instant::now();
                    poll(&polling, &mut previous);
                    std::thread::sleep(Duration::from_secs(2).saturating_sub(started.elapsed()));
                }
            });
            std::thread::spawn(move || {
                let _instance = instance;
                let mut tick = 0_u32;
                loop {
                    if let Some(window) = handle.get_webview_window("main") {
                        while let Ok(command) = receiver.try_recv() {
                            match command {
                                tray::Command::Window => { let _ = window.show(); let _ = window.unminimize(); let _ = window.set_focus(); }
                                tray::Command::Quit => { handle.exit(0); return; }
                                tray::Command::Run(label, action) => { std::thread::spawn(move || { if let Err(e) = crate::cli::run(action) { crate::notify::send(crate::notify::Urgency::Normal, &format!("{label}: {e}")); } }); }
                            }
                        }
                    } else { return; }
                    if tick.is_multiple_of(10) && let Some(tray) = &tray { let config = crate::config::Config::load().ok(); tray.update(config.as_ref().map_or_else(String::new, |c| crate::vm::virsh::state(&c.vm.name)), config.is_some());
                    }
                    tick = tick.wrapping_add(1);
                    std::thread::sleep(Duration::from_millis(200));
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
fn poll(app: &tauri::AppHandle, previous: &mut Option<types::Status>) {
    let state = app.state::<State>();
    let Ok(mut inner) = state.lock() else { return };
    if inner.listeners == 0 || state.operation.load(Ordering::Relaxed) { return; }
    if let Some(f) = &mut inner.fixture {
        let status = f.status.clone();
        let metrics = f.metrics();
        drop(inner);
        if previous.as_ref() != Some(&status) { let _ = app.emit("status", &status); *previous = Some(status); }
        if metrics.available { let _ = app.emit("metrics", metrics); }
        return;
    }
    drop(inner);
    let running = crate::config::Config::load().ok().is_some_and(|c| crate::vm::virsh::is_running(&c.vm.name));
    let reading = running.then(readings::read).transpose().ok().flatten();
    let credential = state.lock().ok().and_then(|inner| inner.credential);
    let Ok(status) = readings::status(reading.as_ref(), credential) else { return };
    if let Ok(mut inner) = state.lock() {
        if let Some(r) = &reading {
            let was_available = inner.history.metrics.available;
            if inner.history.record(r) || was_available != inner.history.metrics.available { let _ = app.emit("metrics", inner.history.metrics.clone()); }
        } else { let changed = inner.history.metrics.available; inner.history.clear(); if changed { let _ = app.emit("metrics", inner.history.metrics.clone()); } }
    }
    if previous.as_ref() != Some(&status) { let _ = app.emit("status", &status); *previous = Some(status); }
}
