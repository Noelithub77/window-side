use super::types;
use specta::Type;
#[derive(Type)]
#[allow(dead_code)]
pub struct Events { pub status: types::Status, pub metrics: types::Metrics }

#[cfg(test)]
mod tests {
    #[test]
    fn export_bindings() -> anyhow::Result<()> {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("ui/src/bindings.ts");
        let temp = root.join("target/bindings.ts");
        super::super::commands::builder().export(specta_typescript::Typescript::default(), &temp)?;
        let mut generated = std::fs::read_to_string(&temp)?;
        // The shared runtime is specialised to string errors, matching every command.
        generated = generated.replace(", string>(__TAURI_INVOKE", ">(__TAURI_INVOKE");
        // Upstream's unused assertion contains `any`; the specialised runtime is checked directly.
        generated = generated.lines().filter(|line| !line.starts_with("const _assertTypedErrorFollowsContract:")).collect::<Vec<_>>().join("\n") + "\n";
        generated.push_str("\ntype CommandData<T> = T extends null ? void : T\nexport type Commands = { [K in Exclude<keyof typeof commands, 'subscriptions' | 'smokeReport'>]: (...args: Parameters<(typeof commands)[K]>) => Promise<CommandData<Extract<Awaited<ReturnType<(typeof commands)[K]>>, { status: 'ok' }>['data']>> }\n");
        if std::env::var_os("WS_UPDATE_BINDINGS").is_some() { std::fs::write(&path, &generated)?; }
        anyhow::ensure!(std::fs::read_to_string(&path).ok().as_deref() == Some(&generated), "Bindings are stale; run ./ops ui:bindings");
        Ok(())
    }
}
