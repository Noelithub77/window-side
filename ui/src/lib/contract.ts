// The Rust ↔ UI contract. Temporary: replaced by tauri-specta's generated
// `src/bindings.ts` once the Rust commands exist; names and shapes must match.

export type Appearance = "pastel" | "omarchy"

/** Omarchy theme colours as hex, read from the active Omarchy theme. */
export type Palette = {
  background: string
  foreground: string
  accent: string
  colors: string[]
}

export type AgentId = "claude" | "codex" | "opencode"
export type Agent = { id: AgentId; name: string }

/** Facts that rarely change while the window is open. */
export type Shell = {
  version: string
  fixture: boolean
  omarchy: boolean
  appearance: Appearance
  palette: Palette | null
  agents: Agent[]
}

export type WindowsInstall = {
  partition: string
  diskModel: string
  size: number
  diskSize: number
  /** Windows owns the whole drive, so the VM can use it directly. */
  dedicatedDrive: boolean
}

export type GpuBlocker = "host-display" | "iommu-off" | "shared-group"

export type GpuOption = {
  address: string
  name: string
  vendor: "nvidia" | "amd" | "intel" | "other"
  blocker: GpuBlocker | null
  usable: boolean
}

export type Host = {
  threads: number
  performanceThreads: number
  memoryMib: number
  memoryLimitMib: number
  vcpuLimit: number
  iommu: boolean
  missingPackages: string[]
  bootLoader: "limine" | "grub" | "systemd-boot" | null
  distro: "arch" | "debian" | "fedora" | null
  desktop: "hyprland" | "kde" | "gnome" | "other"
  omarchy: boolean
}

export type Choices = {
  windows: string
  gpu: string | null
  memoryMib: number
  vcpus: number
  plugin: boolean
}

export type Detection = {
  windows: WindowsInstall[]
  gpus: GpuOption[]
  host: Host
  /** Defaults, or the current setup when there is one. */
  choices: Choices | null
  configured: boolean
}

export type PlanFile = { path: string; contents: string }

export type Plan = {
  facts: { label: string; value: string }[]
  notes: string[]
  packages: string[]
  groups: string[]
  files: PlanFile[]
  domainXml: string
}

export type VmState = "running" | "stopped" | "paused" | "stopping" | "unavailable"
export type BootProfile = "normal" | "gpu"

export type FirstBootStage = "start" | "drivers" | "signIn" | "ready" | "gpuBoot" | "complete"
export type FirstBoot = {
  stage: FirstBootStage
  storageReady: boolean | null
  networkReady: boolean | null
}

export type VmSummary = {
  name: string
  memoryMib: number
  vcpus: number
  gpu: string | null
  diskMode: "whole" | "partition"
}

export type Status = {
  configured: boolean
  vm: VmSummary | null
  state: VmState
  bootProfile: BootProfile | null
  storageError: string | null
  pendingUpdates: number | null
  uptimeSeconds: number | null
  processes: number | null
  credentialSaved: boolean | null
  /** Present until the first boot has finished. */
  firstBoot: FirstBoot | null
}

/** One guest sample; network values are bytes per second. */
export type Sample = {
  at: number
  cpu: number | null
  memory: number | null
  memoryUsed: string | null
  gpu: number | null
  down: number | null
  up: number | null
}

/** Rolling history kept in Rust, newest last. */
export type Metrics = { available: boolean; samples: Sample[] }

export type Action =
  | { kind: "launch" }
  | { kind: "console" }
  | { kind: "stop" }
  | { kind: "forceStop" }
  | { kind: "reboot"; profile: BootProfile }
  | { kind: "resolution"; width: number; height: number }
  | { kind: "lookingGlassSetup" }
  | { kind: "updatesCheck" }
  | { kind: "updatesInstall" }
  | { kind: "signIn" }
  | { kind: "installPlugin" }
  | { kind: "firstBootStart" }
  | { kind: "firstBootFinish" }

/** Commands, all async and rejecting with a short message string. */
export type Commands = {
  shell: () => Promise<Shell>
  detect: () => Promise<Detection>
  plan: (choices: Choices) => Promise<Plan>
  /** Re-plans from `choices` in Rust, then applies through pkexec. */
  apply: (choices: Choices) => Promise<void>
  status: () => Promise<Status>
  metrics: () => Promise<Metrics>
  run: (action: Action) => Promise<void>
  saveCredential: (secret: string) => Promise<void>
  forgetCredential: () => Promise<void>
  setAppearance: (appearance: Appearance) => Promise<Shell>
  agentPrompt: (agent: AgentId) => Promise<string>
}

/** Events pushed from Rust: `status` on change, `metrics` on every sample. */
export type Events = {
  status: Status
  metrics: Metrics
}
