//! Windows usage through the guest agent; never substitutes the VM's allocation.
use crate::config::Config;
use crate::guest::agent::powershell_ok;
use crate::guest::commands::{updates_file, write_private};
use crate::host::Profile;
use crate::process::hook;
use crate::vm::virsh;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::fs::{self, File};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SCRIPT: &str = include_str!("../../assets/guest/metrics.ps1");
const CACHE_SECONDS: f64 = 10.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Snapshot {
    cpu_percent: f64,
    memory_total_bytes: f64,
    memory_used_bytes: f64,
    uptime_seconds: f64,
    booted_at: String,
    network_received_bytes: f64,
    network_sent_bytes: f64,
    process_count: f64,
    reboot_required: Option<bool>,
    #[serde(default)]
    sampled_at: f64,
}

impl Snapshot {
    fn is_valid(&self) -> bool {
        let counts = [
            self.cpu_percent, self.memory_total_bytes, self.memory_used_bytes, self.uptime_seconds,
            self.network_received_bytes, self.network_sent_bytes, self.process_count,
        ];
        counts.iter().all(|value| value.is_finite() && *value >= 0.0)
            && self.cpu_percent <= 100.0
            && self.memory_total_bytes > 0.0
            && self.memory_used_bytes <= self.memory_total_bytes
            && !self.booted_at.is_empty()
    }
}

fn now() -> f64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0.0, |elapsed| elapsed.as_secs_f64())
}

fn runtime_dir() -> PathBuf {
    let base = std::env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("window-side-metrics")
}

pub(crate) fn byte_label(value: f64) -> String {
    let mut amount = value;
    for unit in ["B", "KiB", "MiB", "GiB"] {
        if amount < 1024.0 {
            return format!("{amount:.1} {unit}");
        }
        amount /= 1024.0;
    }
    format!("{amount:.1} TiB")
}

fn storage(config: &Config) -> Value {
    let disk_present = config.disk_link().exists();
    let ready = match hook(&config.hooks.storage_status) {
        None => Some(disk_present),
        Some(Ok(output)) => serde_json::from_slice::<Value>(&output.stdout).ok().and_then(|data| data["ready"].as_bool()),
        Some(Err(_)) => None,
    };
    match ready {
        Some(true) => json!({"status": "ready", "error": ""}),
        Some(false) if !disk_present => json!({"status": "missing", "error": "Windows disk not detected"}),
        Some(false) => json!({"status": "unavailable", "error": "Windows disk unavailable"}),
        None => json!({"status": "unknown", "error": "Could not check the Windows disk"}),
    }
}

/// Fresh guest numbers, reusing a sample younger than ten seconds.
fn sample(domain: &str) -> Option<Snapshot> {
    let cache = runtime_dir().join("guest.json");
    if let Some(cached) = fs::read_to_string(&cache).ok().and_then(|text| serde_json::from_str::<Snapshot>(&text).ok()) {
        let age = now() - cached.sampled_at;
        if cached.is_valid() && (0.0..CACHE_SECONDS).contains(&age) {
            return Some(cached);
        }
    }
    let fresh = powershell_ok(domain, SCRIPT, "", Duration::from_secs(8))
        .ok()
        .and_then(|stdout| serde_json::from_str::<Snapshot>(stdout.trim()).ok())
        .filter(Snapshot::is_valid)
        .map(|snapshot| Snapshot { sampled_at: now(), ..snapshot });
    match &fresh {
        Some(snapshot) => {
            let _ = serde_json::to_string(snapshot).map(|text| write_private(&cache, &text));
        }
        None => {
            let _ = fs::remove_file(&cache);
        }
    }
    fresh
}

fn updates_summary(guest: Option<&Snapshot>, now: f64) -> Value {
    let saved: Value = fs::read_to_string(updates_file()).ok().and_then(|text| serde_json::from_str(&text).ok()).unwrap_or(Value::Null);
    let checked = saved["checkedAt"].as_f64().unwrap_or(0.0);
    let titles = saved["pending"].as_array().filter(|titles| titles.iter().all(Value::is_string));
    let fresh = (0.0..12.0 * 3600.0).contains(&(now - checked))
        && titles.is_some()
        && guest.is_some_and(|guest| saved["bootedAt"].as_str() == Some(guest.booted_at.as_str()));
    json!({
        "pending": if fresh { titles.map(Vec::len) } else { None },
        "titles": if fresh { titles.cloned().unwrap_or_default() } else { Vec::new() },
        "rebootRequired": guest.and_then(|guest| guest.reboot_required),
        "checkedAt": checked,
        "fresh": fresh,
    })
}

