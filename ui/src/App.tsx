import { useEffect } from "react"
import { AnimatePresence, MotionConfig, motion, useIsPresent } from "motion/react"
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
  return useApp(({ status, started, editing, page }) => {
    if (!status) return "loading"
    if (!status.configured) return started ? "setup" : "welcome"
    if (editing) return "setup"
    if (status.firstBoot) return "first-boot"
    return page
  })
}

function useMood(): Mood {
  return useApp(({ status, busy }) => {
    if (busy) return "busy"
    if (status?.state !== "running") return "idle"
    return status.bootProfile === "gpu" ? "gpu" : "live"
  })
}

const PAGES: Record<Exclude<View, "loading">, () => React.JSX.Element | null> = {
  welcome: Welcome,
  setup: Setup,
  "first-boot": FirstBoot,
  overview: Overview,
  settings: Settings,
}

function PageView({ view }: { view: View }) {
  const present = useIsPresent()
  const Page = view === "loading" ? null : PAGES[view]
  return (
    <motion.div
      className="absolute inset-0 flex min-h-0 flex-col"
      inert={!present}
      aria-hidden={!present}
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: -4 }}
      transition={{ duration: present ? 0.18 : 0.12, ease: "easeOut" }}
    >
      {Page ? <Page /> : (
        <div className="grid flex-1 place-items-center">
          <WindowsMark size={44} live />
        </div>
      )}
    </motion.div>
  )
}

export function App() {
  const load = useApp((state) => state.load)
  const view = useView()
  const mood = useMood()
  const tint = usePalette()

  useEffect(() => {
    void load()
  }, [load])

  return (
    <MotionConfig reducedMotion="user">
      <TooltipProvider delayDuration={300}>
        <Backdrop mood={mood} tint={tint} />
        <div className="flex h-full flex-col">
          <TitleBar navigable={view === "overview" || view === "settings"} />
          <main className="relative flex min-h-0 flex-1 flex-col">
            <AnimatePresence>
              <PageView key={view} view={view} />
            </AnimatePresence>
          </main>
        </div>
        <Toaster position="bottom-center" />
      </TooltipProvider>
    </MotionConfig>
  )
}
