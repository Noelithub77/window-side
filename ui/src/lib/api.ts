import { invoke } from "@tauri-apps/api/core"
import { listen } from "@tauri-apps/api/event"
import type { Commands, Events } from "@/lib/contract"
import { mock, mockListen } from "@/lib/mock"

/** True inside the Tauri window; the browser preview and UI tests use the mock. */
export const native = "__TAURI_INTERNALS__" in window

const tauri: Commands = {
  shell: () => invoke("shell"),
  detect: () => invoke("detect"),
  plan: (choices) => invoke("plan", { choices }),
  apply: (choices) => invoke("apply", { choices }),
  status: () => invoke("status"),
  metrics: () => invoke("metrics"),
  run: (action) => invoke("run", { action }),
  saveCredential: (secret) => invoke("save_credential", { secret }),
  forgetCredential: () => invoke("forget_credential"),
  setAppearance: (appearance) => invoke("set_appearance", { appearance }),
  agentPrompt: (agent) => invoke("agent_prompt", { agent }),
}

export const api: Commands = native ? tauri : mock

export function subscribe<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
  if (!native) return mockListen(event, handler)
  const stop = listen<Events[K]>(event, ({ payload }) => {
    handler(payload)
  })
  return () => {
    void stop.then((unlisten) => {
      unlisten()
    })
  }
}

export function message(error: unknown): string {
  if (typeof error === "string") return error
  if (error instanceof Error) return error.message
  return "Something went wrong"
}
