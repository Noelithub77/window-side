import { useState } from "react"
import { ArrowDown, ArrowUp, Cpu, Gpu, MemoryStick, Network } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { Area, AreaChart, CartesianGrid, YAxis } from "recharts"
import { ChartContainer, type ChartConfig } from "@/components/ui/chart"
import type { Sample } from "@/lib/contract"
import { bytes } from "@/lib/format"
import { useApp } from "@/lib/store"
import { Caption, Gauge, Tile } from "@/features/overview/Gauge"

type Vital = "cpu" | "memory" | "gpu" | "network"

const LOOK = {
  cpu: { label: "Processor", icon: Cpu, color: "var(--mint)" },
  memory: { label: "Memory", icon: MemoryStick, color: "var(--lavender)" },
  gpu: { label: "Graphics", icon: Gpu, color: "var(--rose)" },
  network: { label: "Network", icon: Network, color: "var(--sky)" },
} as const

function rate(value: number | null): string {
  return value === null ? "–" : `${bytes(value)}/s`
}

function NetworkTile({ latest, active, onSelect }: { latest: Sample | undefined; active: boolean; onSelect: () => void }) {
  const look = LOOK.network
  const rows = [
    { icon: ArrowDown, value: latest?.down ?? null, color: "var(--sky)", label: "Download" },
    { icon: ArrowUp, value: latest?.up ?? null, color: "var(--peach)", label: "Upload" },
  ]
  return (
    <Tile label={look.label} color={look.color} active={active} onSelect={onSelect}>
      <div className="flex w-full flex-1 flex-col justify-center gap-3">
        {rows.map(({ icon: Icon, value, color, label }) => (
          <div key={label} className="flex items-center gap-3" title={label}>
            <span className="grid size-8 place-items-center rounded-xl" style={{ color, background: `color-mix(in oklch, ${color} 14%, transparent)` }}>
              <Icon className="size-4" />
            </span>
            <span className="font-heading text-lg font-semibold tabular">{rate(value)}</span>
          </div>
        ))}
      </div>
      <Caption icon={look.icon} label={look.label} color={look.color} />
    </Tile>
  )
}

const NETWORK_CONFIG = {
  down: { label: "Download", color: "var(--sky)" },
  up: { label: "Upload", color: "var(--peach)" },
} satisfies ChartConfig

/** A beacon pinned to the newest sample's edge so the chart reads as live. */
function Pulse({ color }: { color: string }) {
  return (
    <span aria-hidden className="pointer-events-none absolute top-5 right-5 flex size-2.5">
      <span className="absolute inset-0 animate-ping rounded-full opacity-70" style={{ background: color }} />
      <span className="relative size-2.5 rounded-full" style={{ background: color, boxShadow: `0 0 12px ${color}` }} />
    </span>
  )
}

/** The selected vital's recent history as a soft glowing area. */
function History({ vital, samples }: { vital: Vital; samples: Sample[] }) {
  const look = LOOK[vital]
  const keys = vital === "network" ? (["down", "up"] as const) : ([vital] as const)
  const config: ChartConfig = vital === "network" ? NETWORK_CONFIG : { [vital]: { label: look.label, color: look.color } }
  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={vital}
        className="relative min-h-0 flex-1"
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: -8 }}
        transition={{ duration: 0.25 }}
      >
        <ChartContainer config={config} className="aspect-auto size-full">
          <AreaChart data={samples} margin={{ top: 8, right: 0, bottom: 0, left: 0 }}>
            <defs>
              {keys.map((key) => (
                <linearGradient key={key} id={`fill-${key}`} x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stopColor={`var(--color-${key})`} stopOpacity={0.42} />
                  <stop offset="100%" stopColor={`var(--color-${key})`} stopOpacity={0} />
                </linearGradient>
              ))}
            </defs>
            <CartesianGrid vertical={false} strokeDasharray="3 6" className="stroke-foreground/6" />
            <YAxis hide domain={[0, (top: number) => Math.max(vital === "network" ? 1024 : 10, top * 1.35)]} />
            {keys.map((key) => (
              <Area
                key={key}
                dataKey={key}
                type="monotone"
                stroke={`var(--color-${key})`}
                strokeWidth={2.5}
                fill={`url(#fill-${key})`}
                isAnimationActive={false}
                connectNulls
                dot={false}
                activeDot={false}
                style={{ filter: `drop-shadow(0 4px 10px var(--color-${key}))` }}
              />
            ))}
          </AreaChart>
        </ChartContainer>
        <Pulse color={look.color} />
      </motion.div>
    </AnimatePresence>
  )
}

export function Vitals() {
  const samples = useApp((state) => state.metrics.samples)
  const [vital, setVital] = useState<Vital>("cpu")
  const latest = samples.at(-1)
  const pick = (next: Vital) => () => {
    setVital(next)
  }
  const gauges = [
    { key: "cpu", value: latest?.cpu ?? null, detail: null },
    { key: "memory", value: latest?.memory ?? null, detail: latest?.memoryUsed?.split(" / ")[0] ?? null },
    { key: "gpu", value: latest?.gpu ?? null, detail: null },
  ] as const
  return (
    <motion.section
      className="flex min-h-0 flex-1 flex-col gap-4"
      initial={{ opacity: 0, y: 16 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, y: 16 }}
      transition={{ type: "spring", stiffness: 220, damping: 26, delay: 0.1 }}
    >
      <div className="grid grid-cols-4 gap-4">
        {gauges.map(({ key, value, detail }) => (
          <Gauge key={key} {...LOOK[key]} value={value} detail={detail} active={vital === key} onSelect={pick(key)} />
        ))}
        <NetworkTile latest={latest} active={vital === "network"} onSelect={pick("network")} />
      </div>
      <div className="glass sheen flex min-h-40 flex-1 flex-col overflow-hidden rounded-3xl pt-4 ring-1 ring-border">
        <History vital={vital} samples={samples} />
      </div>
    </motion.section>
  )
}
