// Synthetic hardware for the browser preview and UI tests. Pick a scenario
// with `?fixture=setup|status|stopped|first-boot-start|first-boot-ready`.
import type {
  Action,
  Commands,
  Detection,
  Events,
  Metrics,
  Plan,
  Sample,
  Shell,
  Status,
} from "@/bindings"

const scenario = new URLSearchParams(window.location.search).get("fixture") ?? "setup"
const HISTORY = 48

const shell: Shell = {
  version: "0.4.0",
  fixture: true,
  omarchy: true,
  appearance: "pastel",
  palette: {
    background: "#1a1b26",
    foreground: "#c0caf5",
    accent: "#7aa2f7",
    colors: ["#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff"],
  },
  agents: [
    { id: "claude", name: "Claude Code" },
    { id: "codex", name: "Codex" },
  ],
}

const detection: Detection = {
  windows: [
    { partition: "/dev/nvme0n1p3", diskModel: "Samsung 990 Pro", size: 950 * 2 ** 30, diskSize: 1024 * 2 ** 30, dedicatedDrive: true },
    { partition: "/dev/sda2", diskModel: "Crucial MX500", size: 420 * 2 ** 30, diskSize: 500 * 2 ** 30, dedicatedDrive: false },
  ],
  gpus: [
    { address: "0000:01:00.0", name: "NVIDIA GeForce RTX 4060 Laptop GPU", vendor: "nvidia", blocker: null, usable: true },
    { address: "0000:00:02.0", name: "Intel Iris Xe Graphics", vendor: "intel", blocker: "host-display", usable: false },
  ],
  host: {
    threads: 20,
    performanceThreads: 12,
    memoryMib: 32768,
    memoryLimitMib: 28672,
    vcpuLimit: 18,
    iommu: true,
    missingPackages: [],
    bootLoader: "limine",
    distro: "arch",
    desktop: "hyprland",
    omarchy: true,
  },
  choices: { windows: "/dev/nvme0n1p3", gpu: "0000:01:00.0", memoryMib: 16384, vcpus: 10, plugin: true },
  configured: false,
}

const vm = { name: "window-side", memoryMib: 16384, vcpus: 10, gpu: "0000:01:00.0", diskMode: "whole" as const }

const setupStatus: Status = { configured: false, vm: null, state: "unavailable", bootProfile: "gpu", storageError: null, pendingUpdates: null, uptimeSeconds: null, processes: null, credentialSaved: null, firstBoot: null }

const statuses: Record<string, Status> = {
  setup: setupStatus,
  status: { configured: true, vm, state: "running", bootProfile: "gpu", storageError: null, pendingUpdates: 2, uptimeSeconds: 3349, processes: 142, credentialSaved: false, firstBoot: null },
  stopped: { configured: true, vm, state: "stopped", bootProfile: "normal", storageError: null, pendingUpdates: null, uptimeSeconds: null, processes: null, credentialSaved: true, firstBoot: null },
  "first-boot-start": { configured: true, vm, state: "stopped", bootProfile: "gpu", storageError: null, pendingUpdates: null, uptimeSeconds: null, processes: null, credentialSaved: null, firstBoot: { stage: "start", storageReady: null, networkReady: null } },
  "first-boot-ready": { configured: true, vm, state: "running", bootProfile: "gpu", storageError: null, pendingUpdates: null, uptimeSeconds: 420, processes: 96, credentialSaved: null, firstBoot: { stage: "ready", storageReady: true, networkReady: true } },
}

let status: Status = structuredClone(statuses[scenario] ?? setupStatus)

function wave(base: number, swing: number, speed: number, phase: number, step: number): number {
  const t = step * speed + phase
  return Math.max(0, base + swing * (Math.sin(t) + 0.45 * Math.sin(t * 2.3)))
}

function sample(step: number): Sample {
  return {
    at: step,
    cpu: Math.min(100, wave(16, 8, 0.35, 0, step)),
    memory: wave(46, 3, 0.18, 1, step),
    memoryUsed: `${(16 * wave(46, 3, 0.18, 1, step) / 100).toFixed(1)} GiB / 16.0 GiB`,
    gpu: Math.min(100, wave(32, 15, 0.28, 2, step)),
    down: wave(900_000, 600_000, 0.4, 0.5, step),
    up: wave(220_000, 150_000, 0.33, 2.5, step),
  }
}

let step = HISTORY
let samples: Sample[] = Array.from({ length: HISTORY }, (_, index) => sample(index))

// False while a freshly launched guest boots, before its agent answers.
let agentReady = true

function metrics(): Metrics {
  const available = status.state === "running" && agentReady
  return { available, samples: available ? samples : [] }
}

