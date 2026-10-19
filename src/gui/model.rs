use crate::{config::Config, setup};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::VecDeque;
use std::process::Command;

/// Samples kept per graph; the guest is sampled at most every ten seconds.
const HISTORY: usize = 40;

#[derive(Clone, Default, Deserialize)]
pub(super) struct Storage {
    #[serde(default)]
    error: String,
}

#[derive(Clone, Default, Deserialize)]
pub(super) struct Updates {
    pending: Option<usize>,
}

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Metrics {
    #[serde(default)]
    status: String,
    #[serde(default)]
    available: bool,
    cpu: Option<f64>,
    memory: Option<String>,
    memory_percent: Option<f64>,
    network_received_bytes: Option<f64>,
    network_sent_bytes: Option<f64>,
    pids: Option<u64>,
    gpu_percent: Option<f64>,
    sampled_at: Option<f64>,
    uptime_seconds: Option<f64>,
    boot_profile: Option<String>,
    storage: Option<Storage>,
    updates: Option<Updates>,
}

impl Metrics {
    pub(super) fn state(&self) -> &str {
        match self.status.as_str() {
            "running" => "Running",
            "shut off" => "Stopped",
            "paused" => "Paused",
            "in shutdown" => "Stopping",
            _ => "Unavailable",
        }
    }

    pub(super) fn running(&self) -> bool {
        self.status == "running"
    }

    pub(super) fn cpu(&self) -> String {
        self.cpu
            .filter(|_| self.available)
            .map_or_else(|| "—".into(), |cpu| format!("{cpu:.0}%"))
    }

    pub(super) fn memory(&self) -> &str {
        if self.available {
            self.memory.as_deref().unwrap_or("—")
        } else {
            "—"
        }
    }


    pub(super) fn memory_percent(&self) -> Option<f64> {
        self.memory_percent.filter(|_| self.available)
    }

    pub(super) fn gpu_percent(&self) -> Option<f64> {
        self.gpu_percent.filter(|_| self.available)
    }

    /// Why Windows cannot start, such as a missing disk.
    pub(super) fn storage_error(&self) -> Option<&str> {
        self.storage.as_ref().map(|storage| storage.error.as_str()).filter(|error| !error.is_empty())
    }

    pub(super) fn pending_updates(&self) -> Option<usize> {
        self.updates.as_ref().and_then(|updates| updates.pending).filter(|count| *count > 0)
    }

    pub(super) fn gpu_profile(&self) -> bool {
        self.boot_profile.as_deref() == Some("gpu")
    }



    pub(super) fn processes(&self) -> String {
        self.pids
            .filter(|_| self.available)
            .map_or_else(|| "—".into(), |pids| pids.to_string())
    }

    pub(super) fn uptime(&self) -> String {
        self.uptime_seconds.filter(|_| self.available).map_or_else(
            || "—".into(),
            |seconds| {
                let minutes = seconds.max(0.) as u64 / 60;
                if minutes >= 60 {
                    format!("{}h {}m", minutes / 60, minutes % 60)
                } else {
                    format!("{minutes}m")
                }
            },
        )
    }

    pub(super) fn profile(&self) -> &str {
        match self.boot_profile.as_deref() {
            Some("gpu") => "GPU passthrough",
            Some("normal") => "Normal",
            _ => "—",
        }
    }

    pub(super) fn read() -> Result<Self> {
        let output = Command::new(std::env::current_exe()?)
            .arg("metrics")
            .output()
            .context("Could not read Windows usage")?;
        anyhow::ensure!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        serde_json::from_slice(&output.stdout).context("Windows usage was invalid")
    }

    pub(super) fn fixture() -> Self {
        Self {
            status: "running".into(),
            available: true,
            cpu: Some(6.),
            memory: Some("7.4 GiB / 16.0 GiB".into()),
            memory_percent: Some(46.3),
            network_received_bytes: Some(461.3 * 1024. * 1024.),
            network_sent_bytes: Some(63.6 * 1024. * 1024.),
            pids: Some(142),
            gpu_percent: Some(31.),
            sampled_at: Some(1000.),
            storage: None,
            updates: Some(Updates { pending: Some(2) }),
            uptime_seconds: Some(3349.),
            boot_profile: Some("gpu".into()),
        }
    }
}

/// Rolling graph data. Network totals become per-second rates between samples.
#[derive(Default)]
pub(super) struct History {
    pub(super) cpu: VecDeque<f64>,
    pub(super) memory: VecDeque<f64>,
    pub(super) gpu: VecDeque<f64>,
    pub(super) down: VecDeque<f64>,
    pub(super) up: VecDeque<f64>,
    last: Option<(f64, f64, f64)>,
}

fn push(series: &mut VecDeque<f64>, value: f64) {
    if series.len() == HISTORY {
        series.pop_front();
    }
    series.push_back(value);
}

impl History {
    pub(super) fn record(&mut self, metrics: &Metrics) {
        let Some(at) = metrics.sampled_at.filter(|_| metrics.available) else {
            *self = Self::default();
            return;
        };
        if self.last.is_some_and(|(last, ..)| last == at) {
            return;
        }
        if let Some(cpu) = metrics.cpu {
            push(&mut self.cpu, cpu);
        }
        if let Some(memory) = metrics.memory_percent {
            push(&mut self.memory, memory);
        }
        if let Some(gpu) = metrics.gpu_percent {
            push(&mut self.gpu, gpu);
        }
        let (Some(received), Some(sent)) = (metrics.network_received_bytes, metrics.network_sent_bytes) else {
            return;
        };
        if let Some((last, last_received, last_sent)) = self.last {
            let seconds = at - last;
            if seconds > 0. {
                push(&mut self.down, ((received - last_received) / seconds).max(0.));
                push(&mut self.up, ((sent - last_sent) / seconds).max(0.));
            }
        }
        self.last = Some((at, received, sent));
    }

    /// Smooth synthetic traces so fixture graphs have shape.
    pub(super) fn fixture() -> Self {
        let wave = |base: f64, swing: f64, speed: f64, phase: f64| -> VecDeque<f64> {
            (0..HISTORY)
                .map(|step| {
                    let t = step as f64 * speed + phase;
                    (base + swing * (t.sin() + 0.45 * (t * 2.3).sin())).max(0.)
                })
                .collect()
        };
        Self {
            cpu: wave(14., 7., 0.35, 0.),
            memory: wave(46., 3., 0.18, 1.),
            gpu: wave(30., 14., 0.28, 2.),
            down: wave(900_000., 600_000., 0.4, 0.5),
            up: wave(220_000., 150_000., 0.33, 2.5),
            last: None,
        }
    }
}

pub(super) struct Detection {
    pub(super) inventory: setup::Inventory,
    pub(super) config: Option<Config>,
}

pub(super) fn detect() -> Result<Detection> {
    Ok(Detection {
        inventory: setup::detect()?,
        config: Config::load().ok(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_data_does_not_show_zero_or_old_numbers() {
        let metrics = Metrics {
            available: false,
            ..Metrics::fixture()
        };
        assert_eq!(
            (metrics.cpu(), metrics.memory(), metrics.uptime()),
            ("—".into(), "—", "—".into())
        );
        assert_eq!(Metrics::default().state(), "Unavailable");
    }
}
