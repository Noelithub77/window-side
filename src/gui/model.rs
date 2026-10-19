use crate::{config::Config, setup};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::process::Command;

#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Metrics {
    #[serde(default)]
    status: String,
    #[serde(default)]
    available: bool,
    cpu: Option<f64>,
    memory: Option<String>,
    network: Option<String>,
    uptime_seconds: Option<f64>,
    boot_profile: Option<String>,
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

    pub(super) fn network(&self) -> &str {
        if self.available {
            self.network.as_deref().unwrap_or("—")
        } else {
            "—"
        }
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
            network: Some("↓ 461.3 MiB   ↑ 63.6 MiB".into()),
            uptime_seconds: Some(3349.),
            boot_profile: Some("gpu".into()),
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
