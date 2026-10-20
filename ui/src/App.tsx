import { useEffect } from "react"
import { AnimatePresence, MotionConfig, motion } from "motion/react"
import { Backdrop, type Mood } from "@/components/app/Backdrop"
import { TitleBar } from "@/components/app/TitleBar"
import { WindowsMark } from "@/components/app/WindowsMark"
import { Toaster } from "@/components/ui/sonner"
import { TooltipProvider } from "@/components/ui/tooltip"
import { FirstBoot } from "@/features/first-boot/FirstBoot"
import { Overview } from "@/features/overview/Overview"
import { Settings } from "@/features/settings/Settings"
import { Setup } from "@/features/setup/Setup"
import { Welcome } from "@/features/welcome/Welcome"
import { usePalette } from "@/lib/palette"
import { useApp } from "@/lib/store"

type View = "loading" | "welcome" | "setup" | "first-boot" | "overview" | "settings"

function useView(): View {
  const { status, started, editing, page } = useApp()
  if (!status) return "loading"
  if (!status.configured) return started ? "setup" : "welcome"
  if (editing) return "setup"
  if (status.firstBoot) return "first-boot"
  return page
}

function useMood(): Mood {
  const { status, busy } = useApp()
  if (busy) return "busy"
  if (status?.state !== "running") return "idle"
  return status.bootProfile === "gpu" ? "gpu" : "live"
}

const PAGES: Record<Exclude<View, "loading">, () => React.JSX.Element | null> = {
  welcome: Welcome,
  setup: Setup,
  "first-boot": FirstBoot,
  overview: Overview,
  settings: Settings,
}

export function App() {
  const load = useApp((state) => state.load)
  const view = useView()
  const mood = useMood()
  const tint = usePalette()

  useEffect(() => {
    void load()
  }, [load])

  const Page = view === "loading" ? null : PAGES[view]
  return (
    <MotionConfig reducedMotion="user">
      <TooltipProvider delayDuration={300}>
        <Backdrop mood={mood} tint={tint} />
        <div className="flex h-full flex-col">
          <TitleBar navigable={view === "overview" || view === "settings"} />
          <AnimatePresence mode="wait">
            <motion.main
              key={view}
              className="relative flex min-h-0 flex-1 flex-col"
              initial={{ opacity: 0, y: 14, scale: 0.985, filter: "blur(10px)" }}
              animate={{ opacity: 1, y: 0, scale: 1, filter: "blur(0px)" }}
              exit={{ opacity: 0, y: -10, scale: 0.99, filter: "blur(8px)" }}
              transition={{ type: "spring", stiffness: 260, damping: 30, mass: 0.8 }}
            >
              {Page ? (
                <Page />
              ) : (
                <div className="grid flex-1 place-items-center">
                  <WindowsMark size={44} live />
                </div>
              )}
            </motion.main>
          </AnimatePresence>
        </div>
        <Toaster position="bottom-center" />
      </TooltipProvider>
    </MotionConfig>
  )
}
