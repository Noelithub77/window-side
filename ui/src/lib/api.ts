import { listen } from "@tauri-apps/api/event"
import { commands } from "@/bindings"
import type { Commands, Events, Result } from "@/bindings"
import { mock, mockListen } from "@/lib/mock"

/** True inside the Tauri window; browser preview and UI tests use the mock. */
export const native = "__TAURI_INTERNALS__" in window

async function unwrap<T>(result: Promise<Result<T, string>>): Promise<T> {
  const value = await result
  if (value.status === "error") throw new Error(value.error)
  return value.data
}

async function empty(result: Promise<Result<null, string>>): Promise<void> {
  await unwrap(result)
}

const tauri: Commands = {
  shell: () => unwrap(commands.shell()),
  detect: () => unwrap(commands.detect()),
  plan: (choices) => unwrap(commands.plan(choices)),
  apply: (choices) => empty(commands.apply(choices)),
  status: () => unwrap(commands.status()),
  metrics: () => unwrap(commands.metrics()),
  run: (action) => empty(commands.run(action)),
  saveCredential: (secret) => empty(commands.saveCredential(secret)),
  forgetCredential: () => empty(commands.forgetCredential()),
  setAppearance: (appearance) => unwrap(commands.setAppearance(appearance)),
  agentPrompt: (agent) => unwrap(commands.agentPrompt(agent)),
}
export const api: Commands = native ? tauri : mock

let listeners = 0
let accounting = Promise.resolve()
function count(delta: number) {
  listeners += delta
  const value = listeners
  accounting = accounting.then(() => empty(commands.subscriptions(value))).catch((error: unknown) => {
    console.error(message(error))
  })
}
export function subscribe<K extends keyof Events>(event: K, handler: (payload: Events[K]) => void): () => void {
  if (!native) return mockListen(event, handler)
  let active = true
  let counted = false
  const stop = listen<Events[K]>(event, ({ payload }) => { handler(payload) })
  void stop.then(() => { if (active) { count(1); counted = true } }).catch((error: unknown) => { console.error(message(error)) })
  return () => {
    if (!active) return
    active = false
    void stop.then((unlisten) => { unlisten(); if (counted) count(-1) }).catch((error: unknown) => { console.error(message(error)) })
  }
}
export function message(error: unknown): string {
  if (typeof error === "string") return error
  if (error instanceof Error) return error.message
  return "Something went wrong"
}
