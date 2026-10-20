//! Native rendering probe, enabled only with a fixture and an explicit output path.
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Report { webgl: bool, lottie: bool, clipboard: bool, title: String, errors: Vec<String> }
#[tauri::command]
#[specta::specta]
pub fn smoke_report(state: tauri::State<'_, super::State>, report: Report) -> Result<(), String> {
    if state.lock()?.fixture.is_none() { return Err("Rendering probes require a fixture".into()); }
    let path = std::env::var("WS_GUI_SMOKE_OUTPUT").map_err(|_| "No rendering probe requested")?;
    std::fs::write(path, serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}

pub fn start(app: &tauri::AppHandle) {
    if std::env::var_os("WS_GUI_SMOKE_OUTPUT").is_none() { return; }
    let Some(window) = app.get_webview_window("main") else { return };
    let script = r#"
(async () => {
  const errors = [];
  window.addEventListener('error', e => errors.push(e.message));
  window.addEventListener('unhandledrejection', e => errors.push(String(e.reason)));
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  for (let i = 0; i < 100 && !document.querySelector('button'); i++) await wait(200);
  await wait(2500);
  const button = name => [...document.querySelectorAll('button')].find(b => !b.disabled && b.textContent.trim() === name);
  button('Get started')?.click();
  await wait(1000);
  for (const name of ['Continue', 'Continue', 'Review']) { button(name)?.click(); await wait(1100); }
  button('Apply')?.click();
  await wait(350);
  const canvases = [...document.querySelectorAll('canvas')];
  const webgl = canvases.some(c => { try { return Boolean(c.getContext('webgl2') || c.getContext('webgl')); } catch { return false; } });
  const lottie = canvases.some(c => {
    try {
      const ctx = c.getContext('2d'); if (!ctx) return false;
      return ctx.getImageData(0,0,c.width,c.height).data.some((v,i) => i % 4 === 3 && v > 0);
    } catch { return false; }
  });
  let clipboard = false;
  try { await navigator.clipboard.writeText('window-side isolated clipboard smoke'); clipboard = true; } catch (e) { errors.push('clipboard: ' + String(e)); }
  await window.__TAURI_INTERNALS__.invoke('smoke_report', { report: { webgl, lottie, clipboard, title: document.title, errors } });
})();
"#;
    if let Err(e) = window.eval(script) { eprintln!("window-side rendering probe: {e}"); }
}
use tauri::Manager;
