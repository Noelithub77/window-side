import { Activity, AppWindow, Clock, Cpu, HardDrive, MemoryStick, Play, Power, Zap } from "lucide-react"
import { motion } from "motion/react"
import { GlowButton } from "@/components/app/GlowButton"
import { WindowsMark } from "@/components/app/WindowsMark"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Spinner } from "@/components/ui/spinner"
import type { Status, VmState } from "@/bindings"
import { gib, uptime } from "@/lib/format"
import { useApp } from "@/lib/store"
import { cn } from "@/lib/utils"
import { GpuOwner } from "@/features/overview/GpuOwner"

const SPRING = { type: "spring", stiffness: 260, damping: 30 } as const

const STATES: Record<VmState, { label: string; dot: string }> = {
  running: { label: "Running", dot: "bg-mint" },
  stopped: { label: "Stopped", dot: "bg-muted-foreground" },
  paused: { label: "Paused", dot: "bg-lavender" },
  stopping: { label: "Stopping", dot: "bg-peach" },
  unavailable: { label: "Unavailable", dot: "bg-destructive" },
}

function StateChip({ state }: { state: VmState }) {
  const busy = useApp((store) => store.busy)
  const look = STATES[state]
  return (
    <motion.span layoutId="hero-chip" transition={SPRING} className="glass inline-flex h-7 items-center gap-2 rounded-full px-3 text-xs font-medium ring-1 ring-border">
      {busy ? (
        <Spinner className="size-3 text-peach" />
      ) : (
        <span className="relative flex size-2">
          {state === "running" && <span className="absolute inset-0 animate-ping rounded-full bg-mint/70" />}
          <span className={cn("relative size-2 rounded-full", look.dot)} />
        </span>
      )}
      {busy ?? look.label}
    </motion.span>
  )
}

/** The mark with a soft halo that breathes while Windows runs. */
function Mark({ size, live }: { size: number; live: boolean }) {
  return (
    <motion.div layoutId="hero-mark" transition={SPRING} className="relative grid place-items-center" style={{ width: size * 1.6, height: size * 1.6 }}>
      <motion.span
        aria-hidden
        className="absolute inset-0 rounded-full bg-primary/25 blur-2xl"
        animate={live ? { scale: [0.85, 1.1, 0.85], opacity: [0.6, 1, 0.6] } : { scale: 0.7, opacity: 0.35 }}
        transition={live ? { duration: 3.2, repeat: Infinity, ease: "easeInOut" } : SPRING}
      />
      <WindowsMark size={size} live={live} />
    </motion.div>
  )
}

function Title() {
  return (
    <motion.h1 layoutId="hero-title" transition={SPRING} className="font-heading text-3xl font-semibold tracking-tight">
      Windows
    </motion.h1>
  )
}

function Stat({ icon: Icon, value, label }: { icon: typeof Clock; value: string; label: string }) {
  return (
    <span className="flex items-center gap-1.5 text-xs text-muted-foreground tabular" title={label}>
      <Icon className="size-3.5" />
      {value}
    </span>
  )
}

function PowerMenu() {
  const run = useApp((store) => store.run)
  const busy = useApp((store) => store.busy)
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="icon-lg" className="rounded-full" aria-label="Power" disabled={busy !== null}>
          <Power />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onSelect={() => void run("Shutting down", { kind: "stop" })}>
          <Power />
          Shut down
        </DropdownMenuItem>
        <DropdownMenuItem variant="destructive" onSelect={() => void run("Stopping", { kind: "forceStop" })}>
          <Zap />
          Force stop
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

/** Running: a compact bar with stats and actions above the vitals. */
export function LiveHero({ status }: { status: Status }) {
  const run = useApp((store) => store.run)
  const busy = useApp((store) => store.busy)
  return (
    <motion.section layout transition={SPRING} className="glass sheen flex items-center gap-5 rounded-[2rem] py-4 pr-4 pl-3 ring-1 ring-border">
      <Mark size={34} live />
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center gap-3">
          <Title />
          <StateChip state={status.state} />
        </div>
        <motion.div className="flex items-center gap-4" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.2 }}>
          {status.uptimeSeconds !== null && <Stat icon={Clock} value={uptime(status.uptimeSeconds)} label="Uptime" />}
          {status.processes !== null && <Stat icon={Activity} value={String(status.processes)} label="Processes" />}
        </motion.div>
      </div>
      <div className="flex-1" />
      {status.vm?.gpu && status.bootProfile && <GpuOwner profile={status.bootProfile} />}
      <PowerMenu />
      <GlowButton loading={busy === "Opening"} onClick={() => void run("Opening", { kind: "console" })}>
        <AppWindow />
        Open
      </GlowButton>
    </motion.section>
  )
}

/** Stopped: the mark centre stage with one launch action. */
export function RestHero({ status }: { status: Status }) {
  const run = useApp((store) => store.run)
  const busy = useApp((store) => store.busy)
  const launching = busy === "Starting"
  return (
    <motion.section layout transition={SPRING} className="flex flex-1 flex-col items-center justify-center gap-6 pb-16">
      <Mark size={84} live={launching} />
      <div className="flex flex-col items-center gap-3">
        <Title />
        <StateChip state={status.state} />
        {status.vm && (
          <motion.div className="flex items-center gap-4" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.2 }}>
            <Stat icon={MemoryStick} value={gib(status.vm.memoryMib)} label="Memory" />
            <Stat icon={Cpu} value={String(status.vm.vcpus)} label="Threads" />
            <Stat icon={HardDrive} value={status.vm.diskMode === "whole" ? "Drive" : "Partition"} label="Windows storage" />
          </motion.div>
        )}
      </div>
      <GlowButton className="h-12 px-8" loading={launching} disabled={status.state !== "stopped" || busy !== null} onClick={() => void run("Starting", { kind: "launch" })}>
        <Play className="fill-current" />
        Launch
      </GlowButton>
      {status.vm?.gpu && status.bootProfile && (
        <motion.div initial={{ opacity: 0, y: 8 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.25 }}>
          <GpuOwner profile={status.bootProfile} />
        </motion.div>
      )}
    </motion.section>
  )
}
