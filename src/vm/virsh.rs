use crate::process::{output, output_with_input, text};
use anyhow::{Context, Result, bail};
use serde_json::Value;

pub const URI: &str = "qemu:///system";

pub fn run(args: &[&str]) -> Result<String> {
    let mut full = vec!["-c", URI];
    full.extend_from_slice(args);
    text("virsh", &full)
}

/// Runs virsh commands from stdin, keeping their arguments out of the process list.
pub fn run_script(script: &str) -> Result<()> {
    let result = output_with_input("virsh", &["-q", "-c", URI], script)?;
    // Batch mode keeps going after a failed command, so check stderr as well.
    if !result.status.success() || String::from_utf8_lossy(&result.stderr).to_lowercase().contains("error") {
        bail!("virsh rejected the command");
    }
    Ok(())
}

pub fn state(name: &str) -> String {
    output("virsh", &["-c", URI, "domstate", name])
        .ok()
        .filter(|result| result.status.success())
        .and_then(|result| String::from_utf8_lossy(&result.stdout).lines().next().map(str::to_lowercase))
        .unwrap_or_default()
}

pub fn exists(name: &str) -> bool {
    run(&["dominfo", name]).is_ok()
}

pub fn is_running(name: &str) -> bool {
    matches!(state(name).as_str(), "running" | "paused" | "in shutdown")
}

/// Sends one QEMU guest agent command and returns its `return` value.
pub fn agent(name: &str, command: &Value, timeout: u64) -> Result<Value> {
    let request = command.to_string();
    let timeout = timeout.to_string();
    let reply = run(&["qemu-agent-command", name, &request, "--timeout", &timeout])
        .context("The Windows guest agent did not respond")?;
    let mut reply: Value = serde_json::from_str(&reply).context("The Windows guest agent sent invalid JSON")?;
    Ok(reply["return"].take())
}
