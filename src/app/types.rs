use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Appearance {
    #[default]
    Pastel,
    Omarchy,
}
#[derive(Clone, Serialize, Deserialize, Type)]
pub struct Palette {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub colors: Vec<String>,
}
#[derive(Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum AgentId {
    Claude,
    Codex,
    Opencode,
}
#[derive(Clone, Serialize, Type)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Shell {
    pub panel_installed: bool,
    pub version: String,
    pub fixture: bool,
    pub omarchy: bool,
    pub appearance: Appearance,
    pub palette: Option<Palette>,
    pub agents: Vec<Agent>,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WindowsInstall {
    pub partition: String,
    pub disk_model: String,
    #[specta(type = specta_typescript::Number)]
    pub size: f64,
    #[specta(type = specta_typescript::Number)]
    pub disk_size: f64,
    pub dedicated_drive: bool,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum GpuBlocker {
    HostDisplay,
    IommuOff,
    SharedGroup,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Vendor {
    Nvidia,
    Amd,
    Intel,
    Other,
}
#[derive(Clone, Serialize, Type)]
pub struct GpuOption {
    pub address: String,
    pub name: String,
    pub vendor: Vendor,
    pub blocker: Option<GpuBlocker>,
    pub usable: bool,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "kebab-case")]
pub enum BootLoader {
    Limine,
    Grub,
    SystemdBoot,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Distro {
    Arch,
    Debian,
    Fedora,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum Desktop {
    Hyprland,
    Kde,
    Gnome,
    Other,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Host {
    pub threads: u32,
    pub performance_threads: u32,
    #[specta(type = specta_typescript::Number)]
    pub memory_mib: f64,
    pub memory_limit_mib: u32,
    pub vcpu_limit: u32,
    pub iommu: bool,
    pub missing_packages: Vec<String>,
    pub boot_loader: Option<BootLoader>,
    pub distro: Option<Distro>,
    pub desktop: Desktop,
    pub omarchy: bool,
}
#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Choices {
    pub windows: String,
    pub gpu: Option<String>,
    pub memory_mib: u32,
    pub vcpus: u32,
    pub plugin: bool,
}
impl From<Choices> for crate::setup::Choices {
    fn from(c: Choices) -> Self {
        Self {
            windows: c.windows,
            gpu: c.gpu,
            memory_mib: c.memory_mib,
            vcpus: c.vcpus,
            plugin: c.plugin,
        }
    }
}
impl From<crate::setup::Choices> for Choices {
    fn from(c: crate::setup::Choices) -> Self {
        Self {
            windows: c.windows,
            gpu: c.gpu,
            memory_mib: c.memory_mib,
            vcpus: c.vcpus,
            plugin: c.plugin,
        }
    }
}
#[derive(Clone, Serialize, Type)]
pub struct Detection {
    pub windows: Vec<WindowsInstall>,
    pub gpus: Vec<GpuOption>,
    pub host: Host,
    pub choices: Option<Choices>,
    pub configured: bool,
}
#[derive(Clone, Serialize, Type)]
pub struct PlanFile {
    pub path: String,
    pub contents: String,
}
#[derive(Clone, Serialize, Type)]
pub struct Fact {
    pub label: String,
    pub value: String,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    pub facts: Vec<Fact>,
    pub notes: Vec<String>,
    pub packages: Vec<String>,
    pub groups: Vec<String>,
    pub files: Vec<PlanFile>,
    pub domain_xml: String,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum VmState {
    Running,
    Stopped,
    Paused,
    Stopping,
    Unavailable,
}
#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum BootProfile {
    Normal,
    Gpu,
}
#[derive(Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum FirstBootStage {
    Start,
    Drivers,
    SignIn,
    Ready,
    GpuBoot,
    Complete,
}
#[derive(Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FirstBoot {
    pub stage: FirstBootStage,
    pub storage_ready: Option<bool>,
    pub network_ready: Option<bool>,
}
#[derive(Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum DiskMode {
    Whole,
    Partition,
}
#[derive(Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct VmSummary {
    pub name: String,
    pub memory_mib: u32,
    pub vcpus: u32,
    pub gpu: Option<String>,
    pub disk_mode: DiskMode,
}
#[derive(Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub unavailable_reason: Option<String>,
    pub configured: bool,
    pub vm: Option<VmSummary>,
    pub state: VmState,
    pub boot_profile: Option<BootProfile>,
    pub storage_error: Option<String>,
    pub pending_updates: Option<u32>,
    pub uptime_seconds: Option<f64>,
    pub processes: Option<f64>,
    pub credential_saved: Option<bool>,
    pub first_boot: Option<FirstBoot>,
}
#[derive(Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Sample {
    #[specta(type = specta_typescript::Number)]
    pub at: f64,
    pub cpu: Option<f64>,
    pub memory: Option<f64>,
    pub memory_used: Option<String>,
    pub gpu: Option<f64>,
    pub down: Option<f64>,
    pub up: Option<f64>,
}
#[derive(Clone, Default, Serialize, Type)]
pub struct Metrics {
    pub available: bool,
    pub samples: Vec<Sample>,
}
#[derive(Clone, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Action {
    Launch,
    Console,
    Stop,
    ForceStop,
    Reboot { profile: BootProfile },
    Resolution { width: u32, height: u32 },
    LookingGlassSetup,
    UpdatesCheck,
    UpdatesInstall,
    SignIn,
    InstallPlugin,
    FirstBootStart,
    FirstBootFinish,
}
