//! Frame-timing probe for page transitions, enabled only with a fixture and an explicit output path.
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri::Manager;

#[derive(Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    name: String,
    runs: u32,
    frames: u32,
    mean_ms: f64,
    p95_ms: f64,
    max_ms: f64,
    /// Frames slower than 25 ms, 1.5 intervals of the 60 Hz test display.
    dropped: u32,
}

#[derive(Deserialize, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PerfReport {
    /// Median frame interval with nothing changing on screen.
    idle_ms: f64,
    scenarios: Vec<Scenario>,
    errors: Vec<String>,
}

#[tauri::command]
#[specta::specta]
pub fn perf_report(state: tauri::State<'_, super::State>, report: PerfReport) -> Result<(), String> {
    if state.lock()?.fixture.is_none() {
        return Err("Frame probes require a fixture".into());
    }
    let path = std::env::var("WS_GUI_PERF_OUTPUT").map_err(|_| "No frame probe requested")?;
    let text = serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| e.to_string())
}

pub fn start(app: &tauri::AppHandle) {
    if std::env::var_os("WS_GUI_PERF_OUTPUT").is_none() {
        return;
    }
    let Some(window) = app.get_webview_window("main") else {
        return;
    };
    if let Err(e) = window.eval(SCRIPT) {
        eprintln!("window-side frame probe: {e}");
    }
}

// Records requestAnimationFrame intervals around each page switch.
const SCRIPT: &str = r#"
(async () => {
  const errors = [];
  window.addEventListener('error', e => errors.push(e.message));
  const wait = ms => new Promise(resolve => setTimeout(resolve, ms));
  const nav = name => [...document.querySelectorAll('nav[aria-label="Pages"] button')].find(b => b.textContent.trim() === name);
  for (let i = 0; i < 100 && !nav('Settings'); i++) await wait(200);
  await wait(3000);
  const record = ms => new Promise(resolve => {
    const times = [];
    const end = performance.now() + ms;
    const tick = now => { times.push(now); if (now < end) requestAnimationFrame(tick); else resolve(times.slice(1).map((t, i) => t - times[i])); };
    requestAnimationFrame(tick);
  });
  const idle = (await record(1000)).sort((a, b) => a - b);
  const idleMs = idle[Math.floor(idle.length / 2)] || 0;
  const budget = 1000 / 60 * 1.5;
  const scenarios = [];
  const measure = async (name, runs, action) => {
    let all = [];
    for (let i = 0; i < runs; i++) {
      const pending = record(700);
      action();
      all = all.concat(await pending);
      await wait(500);
    }
    const sorted = [...all].sort((a, b) => a - b);
    const sum = all.reduce((a, b) => a + b, 0);
    scenarios.push({ name, runs, frames: all.length, meanMs: sum / Math.max(1, all.length),
      p95Ms: sorted[Math.floor(sorted.length * 0.95)] || 0, maxMs: sorted[sorted.length - 1] || 0,
      dropped: all.filter(t => t > budget).length });
  };
  await measure('idle', 3, () => {});
  for (let i = 0; i < 6; i++) {
    await measure('to settings', 1, () => nav('Settings')?.click());
    await measure('to overview', 1, () => nav('Overview')?.click());
  }
  const merged = {};
  for (const s of scenarios) {
    const m = merged[s.name] ??= { name: s.name, runs: 0, frames: 0, meanMs: 0, p95Ms: 0, maxMs: 0, dropped: 0 };
    m.meanMs = (m.meanMs * m.frames + s.meanMs * s.frames) / Math.max(1, m.frames + s.frames);
    m.runs += s.runs; m.frames += s.frames; m.dropped += s.dropped;
    m.p95Ms = Math.max(m.p95Ms, s.p95Ms); m.maxMs = Math.max(m.maxMs, s.maxMs);
  }
  await window.__TAURI_INTERNALS__.invoke('perf_report', { report: { idleMs, scenarios: Object.values(merged), errors } });
})();
"#;