type Handlers = { [K in keyof Events]: Set<(payload: Events[K]) => void> }
const handlers: Handlers = { status: new Set(), metrics: new Set() }

function emitStatus() {
  for (const handler of handlers.status) handler(status)
}

function emitMetrics() {
  for (const handler of handlers.metrics) handler(metrics())
}

window.setInterval(() => {
  if (status.state !== "running") return
  samples = [...samples.slice(1 - HISTORY), sample(step)]
  step += 1
  emitMetrics()
}, 2000)

export function mockListen<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
  const set = handlers[event] as Set<(payload: Events[K]) => void>
  set.add(handler)
  return () => set.delete(handler)
}

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))

function update(next: Partial<Status>) {
  status = { ...status, ...next }
  emitStatus()
  emitMetrics()
}

const plan: Plan = {
  facts: [
    { label: "Windows", value: "Samsung 990 Pro · whole drive" },
    { label: "Graphics", value: "RTX 4060 in the GPU profile" },
    { label: "Memory", value: "16 GiB" },
    { label: "Processor", value: "10 threads on performance cores" },
  ],
  notes: ["Linux keeps the Intel GPU for its own screen.", "A GPU boot entry is added to Limine."],
  packages: [],
  groups: ["libvirt", "kvm"],
  files: [
    { path: "/etc/modprobe.d/window-side-vfio.conf", contents: "options vfio-pci ids=10de:28e0,10de:22be\nsoftdep nvidia pre: vfio-pci\n" },
    { path: "/etc/default/limine", contents: "# window-side GPU profile\nKERNEL_CMDLINE[gpu]+=\" intel_iommu=on vfio-pci.ids=10de:28e0,10de:22be\"\n" },
  ],
  domainXml: "<domain type=\"kvm\">\n  <name>window-side</name>\n  <memory unit=\"MiB\">16384</memory>\n  <vcpu placement=\"static\">10</vcpu>\n  <os firmware=\"efi\">\n    <type arch=\"x86_64\" machine=\"q35\">hvm</type>\n  </os>\n</domain>\n",
}

/** Walks the guest through drivers and sign-in so the preview shows each stage. */
function playFirstBoot() {
  const steps: Status["firstBoot"][] = [
    { stage: "drivers", storageReady: true, networkReady: false },
    { stage: "drivers", storageReady: true, networkReady: true },
    { stage: "signIn", storageReady: true, networkReady: true },
    { stage: "ready", storageReady: true, networkReady: true },
  ]
  steps.forEach((firstBoot, index) => {
    window.setTimeout(() => {
      if (status.firstBoot) update({ firstBoot })
    }, 2500 * (index + 1))
  })
}

async function run(action: Action): Promise<void> {
  await wait(900)
  switch (action.kind) {
    case "launch":
      agentReady = false
      update({ state: "running", uptimeSeconds: null, processes: null })
      window.setTimeout(() => {
        agentReady = true
        update({ uptimeSeconds: 12, processes: 88 })
      }, 4000)
      return
    case "stop":
    case "forceStop":
      update({ state: "stopped", uptimeSeconds: null, processes: null })
      return
    case "reboot":
      update({ bootProfile: action.profile })
      return
    case "updatesInstall":
      update({ pendingUpdates: null })
      return
    case "firstBootStart":
      update({ state: "running", firstBoot: { stage: "drivers", storageReady: false, networkReady: false } })
      playFirstBoot()
      return
    case "firstBootFinish":
      update({ firstBoot: null })
      return
    case "console":
    case "resolution":
    case "lookingGlassSetup":
    case "updatesCheck":
    case "signIn":
    case "installPlugin":
      return
  }
}

export const mock: Commands = {
  shell: () => Promise.resolve(shell),
  detect: () => Promise.resolve({ ...detection, configured: status.configured }),
  plan: async () => {
    await wait(500)
    return plan
  },
  apply: async () => {
    await wait(1400)
    update({ configured: true, vm, state: "stopped", firstBoot: { stage: "start", storageReady: null, networkReady: null } })
  },
  status: () => Promise.resolve(status),
  metrics: () => Promise.resolve(metrics()),
  run,
  saveCredential: async (secret) => {
    await wait(300)
    if (!secret) throw new Error("Enter the Windows password or PIN")
    update({ credentialSaved: true })
  },
  forgetCredential: async () => {
    await wait(300)
    update({ credentialSaved: false })
  },
  setAppearance: (appearance) => {
    shell.appearance = appearance
    return Promise.resolve({ ...shell })
  },
  agentPrompt: (agent) => Promise.resolve(`Set up window-side with ${agent}: run \`ws setup --help\` first.`),
}
