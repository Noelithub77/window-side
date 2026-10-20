use super::types::*;
use crate::{config::Config, host, vm::virsh};
use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    #[serde(default)] pub available: bool,
    pub cpu: Option<f64>, pub memory: Option<String>, pub memory_percent: Option<f64>, pub gpu_percent: Option<f64>,
    pub network_received_bytes: Option<f64>, pub network_sent_bytes: Option<f64>, pub sampled_at: Option<f64>,
    pub uptime_seconds: Option<f64>, pub pids: Option<f64>, pub updates: Option<Updates>, pub storage: Option<Storage>,
}
#[derive(Deserialize)]
pub struct Updates { pending: Option<u32> }
#[derive(Deserialize)]
pub struct Storage { error: String }

pub fn read() -> Result<Reading> {
    let output = std::process::Command::new(std::env::current_exe()?).arg("metrics").env("WS_METRICS_CACHE_SECONDS", "2").output()?;
    anyhow::ensure!(output.status.success(), "Could not read Windows usage");
    serde_json::from_slice(&output.stdout).context("Windows returned invalid usage")
}
pub fn status(reading: Option<&Reading>, credential: Option<bool>) -> Result<Status> {
    let path = crate::config::config_dir().join("config.json");
    let config = if path.exists() || crate::config::config_dir().join("config.toml").exists() { Some(Config::load()?) } else { None };
    let Some(c) = config else { return Ok(empty()) };
    let state = match virsh::state(&c.vm.name).as_str() { "running" => VmState::Running, "shut off" => VmState::Stopped, "paused" => VmState::Paused, "in shutdown" => VmState::Stopping, _ => VmState::Unavailable };
    let first_boot = if c.needs_first_boot() {
        let boot = crate::first_boot::status(&c)?;
        Some(FirstBoot { stage: match boot.stage { crate::first_boot::Stage::Start => FirstBootStage::Start, crate::first_boot::Stage::Drivers => FirstBootStage::Drivers, crate::first_boot::Stage::SignIn => FirstBootStage::SignIn, crate::first_boot::Stage::Ready => FirstBootStage::Ready, crate::first_boot::Stage::GpuBoot => FirstBootStage::GpuBoot, crate::first_boot::Stage::Complete => FirstBootStage::Complete }, storage_ready: boot.storage_ready, network_ready: boot.network_ready })
    } else { None };
    let live = reading.filter(|r| r.available && state == VmState::Running);
    Ok(Status { configured: true, vm: Some(VmSummary { name: c.vm.name.clone(), memory_mib: c.vm.memory_mib, vcpus: c.vm.vcpus, gpu: c.gpu.as_ref().map(|g| g.address.clone()), disk_mode: match c.disk.mode { crate::config::DiskMode::Whole => DiskMode::Whole, crate::config::DiskMode::Partition => DiskMode::Partition } }), state,
        boot_profile: match host::boot_profile(c.gpu.as_ref()) { host::Profile::Normal => Some(BootProfile::Normal), host::Profile::Gpu => Some(BootProfile::Gpu), host::Profile::Unknown => None },
        storage_error: reading.and_then(|r| r.storage.as_ref()).map(|s| s.error.clone()).filter(|e| !e.is_empty()).or_else(|| (!c.disk_link().exists()).then(|| "Windows disk not detected".into())),
        pending_updates: live.and_then(|r| r.updates.as_ref()).and_then(|u| u.pending), uptime_seconds: live.and_then(|r| r.uptime_seconds), processes: live.and_then(|r| r.pids), credential_saved: credential, first_boot,
    })
}
pub fn empty() -> Status {
    Status { configured: false, vm: None, state: VmState::Unavailable, boot_profile: None, storage_error: None, pending_updates: None, uptime_seconds: None, processes: None, credential_saved: None, first_boot: None }
}
#[derive(Default)]
pub struct History { pub metrics: Metrics, last: Option<(f64, Option<f64>, Option<f64>)> }
impl History {
    pub fn record(&mut self, r: &Reading) -> bool {
        if !r.available { self.metrics = Metrics::default(); self.last = None; return false; }
        let Some(at) = r.sampled_at else { return false };
        if self.last.is_some_and(|(last, ..)| last == at) { return false; }
        let rate = |current: Option<f64>, old: Option<f64>, elapsed: f64| current.zip(old).filter(|(a,b)| elapsed > 0. && a >= b).map(|(a,b)| (a-b)/elapsed);
        let (down, up) = self.last.map_or((None, None), |(t,d,u)| (rate(r.network_received_bytes,d,at-t), rate(r.network_sent_bytes,u,at-t)));
        self.metrics.available = true;
        self.metrics.samples.push(Sample { at, cpu: r.cpu, memory: r.memory_percent, memory_used: r.memory.clone(), gpu: r.gpu_percent, down, up });
        if self.metrics.samples.len() > 48 { self.metrics.samples.remove(0); }
        self.last = Some((at,r.network_received_bytes,r.network_sent_bytes));
        true
    }
    pub fn clear(&mut self) { *self = Self::default(); }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample(at: f64, down: Option<f64>) -> Reading {
        Reading { available: true, sampled_at: Some(at), cpu: Some(12.), network_received_bytes: down, network_sent_bytes: Some(at * 10.), ..Default::default() }
    }
    #[test]
    fn history_keeps_48_samples_and_derives_rates_without_fabricating_unknowns() {
        let mut h = History::default();
        assert!(h.record(&sample(1., Some(100.))));
        assert!(!h.record(&sample(1., Some(100.))));
        assert_eq!(h.metrics.samples[0].down, None);
        h.record(&sample(3., Some(200.)));
        assert_eq!(h.metrics.samples[1].down, Some(50.));
        h.record(&sample(5., None));
        assert_eq!(h.metrics.samples[2].down, None);
        h.record(&sample(7., Some(10.)));
        assert_eq!(h.metrics.samples[3].down, None);
        for at in 8..100 { h.record(&sample(f64::from(at), Some(f64::from(at)))); }
        assert_eq!(h.metrics.samples.len(), 48);
        assert_eq!(h.metrics.samples[47].at, 99.);
        h.record(&Reading::default());
        assert!(!h.metrics.available);
        assert!(h.metrics.samples.is_empty());
    }
}
