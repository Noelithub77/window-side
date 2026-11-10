import { Keyboard } from "lucide-react"
import { toast } from "sonner"
import { Kbd, KbdGroup } from "@/components/ui/kbd"

// The viewer's keys, as `looking_glass.rs` configures them.
const CONTROLS = [
  { keys: ["Ctrl", "Alt"], label: "Grab or release input" },
  { keys: ["Insert"], label: "Viewer overlay" },
  { keys: ["Scroll Lock", "F"], label: "Fullscreen" },
  { keys: ["Scroll Lock", "Q"], label: "Close viewer" },
] as const

const HINT_KEY = "window-side:controls-hint"

/** The viewer's keyboard shortcuts while Windows is on the Looking Glass display. */
export function Controls() {
  return (
    <section aria-label="Viewer controls" className="glass sheen flex items-center gap-5 rounded-3xl px-5 py-3 ring-1 ring-border">
      <Keyboard className="size-4 shrink-0 text-muted-foreground" aria-hidden />
      <ul className="grid flex-1 grid-cols-4 gap-4">
        {CONTROLS.map(({ keys, label }) => (
          <li key={label} className="flex min-w-0 items-center gap-2 text-xs text-muted-foreground">
            <KbdGroup>
              {keys.map((key) => <Kbd key={key}>{key}</Kbd>)}
            </KbdGroup>
            <span className="truncate">{label}</span>
          </li>
        ))}
      </ul>
    </section>
  )
}

/** The first time the viewer opens, say how to get the mouse back. */
export function hintControlsOnce() {
  try {
    if (localStorage.getItem(HINT_KEY)) return
    localStorage.setItem(HINT_KEY, "shown")
  } catch {
    return
  }
  toast("Ctrl + Alt releases the mouse and keyboard", { description: "Overview lists the other viewer keys.", duration: 8000 })
}