fn report(state: &str, profile: Profile, guest: Option<&Snapshot>, now: f64) -> Value {
    let guest = guest.filter(|_| state == "running");
    let mut result = json!({
        "status": state, "bootProfile": profile, "backend": "libvirt", "available": guest.is_some(),
        "source": guest.map(|_| "windows-guest-agent"), "cpu": null, "memory": "—", "memoryPercent": null,
        "network": "—", "pids": null, "uptimeSeconds": null, "sampledAt": null,
        "updates": updates_summary(guest, now),
    });
    if let Some(guest) = guest {
        let age = (now - guest.sampled_at).max(0.0);
        if let (Value::Object(fields), Ok(Value::Object(sample))) = (&mut result, serde_json::to_value(guest)) {
            fields.extend(sample);
        }
        result["cpu"] = json!(guest.cpu_percent);
        result["memory"] = json!(format!("{} / {}", byte_label(guest.memory_used_bytes), byte_label(guest.memory_total_bytes)));
        result["memoryPercent"] = json!(100.0 * guest.memory_used_bytes / guest.memory_total_bytes);
        result["network"] = json!(format!("↓ {}\n↑ {}", byte_label(guest.network_received_bytes), byte_label(guest.network_sent_bytes)));
        result["pids"] = json!(guest.process_count as u64);
        result["uptimeSeconds"] = json!((guest.uptime_seconds + age) as u64);
    }
    result
}

/// Prints the JSON the Omarchy panel polls. Exits 1 only when libvirt cannot be asked.
pub fn show(config: &Config, profile: Profile) -> Result<i32> {
    let runtime = runtime_dir();
    write_private(&runtime.join("lock"), "")?;
    let lock = File::options().write(true).open(runtime.join("lock")).context("Could not open the metrics lock")?;
    lock.lock().context("Could not lock the metrics cache")?;
    let domain = config.vm.name.as_str();
    let state = match virsh::state(domain) {
        state if state.is_empty() => "unknown".to_owned(),
        state => state,
    };
    let guest = if state == "running" {
        sample(domain)
    } else {
        let _ = fs::remove_file(runtime.join("guest.json"));
        None
    };
    let mut result = report(&state, profile, guest.as_ref(), now());
    result["storage"] = storage(config);
    println!("{result}");
    Ok(if state == "unknown" { 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> Snapshot {
        Snapshot {
            cpu_percent: 12.5, memory_total_bytes: 8.0 * 1024.0 * 1024.0 * 1024.0, memory_used_bytes: 2.0 * 1024.0 * 1024.0 * 1024.0,
            uptime_seconds: 60.0, booted_at: "2026-10-19T00:00:00Z".into(), network_received_bytes: 2048.0,
            network_sent_bytes: 1024.0, process_count: 120.0, reboot_required: Some(false), sampled_at: 1000.0,
        }
    }

    #[test]
    fn rejects_impossible_samples() {
        assert!(snapshot().is_valid());
        assert!(!Snapshot { cpu_percent: 140.0, ..snapshot() }.is_valid());
        assert!(!Snapshot { memory_used_bytes: 9e12, ..snapshot() }.is_valid());
        assert!(!Snapshot { booted_at: String::new(), ..snapshot() }.is_valid());
    }

    #[test]
    fn running_report_carries_panel_fields_and_ages_uptime() {
        let result = report("running", Profile::Gpu, Some(&snapshot()), 1030.0);
        assert_eq!(result["available"], true);
        assert_eq!(result["bootProfile"], "gpu");
        assert_eq!(result["memory"], "2.0 GiB / 8.0 GiB");
        assert_eq!(result["memoryPercent"], 25.0);
        assert_eq!(result["uptimeSeconds"], 90);
        assert_eq!(result["pids"], 120);
    }

    #[test]
    fn stopped_report_hides_stale_guest_numbers() {
        let result = report("shut off", Profile::Normal, Some(&snapshot()), 1030.0);
        assert_eq!(result["available"], false);
        assert_eq!(result["memory"], "—");
        assert!(result["cpu"].is_null());
    }
}
