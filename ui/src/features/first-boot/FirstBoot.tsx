import { useState } from "react"
import { AppWindow, Check, Disc3, Gpu, HardDrive, type LucideIcon, Power, RotateCcw, Sparkles, UserRound, Wifi } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { GlowButton } from "@/components/app/GlowButton"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Spinner } from "@/components/ui/spinner"
import type { Action, FirstBoot as Progress, FirstBootStage } from "@/bindings"
import { useApp } from "@/lib/store"
import { cn } from "@/lib/utils"
import { type Look, Timeline } from "@/features/first-boot/Timeline"

type Stage = Exclude<FirstBootStage, "complete">

const LOOKS: Record<Stage, Look & { title: string; line: string; action: { label: string; busy: string; icon: LucideIcon; run: Action } }> = {
  gpuBoot: {
    label: "Graphics",
    icon: Gpu,
    color: "var(--rose)",
    title: "Graphics profile",
    line: "Restart into the GPU profile to continue.",
    action: { label: "Restart", busy: "Restarting", icon: RotateCcw, run: { kind: "reboot", profile: "gpu" } },
  },
  start: {
    label: "Start",
    icon: Power,
    color: "var(--mint)",
    title: "Start Windows",
    line: "Windows opens with the driver disc attached.",
    action: { label: "Start", busy: "Starting", icon: Power, run: { kind: "firstBootStart" } },
  },
  drivers: {
    label: "Drivers",
    icon: Disc3,
    color: "var(--sky)",
    title: "Install drivers",
    line: "Run the installer on the driver disc inside Windows.",
    action: { label: "Open", busy: "Opening", icon: AppWindow, run: { kind: "console" } },
  },
  signIn: {
    label: "Sign in",
    icon: UserRound,
    color: "var(--peach)",
    title: "Sign in",
    line: "Sign in to Windows once.",
    action: { label: "Open", busy: "Opening", icon: AppWindow, run: { kind: "console" } },
  },
  ready: {
    label: "Finish",
    icon: Sparkles,
    color: "var(--lavender)",
    title: "Finish",
    line: "Windows restarts with fast drivers.",
    action: { label: "Finish", busy: "Finishing", icon: Sparkles, run: { kind: "firstBootFinish" } },
  },
}

/** The current stage's icon inside a slowly turning halo ring. */
function Orb({ look, working }: { look: Look; working: boolean }) {
  const Icon = look.icon
  return (
    <div className="relative grid size-36 place-items-center">
      <motion.span
        aria-hidden
        className="absolute inset-0 rounded-full"
        style={{ background: `conic-gradient(from 0deg, transparent 0deg, ${look.color} 120deg, transparent 240deg)`, mask: "radial-gradient(closest-side, transparent 88%, black 90%)" }}
        animate={{ rotate: 360 }}
        transition={{ duration: working ? 1.6 : 6, repeat: Infinity, ease: "linear" }}
      />
      <motion.span
        aria-hidden
        className="absolute inset-4 rounded-full blur-2xl"
        style={{ background: look.color }}
        animate={{ opacity: [0.18, 0.34, 0.18] }}
        transition={{ duration: 3, repeat: Infinity, ease: "easeInOut" }}
      />
      <span className="glass relative grid size-24 place-items-center rounded-full ring-1 ring-border" style={{ color: look.color }}>
        <Icon className="size-10" strokeWidth={1.5} />
      </span>
    </div>
  )
}

function Readiness({ icon: Icon, label, ready }: { icon: LucideIcon; label: string; ready: boolean | null }) {
  return (
    <span className={cn("glass flex h-8 items-center gap-2 rounded-full px-3 text-xs font-medium ring-1 transition-colors", ready ? "text-mint ring-mint/30" : "text-muted-foreground ring-border")}>
      <Icon className="size-3.5" />
      {label}
      {ready ? <Check className="size-3.5" strokeWidth={3} /> : <Spinner className="size-3" />}
    </span>
  )
}

function Act({ stage }: { stage: Stage }) {
  const run = useApp((state) => state.run)
  const busy = useApp((state) => state.busy)
  const [confirming, setConfirming] = useState(false)
  const { action } = LOOKS[stage]
  const Icon = action.icon
  const go = () => void run(action.busy, action.run)
  return (
    <>
      <GlowButton
        className="h-12 px-8"
        loading={busy === action.busy}
        disabled={busy !== null}
        onClick={() => {
          if (stage === "gpuBoot") setConfirming(true)
          else go()
        }}
      >
        <Icon />
        {action.label}
      </GlowButton>
      <AlertDialog open={confirming} onOpenChange={setConfirming}>
        <AlertDialogContent size="sm">
          <AlertDialogHeader>
            <AlertDialogTitle>Restart the computer?</AlertDialogTitle>
            <AlertDialogDescription>Windows gets the graphics card.</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction className="bg-rose text-ink hover:bg-rose/85" onClick={go}>
              Restart
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

export function FirstBoot() {
  const status = useApp((state) => state.status)
  const busy = useApp((state) => state.busy)
  const progress: Progress | null = status?.firstBoot ?? null
  if (!status || !progress || progress.stage === "complete") return null
  const stage = progress.stage
  const stages: Stage[] = status.vm?.gpu ? ["gpuBoot", "start", "drivers", "signIn", "ready"] : ["start", "drivers", "signIn", "ready"]
  const look = LOOKS[stage]

  return (
    <div className="flex min-h-0 flex-1 flex-col items-center gap-10 px-8 pt-6 pb-12">
      <Timeline steps={stages.map((item) => LOOKS[item])} current={stages.indexOf(stage)} />
      <AnimatePresence mode="wait">
        <motion.section
          key={stage}
          className="flex flex-1 flex-col items-center justify-center gap-6 pb-10"
          initial={{ opacity: 0, scale: 0.94, filter: "blur(8px)" }}
          animate={{ opacity: 1, scale: 1, filter: "blur(0px)" }}
          exit={{ opacity: 0, scale: 1.04, filter: "blur(8px)" }}
          transition={{ type: "spring", stiffness: 240, damping: 26 }}
        >
          <Orb look={look} working={busy !== null || stage === "drivers"} />
          <div className="flex flex-col items-center gap-2 text-center">
            <h1 className="font-heading text-3xl font-semibold tracking-tight">{look.title}</h1>
            <p className="text-sm text-muted-foreground">{look.line}</p>
          </div>
          {stage === "drivers" && (
            <div className="flex gap-2">
              <Readiness icon={HardDrive} label="Storage" ready={progress.storageReady} />
              <Readiness icon={Wifi} label="Network" ready={progress.networkReady} />
            </div>
          )}
          <Act stage={stage} />
        </motion.section>
      </AnimatePresence>
    </div>
  )
}
