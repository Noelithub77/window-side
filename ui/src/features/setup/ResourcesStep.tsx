import { useState } from "react"
import NumberFlow from "@number-flow/react"
import { Cpu, MemoryStick, Terminal } from "lucide-react"
import { motion } from "motion/react"
import { IconTile } from "@/components/app/IconTile"
import { Panel } from "@/components/app/Panel"
import { WindowsMark } from "@/components/app/WindowsMark"
import { Slider } from "@/components/ui/slider"
import type { Choices, Host } from "@/bindings"
import { cn } from "@/lib/utils"

type Props = { host: Host; choices: Choices; change: (next: Partial<Choices>) => void }

/** Host memory as one bar: Windows' share on the left, what Linux keeps on the right. */
function MemorySplit({ host, choices, change }: Props) {
  const total = Math.round(host.memoryMib / 1024)
  const windows = Math.round(choices.memoryMib / 1024)
  const share = (windows / Math.max(1, total)) * 100
  return (
    <Panel delay={0}>
      <div className="flex items-end justify-between">
        <div className="flex items-center gap-3">
          <IconTile icon={MemoryStick} color="var(--lavender)" />
          <div className="flex items-baseline gap-1.5 font-heading">
            <NumberFlow value={windows} className="text-4xl font-semibold tabular" />
            <span className="text-muted-foreground">GiB</span>
          </div>
        </div>
        <div className="flex items-center gap-2 text-sm text-muted-foreground tabular">
          <Terminal className="size-4" />
          <NumberFlow value={total - windows} /> GiB
        </div>
      </div>
      <div className="relative rounded-2xl ring-ring has-[[role=slider]:focus-visible]:ring-2">
        <div className="flex h-10 gap-1.5 overflow-hidden rounded-2xl">
          <motion.div
            className="flex items-center rounded-2xl bg-gradient-to-r from-mint/90 to-sky/80 pl-3"
            animate={{ width: `${share.toFixed(1)}%` }}
            transition={{ type: "spring", stiffness: 300, damping: 30 }}
          >
            <WindowsMark size={14} className="[&>*]:bg-primary-foreground" />
          </motion.div>
          <div className="flex flex-1 items-center justify-end rounded-2xl bg-lavender/20 pr-3 text-lavender">
            <Terminal className="size-4" />
          </div>
        </div>
        <Slider
          aria-label="Windows memory"
          className="absolute inset-0 opacity-0 [&_[data-slot=slider-track]]:h-10"
          min={2}
          max={Math.floor(host.memoryLimitMib / 1024)}
          step={1}
          value={[windows]}
          onValueChange={([value]) => {
            if (value !== undefined) change({ memoryMib: value * 1024 })
          }}
        />
      </div>
    </Panel>
  )
}

/** One cell per host thread; click a cell to give Windows that many. */
function ThreadGrid({ host, choices, change }: Props) {
  const [hover, setHover] = useState<number | null>(null)
  const shown = hover ?? choices.vcpus
  return (
    <Panel delay={0.08}>
      <div className="flex items-center gap-3">
        <IconTile icon={Cpu} color="var(--mint)" />
        <div className="flex items-baseline gap-1.5 font-heading">
          <NumberFlow value={shown} className="text-4xl font-semibold tabular" />
          <span className="text-muted-foreground">/ {host.threads}</span>
        </div>
      </div>
      <fieldset className="grid gap-2" style={{ gridTemplateColumns: `repeat(${String(Math.min(host.threads, 10))}, minmax(0, 1fr))` }}>
        <legend className="sr-only">Windows threads</legend>
        {Array.from({ length: host.threads }, (_, thread) => {
          const count = Math.min(Math.max(thread + 1, 2), host.vcpuLimit)
          const given = thread < shown
          const reachable = thread < host.vcpuLimit
          return (
            <motion.button
              key={`thread-${String(thread)}`}
              type="button"
              aria-label={`${String(count)} threads`}
              aria-pressed={thread < choices.vcpus}
              disabled={!reachable}
              onMouseEnter={() => {
                setHover(count)
              }}
              onMouseLeave={() => {
                setHover(null)
              }}
              onClick={() => {
                change({ vcpus: count })
              }}
              initial={{ opacity: 0, scale: 0.4 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ delay: 0.15 + thread * 0.018, type: "spring", stiffness: 400, damping: 22 }}
              className={cn(
                "aspect-square rounded-xl transition-colors duration-200",
                given ? "bg-mint shadow-[0_0_18px_-6px_var(--mint)]" : "bg-foreground/8 hover:bg-foreground/15",
                thread >= host.performanceThreads && "rounded-full",
                !reachable && "opacity-30",
              )}
            />
          )
        })}
      </fieldset>
    </Panel>
  )
}

export function ResourcesStep(props: Props) {
  return (
    <div className="flex flex-col gap-4">
      <MemorySplit {...props} />
      <ThreadGrid {...props} />
    </div>
  )
}
