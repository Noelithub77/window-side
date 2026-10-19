use crate::vm::virsh;
use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub const GUEST_DIR: &str = r"C:\ProgramData\window-side";
const CHUNK: usize = 48 * 1024;

pub struct Run {
    pub code: i64,
    pub stdout: String,
    pub stderr: String,
}

/// Runs PowerShell as SYSTEM. `arguments` follow the script block, e.g. `-Width 2560`.
pub fn powershell(domain: &str, script: &str, arguments: &str, timeout: Duration) -> Result<Run> {
    let wrapped = format!("$ProgressPreference='SilentlyContinue'\n& {{\n{script}\n}} {arguments}");
    let utf16: Vec<u8> = wrapped.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let encoded = STANDARD.encode(utf16);
    let started = virsh::agent(domain, &json!({
        "execute": "guest-exec",
        "arguments": {
            "path": "powershell.exe",
            "capture-output": true,
            "arg": ["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-EncodedCommand", encoded],
        },
    }), 30)?;
    let pid = started["pid"].as_i64().context("The guest agent did not return a process id")?;
    let deadline = Instant::now() + timeout;
    let status = loop {
        let status = virsh::agent(domain, &json!({"execute": "guest-exec-status", "arguments": {"pid": pid}}), 30)?;
        if status["exited"].as_bool() == Some(true) {
            break status;
        }
        if Instant::now() > deadline {
            bail!("Guest command {pid} is still running after {}s; it was not stopped", timeout.as_secs());
        }
        sleep(Duration::from_millis(500));
    };
    Ok(Run {
        code: status["exitcode"].as_i64().unwrap_or(1),
        stdout: decode(&status["out-data"]),
        stderr: decode(&status["err-data"]),
    })
}

fn decode(value: &Value) -> String {
    let bytes = STANDARD.decode(value.as_str().unwrap_or_default()).unwrap_or_default();
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Like `powershell`, but a non-zero exit becomes an error carrying its stderr.
pub fn powershell_ok(domain: &str, script: &str, arguments: &str, timeout: Duration) -> Result<String> {
    let run = powershell(domain, script, arguments, timeout)?;
    if run.code != 0 {
        let error = run.stderr.trim();
        bail!("{}", if error.is_empty() { format!("Windows command exited with {}", run.code) } else { error.to_owned() });
    }
    Ok(run.stdout)
}

pub fn write_file(domain: &str, target: &str, data: &[u8]) -> Result<()> {
    let handle = virsh::agent(domain, &json!({"execute": "guest-file-open", "arguments": {"path": target, "mode": "wb"}}), 30)?;
    let written = data.chunks(CHUNK).try_for_each(|chunk| {
        let request = json!({"execute": "guest-file-write", "arguments": {"handle": handle, "buf-b64": STANDARD.encode(chunk)}});
        virsh::agent(domain, &request, 30).map(drop)
    });
    let closed = virsh::agent(domain, &json!({"execute": "guest-file-close", "arguments": {"handle": handle}}), 30);
    written.and(closed.map(drop))
}

/// Copies an embedded script into Windows so it can relaunch itself in the user session.
pub fn install_script(domain: &str, name: &str, source: &str) -> Result<String> {
    powershell_ok(domain, &format!("New-Item -ItemType Directory -Force '{GUEST_DIR}' | Out-Null"), "", Duration::from_secs(15))
        .context("Could not prepare the Windows operator directory")?;
    let target = format!(r"{GUEST_DIR}\{name}");
    write_file(domain, &target, source.as_bytes())?;
    Ok(target)
}

pub fn ping(domain: &str) -> bool {
    virsh::agent(domain, &json!({"execute": "guest-ping"}), 2).is_ok()
}

/// `DOMAIN\user` of the signed-in console user, or empty at the sign-in screen.
pub fn signed_in_user(domain: &str) -> Result<String> {
    let user = powershell_ok(domain, "(Get-CimInstance Win32_ComputerSystem).UserName", "", Duration::from_secs(60))
        .context("The Windows guest agent did not answer")?;
    Ok(user.trim().to_owned())
}
