//! Synthetic scenarios; every command stays in memory and cannot reach host operations.
use super::{readings, types::*};

pub struct Fixture { pub status: Status, pub appearance: Appearance, step: u32, started: Option<std::time::Instant> }
impl Fixture {
    pub fn new(scenario: &str) -> anyhow::Result<Self> {
        anyhow::ensure!(["setup", "status", "stopped", "first-boot-start", "first-boot-ready"].contains(&scenario), "Unknown GUI fixture: {scenario}");
        let configured = scenario != "setup";
        let running = matches!(scenario, "status" | "first-boot-ready");
        let mut status = readings::empty();
        status.configured = configured;
        status.vm = configured.then(vm);
        status.state = if running { VmState::Running } else if configured { VmState::Stopped } else { VmState::Unavailable };
        status.boot_profile = Some(if scenario == "stopped" { BootProfile::Normal } else { BootProfile::Gpu });
        status.credential_saved = match scenario { "status" => Some(false), "stopped" => Some(true), _ => None };
        if scenario.starts_with("first-boot-") { status.first_boot = Some(FirstBoot { stage: if running { FirstBootStage::Ready } else { FirstBootStage::Start }, storage_ready: running.then_some(true), network_ready: running.then_some(true) }); }
        if running { status.uptime_seconds = Some(if scenario == "status" { 3349. } else { 420. }); status.processes = Some(if scenario == "status" { 142. } else { 96. }); }
        if scenario == "status" { status.pending_updates = Some(2); }
        Ok(Self { status, appearance: Appearance::Pastel, step: 48, started: None })
    }
    pub fn shell(&self) -> Shell {
        Shell { version: env!("CARGO_PKG_VERSION").into(), fixture: true, omarchy: true, appearance: self.appearance,
            palette: Some(Palette { background: "#1a1b26".into(), foreground: "#c0caf5".into(), accent: "#7aa2f7".into(), colors: ["#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff"].map(str::to_owned).to_vec() }),
            agents: vec![Agent { id: AgentId::Claude, name: "Claude Code".into() }, Agent { id: AgentId::Codex, name: "Codex".into() }] }
    }
    pub fn detection(&self) -> Detection {
        Detection { windows: vec![WindowsInstall { partition: "/dev/nvme0n1p3".into(), disk_model: "Samsung 990 Pro".into(), size: 950. * 2_f64.powi(30), disk_size: 1024. * 2_f64.powi(30), dedicated_drive: true }, WindowsInstall { partition: "/dev/sda2".into(), disk_model: "Crucial MX500".into(), size: 420. * 2_f64.powi(30), disk_size: 500. * 2_f64.powi(30), dedicated_drive: false }],
            gpus: vec![GpuOption { address: "0000:01:00.0".into(), name: "NVIDIA GeForce RTX 4060 Laptop GPU".into(), vendor: Vendor::Nvidia, blocker: None, usable: true }, GpuOption { address: "0000:00:02.0".into(), name: "Intel Iris Xe Graphics".into(), vendor: Vendor::Intel, blocker: Some(GpuBlocker::HostDisplay), usable: false }],
            host: Host { threads: 20, performance_threads: 12, memory_mib: 32768., memory_limit_mib: 28672, vcpu_limit: 18, iommu: true, missing_packages: vec![], boot_loader: Some(BootLoader::Limine), distro: Some(Distro::Arch), desktop: Desktop::Hyprland, omarchy: true },
            choices: Some(Choices { windows: "/dev/nvme0n1p3".into(), gpu: Some("0000:01:00.0".into()), memory_mib: 16384, vcpus: 10, plugin: true }), configured: self.status.configured }
    }
    pub fn plan(&self, choices: Choices) -> anyhow::Result<Plan> {
        let d = self.detection();
        anyhow::ensure!(d.windows.iter().any(|w| w.partition == choices.windows), "Choose a detected Windows install");
        anyhow::ensure!((2048..=28672).contains(&choices.memory_mib) && (2..=18).contains(&choices.vcpus), "Choose valid memory and processor limits");
        anyhow::ensure!(choices.gpu.as_ref().is_none_or(|address| d.gpus.iter().any(|g| g.usable && &g.address == address)), "That GPU cannot be passed through");
        Ok(Plan { facts: vec![Fact { label: "Windows".into(), value: "Samsung 990 Pro · whole drive".into() }, Fact { label: "Memory".into(), value: format!("{} GiB", choices.memory_mib / 1024) }, Fact { label: "Processor".into(), value: format!("{} threads", choices.vcpus) }], notes: vec!["Linux keeps the Intel GPU for its own screen.".into(), "A GPU boot entry is added to Limine.".into()], packages: vec![], groups: vec!["libvirt".into(), "kvm".into()], files: vec![PlanFile { path: "/etc/modprobe.d/window-side-vfio.conf".into(), contents: "options vfio-pci ids=10de:28e0,10de:22be\n".into() }], domain_xml: "<domain type=\"kvm\"><name>window-side</name></domain>\n".into() })
    }
    pub fn apply(&mut self, choices: Choices) -> anyhow::Result<()> {
        self.plan(choices.clone())?;
        self.status.configured = true;
        self.status.vm = Some(VmSummary { memory_mib: choices.memory_mib, vcpus: choices.vcpus, gpu: choices.gpu, ..vm() });
        self.status.state = VmState::Stopped;
        self.status.first_boot = Some(FirstBoot { stage: FirstBootStage::Start, storage_ready: None, network_ready: None });
        Ok(())
    }
    pub fn action(&mut self, action: Action) {
        match action {
            Action::Launch => { self.status.state = VmState::Running; self.status.uptime_seconds = Some(12.); self.status.processes = Some(88.); }
            Action::Stop | Action::ForceStop => { self.status.state = VmState::Stopped; self.status.uptime_seconds = None; self.status.processes = None; }
            Action::Reboot { profile } => self.status.boot_profile = Some(profile),
            Action::UpdatesInstall => self.status.pending_updates = None,
            Action::FirstBootStart => { self.started = Some(std::time::Instant::now()); self.status.state = VmState::Running; self.status.first_boot = Some(FirstBoot { stage: FirstBootStage::Drivers, storage_ready: Some(false), network_ready: Some(false) }); }
            Action::FirstBootFinish => { self.started = None; self.status.first_boot = None; },
            _ => {}
        }
    }
    pub fn metrics(&mut self) -> Metrics {
        if self.status.state != VmState::Running { return Metrics::default(); }
        if let Some(started) = self.started {
            let elapsed = started.elapsed().as_secs_f64();
            self.status.first_boot = Some(FirstBoot {
                stage: if elapsed >= 10. { FirstBootStage::Ready } else if elapsed >= 7.5 { FirstBootStage::SignIn } else { FirstBootStage::Drivers },
                storage_ready: Some(elapsed >= 2.5), network_ready: Some(elapsed >= 5.),
            });
        }
        let wave = |base: f64, swing: f64, speed: f64, phase: f64, step: u32| { let t = f64::from(step) * speed + phase; (base + swing * (t.sin() + 0.45 * (t * 2.3).sin())).max(0.) };
        let samples = (self.step.saturating_sub(48)..self.step).map(|i| Sample { at: f64::from(i), cpu: Some(wave(16.,8.,0.35,0.,i).min(100.)), memory: Some(wave(46.,3.,0.18,1.,i)), memory_used: Some(format!("{:.1} GiB / 16.0 GiB", 16. * wave(46.,3.,0.18,1.,i)/100.)), gpu: Some(wave(32.,15.,0.28,2.,i).min(100.)), down: Some(wave(900000.,600000.,0.4,0.5,i)), up: Some(wave(220000.,150000.,0.33,2.5,i)) }).collect();
        self.step += 1;
        Metrics { available: true, samples }
    }
}
fn vm() -> VmSummary { VmSummary { name: "window-side".into(), memory_mib: 16384, vcpus: 10, gpu: Some("0000:01:00.0".into()), disk_mode: DiskMode::Whole } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scenarios_and_actions_are_isolated_in_memory() -> anyhow::Result<()> {
        for mode in ["setup", "status", "stopped", "first-boot-start", "first-boot-ready"] {
            let mut f = Fixture::new(mode)?;
            assert!(f.shell().fixture);
            assert_eq!(f.status.configured, mode != "setup");
            assert_eq!(f.metrics().available, matches!(mode, "status" | "first-boot-ready"));
        }
        let mut f = Fixture::new("setup")?;
        let choices = f.detection().choices.ok_or_else(|| anyhow::anyhow!("Missing choices"))?;
        f.apply(choices)?;
        assert!(f.status.configured);
        f.action(Action::FirstBootStart);
        assert_eq!(serde_json::to_value(&f.status.first_boot)?["stage"], "drivers");
        f.action(Action::FirstBootFinish);
        assert!(f.status.first_boot.is_none());
        f.action(Action::ForceStop);
        assert!(matches!(f.status.state, VmState::Stopped));
        assert!(Fixture::new("unknown").is_err());
        Ok(())
    }
}
