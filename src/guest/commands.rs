use crate::config::state_dir;
use crate::guest::agent::{install_script, powershell, powershell_ok, write_file};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const OUTPUT_LIMIT: usize = 64 * 1024;
const RESOLUTION_SCRIPT: &str = include_str!("../../assets/guest/set-display-resolution.ps1");
const UPDATE_SCRIPT: &str = include_str!("../../assets/guest/windows-update.ps1");

/// Keeps the last 64 KiB so huge outputs cannot flood callers.
fn bounded(text: &str) -> (&str, bool) {
    if text.len() <= OUTPUT_LIMIT {
        return (text, false);
    }
    let mut start = text.len() - OUTPUT_LIMIT;
    while !text.is_char_boundary(start) {
        start += 1;
    }
    (&text[start..], true)
}

pub fn exec(domain: &str, command: Option<&str>, file: Option<&Path>, timeout: u64, as_json: bool) -> Result<i32> {
    let script = match (file, command) {
        (Some(path), _) => fs::read_to_string(path).with_context(|| format!("Could not read {}", path.display()))?,
        (None, Some(command)) if !command.is_empty() => command.to_owned(),
        _ => bail!("Give a PowerShell command or --file"),
    };
    let run = powershell(domain, &script, "", Duration::from_secs(timeout))?;
    let (stdout, out_cut) = bounded(&run.stdout);
    let (stderr, err_cut) = bounded(&run.stderr);
    if as_json {
        println!("{}", json!({"exitCode": run.code, "stdout": stdout, "stderr": stderr, "truncated": out_cut || err_cut}));
    } else {
        print!("{stdout}");
        eprint!("{stderr}");
    }
    Ok(i32::try_from(run.code).unwrap_or(1))
}

pub fn copy(domain: &str, source: &Path, target: &str) -> Result<()> {
    let data = fs::read(source).with_context(|| format!("Could not read {}", source.display()))?;
    write_file(domain, target, &data)?;
    println!("{}", json!({"target": target, "bytes": data.len()}));
    Ok(())
}

pub fn resolution(domain: &str, width: u32, height: u32, as_json: bool) -> Result<()> {
    if !(640..=7680).contains(&width) || !(480..=4320).contains(&height) {
        bail!("Resolution must be within 640-7680 pixels wide and 480-4320 pixels high");
    }
    let target = install_script(domain, "set-display-resolution.ps1", RESOLUTION_SCRIPT)?;
    let script = format!(
        "$ErrorActionPreference='Stop'\ntry {{ & '{target}' -Width {width} -Height {height} }}\n\
         catch {{ [Console]::Error.WriteLine($_.Exception.Message); exit 1 }}"
    );
    let stdout = powershell_ok(domain, &script, "", Duration::from_secs(90))?;
    let result: Value = serde_json::from_str(stdout.trim()).context("Windows sent an invalid resolution report")?;
    if as_json {
        println!("{result}");
    } else {
        println!("Windows display is {}×{}; current and saved settings verified", result["width"], result["height"]);
    }
    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum UpdateAction {
    Status,
    Check,
    Install,
}

pub fn updates_file() -> std::path::PathBuf {
    state_dir().join("updates.json")
}

pub fn updates(domain: &str, action: UpdateAction) -> Result<i32> {
    if action == UpdateAction::Status {
        let saved = fs::read_to_string(updates_file()).unwrap_or_else(|_| json!({"checkedAt": 0, "pending": []}).to_string());
        println!("{}", saved.trim());
        return Ok(0);
    }
    let (mode, hours) = if action == UpdateAction::Install { ("install", 3) } else { ("check", 0) };
    let timeout = Duration::from_secs(if hours > 0 { hours * 3600 } else { 1800 });
    let stdout = powershell_ok(domain, UPDATE_SCRIPT, &format!("-Mode {mode}"), timeout)?;
    let last = stdout.trim().lines().last().unwrap_or_default();
    let mut data: Value = serde_json::from_str(last).context("Windows Update sent an invalid report")?;
    for key in ["pending", "installed", "failed"] {
        // ConvertTo-Json turns one-item arrays into strings.
        data[key] = match data[key].take() {
            Value::String(title) => json!([title]),
            Value::Array(titles) => Value::Array(titles),
            _ => json!([]),
        };
    }
    let now = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |elapsed| elapsed.as_secs());
    data["checkedAt"] = json!(now);
    write_private(&updates_file(), &data.to_string())?;
    println!("{data}");
    Ok(if data["failed"].as_array().is_some_and(|failed| !failed.is_empty()) { 1 } else { 0 })
}

/// Atomic write into the private state directory.
pub fn write_private(path: &Path, contents: &str) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;
    if let Some(parent) = path.parent() {
        fs::DirBuilder::new().recursive(true).mode(0o700).create(parent)?;
    }
    let temporary = path.with_extension("tmp");
    fs::File::create(&temporary)?.write_all(contents.as_bytes())?;
    fs::rename(&temporary, path).with_context(|| format!("Could not save {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_output_keeps_the_tail_on_a_char_boundary() {
        let text = format!("é{}", "a".repeat(OUTPUT_LIMIT));
        let (kept, cut) = bounded(&text);
        assert!(cut);
        assert_eq!(kept.len(), OUTPUT_LIMIT);
        assert_eq!(bounded("short"), ("short", false));
    }
}
