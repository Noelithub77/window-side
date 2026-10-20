use super::{State, backend, bindings, types::*};
use tauri::State as Managed;

type Reply<T> = Result<T, String>;
fn error(e: impl std::fmt::Display) -> String { e.to_string().lines().next().unwrap_or("Something went wrong").chars().take(240).collect() }

#[tauri::command]
#[specta::specta]
pub async fn shell(state: Managed<'_, State>) -> Reply<Shell> {
    let inner = state.lock()?;
    Ok(inner.fixture.as_ref().map_or_else(|| backend::shell(inner.appearance), |f| f.shell()))
}
#[tauri::command]
#[specta::specta]
pub async fn detect(state: Managed<'_, State>) -> Reply<Detection> {
    if let Some(f) = &state.lock()?.fixture { return Ok(f.detection()); }
    tauri::async_runtime::spawn_blocking(backend::detection).await.map_err(error)?.map_err(error)
}
#[tauri::command]
#[specta::specta]
pub async fn plan(state: Managed<'_, State>, choices: Choices) -> Reply<Plan> {
    if let Some(f) = &state.lock()?.fixture { return f.plan(choices).map_err(error); }
    tauri::async_runtime::spawn_blocking(move || backend::core_plan(choices).map(backend::plan)).await.map_err(error)?.map_err(error)
}
#[tauri::command]
#[specta::specta]
pub async fn apply(state: Managed<'_, State>, choices: Choices) -> Reply<()> {
    { let mut inner = state.lock()?; if let Some(f) = &mut inner.fixture { return f.apply(choices).map_err(error); } }
    let guard = Operation::claim(state.operation.clone())?;
    tauri::async_runtime::spawn_blocking(move || { let _guard = guard; backend::core_plan(choices).and_then(|p| crate::setup::apply(&p)) }).await.map_err(error)?.map_err(error)
}
#[tauri::command]
#[specta::specta]
pub async fn status(state: Managed<'_, State>) -> Reply<Status> {
    { let inner = state.lock()?; if let Some(f) = &inner.fixture { return Ok(f.status.clone()); } }
    let result = tauri::async_runtime::spawn_blocking(|| {
        let credential = crate::config::Config::load().ok().map(|c| crate::guest::sign_in::is_stored(&c.windows.user));
        super::readings::status(None, credential)
    }).await.map_err(error)?.map_err(error)?;
    state.lock()?.credential = result.credential_saved;
    Ok(result)
}
#[tauri::command]
#[specta::specta]
pub async fn metrics(state: Managed<'_, State>) -> Reply<Metrics> {
    let mut inner = state.lock()?;
    if let Some(f) = &mut inner.fixture { return Ok(f.metrics()); }
    Ok(inner.history.metrics.clone())
}
#[tauri::command]
#[specta::specta]
pub async fn run(state: Managed<'_, State>, action: Action) -> Reply<()> {
    { let mut inner = state.lock()?; if let Some(f) = &mut inner.fixture { f.action(action); return Ok(()); } }
    let guard = Operation::claim(state.operation.clone())?;
    tauri::async_runtime::spawn_blocking(move || { let _guard = guard; backend::action(action) }).await.map_err(error)?.map_err(error)
}
#[tauri::command]
#[specta::specta]
pub async fn save_credential(state: Managed<'_, State>, secret: String) -> Reply<()> {
    if secret.is_empty() { return Err("Enter the Windows password or PIN".into()); }
    { let mut inner = state.lock()?; if let Some(f) = &mut inner.fixture { f.status.credential_saved = Some(true); return Ok(()); } }
    tauri::async_runtime::spawn_blocking(move || { let c = crate::config::Config::load()?; crate::guest::sign_in::store(&c.windows.user, &secret) }).await.map_err(error)?.map_err(error)?;
    state.lock()?.credential = Some(true);
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub async fn forget_credential(state: Managed<'_, State>) -> Reply<()> {
    { let mut inner = state.lock()?; if let Some(f) = &mut inner.fixture { f.status.credential_saved = Some(false); return Ok(()); } }
    tauri::async_runtime::spawn_blocking(|| { let c = crate::config::Config::load()?; crate::guest::sign_in::forget(&c.windows.user) }).await.map_err(error)?.map_err(error)?;
    state.lock()?.credential = Some(false);
    Ok(())
}
#[tauri::command]
#[specta::specta]
pub async fn set_appearance(state: Managed<'_, State>, appearance: Appearance) -> Reply<Shell> {
    { let mut inner = state.lock()?; if let Some(f) = &mut inner.fixture { f.appearance = appearance; return Ok(f.shell()); } }
    tauri::async_runtime::spawn_blocking(move || backend::save_appearance(appearance)).await.map_err(error)?.map_err(error)?;
    state.lock()?.appearance = appearance;
    Ok(backend::shell(appearance))
}
#[tauri::command]
#[specta::specta]
pub async fn agent_prompt(state: Managed<'_, State>, agent: AgentId) -> Reply<String> {
    let _ = (state, agent);
    Ok(crate::agents::SETUP_PROMPT.into())
}
#[tauri::command]
#[specta::specta]
pub async fn launch_agent(state: Managed<'_, State>, agent: AgentId) -> Reply<()> {
    if state.lock()?.fixture.is_some() { return Ok(()); }
    let agent = match agent {
        AgentId::Claude => crate::agents::Agent::Claude,
        AgentId::Codex => crate::agents::Agent::Codex,
        AgentId::Opencode => crate::agents::Agent::OpenCode,
    };
    tauri::async_runtime::spawn_blocking(move || crate::agents::launch(agent)).await.map_err(error)?.map_err(error)
}
// Internal subscription accounting; not product state or a public UI command.
#[tauri::command]
#[specta::specta]
pub fn subscriptions(state: Managed<'_, State>, count: u32) -> Reply<()> {
    state.lock()?.listeners = count;
    Ok(())
}

pub fn builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::new().typed_error_impl(ERROR_RUNTIME).commands(tauri_specta::collect_commands![shell, detect, plan, apply, status, metrics, run, save_credential, forget_credential, set_appearance, agent_prompt, launch_agent, subscriptions, copy_text, super::smoke::smoke_report]).typ::<bindings::Events>()
}

struct Operation(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl Operation {
    fn claim(flag: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Reply<Self> {
        flag.compare_exchange(false, true, std::sync::atomic::Ordering::AcqRel, std::sync::atomic::Ordering::Acquire).map_err(|_| "Another action is in progress".to_owned())?;
        Ok(Self(flag))
    }
}
impl Drop for Operation { fn drop(&mut self) { self.0.store(false, std::sync::atomic::Ordering::Release); } }

// All command errors are strings; narrow unknown IPC errors without generated casts.
const ERROR_RUNTIME: &str = r#"
export type Result<T, E> = { status: "ok"; data: T } | { status: "error"; error: E };
async function typedError<T>(result: Promise<T>): Promise<Result<T, string>> {
    try { return { status: "ok", data: await result }; }
    catch (error: unknown) {
        if (typeof error === "string") return { status: "error", error };
        throw error;
    }
}
"#;

#[tauri::command]
#[specta::specta]
pub fn copy_text(app: tauri::AppHandle, text: String) -> Reply<()> {
    use tauri_plugin_clipboard_manager::ClipboardExt;
    app.clipboard().write_text(text).map_err(|_| "Could not copy the prompt".into())
}
