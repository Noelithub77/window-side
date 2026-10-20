import { create } from "zustand"
import { toast } from "sonner"
import { api, message, subscribe } from "@/lib/api"
import type { Action, Detection, Metrics, Shell, Status } from "@/bindings"

export type Page = "overview" | "settings"

type State = {
  shell: Shell | null
  status: Status | null
  metrics: Metrics
  detection: Detection | null
  detecting: boolean
  page: Page
  /** Set once the user leaves the welcome screen. */
  started: boolean
  /** The setup wizard over a configured VM. */
  editing: boolean
  /** Label of the running action, such as "Starting". */
  busy: string | null
  load: () => Promise<void>
  go: (page: Page) => void
  start: () => void
  edit: (open: boolean) => void
  run: (label: string, action: Action) => Promise<void>
  refresh: () => Promise<void>
  detect: () => Promise<void>
  setShell: (shell: Shell) => void
}

export const useApp = create<State>()((set, get) => ({
  shell: null,
  status: null,
  metrics: { available: false, samples: [] },
  detection: null,
  detecting: false,
  page: "overview",
  started: false,
  editing: false,
  busy: null,
  async load() {
    subscribe("status", (status) => {
      set({ status })
    })
    subscribe("metrics", (metrics) => {
      set({ metrics })
    })
    try {
      const [shell, status, metrics] = await Promise.all([api.shell(), api.status(), api.metrics()])
      set({ shell, status, metrics })
    } catch (error) {
      toast.error(message(error))
    }
  },
  go(page) {
    set({ page })
  },
  start() {
    set({ started: true })
  },
  edit(open) {
    set({ editing: open })
  },
  async run(label, action) {
    if (get().busy) return
    set({ busy: label })
    try {
      await api.run(action)
    } catch (error) {
      toast.error(message(error))
    } finally {
      set({ busy: null })
      await get().refresh()
    }
  },
  async detect() {
    if (get().detecting) return
    set({ detecting: true })
    try {
      set({ detection: await api.detect() })
    } catch (error) {
      toast.error(message(error))
    } finally {
      set({ detecting: false })
    }
  },
  setShell(shell) {
    set({ shell })
  },
  async refresh() {
    try {
      const [status, metrics] = await Promise.all([api.status(), api.metrics()])
      set({ status, metrics })
    } catch (error) {
      toast.error(message(error))
    }
  },
}))
