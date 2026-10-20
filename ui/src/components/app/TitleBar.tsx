import { getCurrentWindow } from "@tauri-apps/api/window"
import { Minus, Square, X } from "lucide-react"
import { motion } from "motion/react"
import { native } from "@/lib/api"
import { type Page, useApp } from "@/lib/store"
import { WindowsMark } from "@/components/app/WindowsMark"
import { cn } from "@/lib/utils"

const PAGES: { page: Page; label: string }[] = [
  { page: "overview", label: "Overview" },
  { page: "settings", label: "Settings" },
]

function Nav() {
  const page = useApp((state) => state.page)
  const go = useApp((state) => state.go)
  return (
    <nav className="glass flex gap-1 rounded-full p-1 ring-1 ring-border" aria-label="Pages">
      {PAGES.map((item) => (
        <button
          key={item.page}
          type="button"
          aria-current={page === item.page ? "page" : undefined}
          onClick={() => {
            go(item.page)
          }}
          className={cn(
            "relative rounded-full px-4 py-1.5 text-xs font-medium transition-colors",
            page === item.page ? "text-primary-foreground" : "text-muted-foreground hover:text-foreground",
          )}
        >
          {page === item.page && (
            <motion.span
              layoutId="nav-pill"
              className="absolute inset-0 rounded-full bg-primary"
              transition={{ type: "spring", stiffness: 420, damping: 34 }}
            />
          )}
          <span className="relative">{item.label}</span>
        </button>
      ))}
    </nav>
  )
}

function Controls() {
  if (!native) return null
  const window = getCurrentWindow()
  const buttons = [
    { label: "Minimize", icon: Minus, run: () => window.minimize() },
    { label: "Maximize", icon: Square, run: () => window.toggleMaximize() },
    { label: "Close", icon: X, run: () => window.close() },
  ]
  return (
    <div className="flex gap-1">
      {buttons.map(({ label, icon: Icon, run }) => (
        <button
          key={label}
          type="button"
          aria-label={label}
          onClick={() => void run()}
          className="grid size-7 place-items-center rounded-full text-muted-foreground transition-colors hover:bg-foreground/10 hover:text-foreground"
        >
          <Icon className="size-3.5" />
        </button>
      ))}
    </div>
  )
}

export function TitleBar({ navigable }: { navigable: boolean }) {
  return (
    <header data-tauri-drag-region className="relative z-20 grid h-14 shrink-0 grid-cols-[1fr_auto_1fr] items-center px-4">
      <div data-tauri-drag-region className="flex items-center gap-2.5">
        <WindowsMark size={16} />
        <span className="font-heading text-sm font-semibold tracking-tight">window-side</span>
      </div>
      <div>{navigable && <Nav />}</div>
      <div data-tauri-drag-region className="flex justify-end">
        <Controls />
      </div>
    </header>
  )
}
