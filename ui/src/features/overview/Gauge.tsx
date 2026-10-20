import { type CSSProperties, useId } from "react"
import NumberFlow from "@number-flow/react"
import type { LucideIcon } from "lucide-react"
import { motion } from "motion/react"
import { cn } from "@/lib/utils"

const RADIUS = 40
const ARC = 2 * Math.PI * RADIUS * 0.75

type Props = {
  label: string
  icon: LucideIcon
  color: string
  value: number | null
  detail?: string | null
  active: boolean
  onSelect: () => void
}

/** A 270° ring with a glowing value arc; pressing it charts its history. */
export function Gauge({ label, icon: Icon, color, value, detail = null, active, onSelect }: Props) {
  const id = useId()
  const share = Math.min(100, Math.max(0, value ?? 0)) / 100
  return (
    <Tile label={label} color={color} active={active} onSelect={onSelect}>
      <div className="relative aspect-square w-full max-w-36">
        <svg viewBox="0 0 100 100" className="size-full rotate-135 overflow-visible">
          <defs>
            <linearGradient id={id} x1="0" y1="1" x2="1" y2="0">
              <stop offset="0%" stopColor={color} stopOpacity={0.45} />
              <stop offset="100%" stopColor={color} />
            </linearGradient>
          </defs>
          <circle cx="50" cy="50" r={RADIUS} fill="none" strokeWidth="7" strokeLinecap="round" strokeDasharray={`${String(ARC)} 999`} className="stroke-foreground/8" />
          <motion.circle
            cx="50"
            cy="50"
            r={RADIUS}
            fill="none"
            strokeWidth="7"
            strokeLinecap="round"
            stroke={`url(#${id})`}
            strokeDasharray={`${String(ARC)} 999`}
            style={{ filter: `drop-shadow(0 0 5px ${color})` }}
            initial={{ strokeDashoffset: ARC }}
            animate={{ strokeDashoffset: ARC * (1 - share), opacity: value === null ? 0 : 1 }}
            transition={{ type: "spring", stiffness: 60, damping: 16 }}
          />
        </svg>
        <div className="absolute inset-0 grid place-items-center">
          <div className="flex flex-col items-center">
            <div className="flex items-baseline font-heading">
              {value === null ? (
                <span className="text-3xl font-semibold text-muted-foreground">–</span>
              ) : (
                <>
                  <NumberFlow value={Math.round(value)} className="text-3xl font-semibold tabular" />
                  <span className="text-sm text-muted-foreground">%</span>
                </>
              )}
            </div>
            {detail && <span className="text-[11px] text-muted-foreground tabular">{detail}</span>}
          </div>
        </div>
      </div>
      <Caption icon={Icon} label={label} color={color} />
    </Tile>
  )
}

type TileProps = { label: string; color: string; active: boolean; onSelect: () => void; children: React.ReactNode }

function tone(color: string): CSSProperties & Record<"--tone", string> {
  return { "--tone": color }
}

/** The shared pressable card behind every vital. */
export function Tile({ label, color, active, onSelect, children }: TileProps) {
  return (
    <motion.button
      type="button"
      aria-pressed={active}
      aria-label={label}
      onClick={onSelect}
      whileHover={{ y: -2 }}
      whileTap={{ scale: 0.98 }}
      style={tone(color)}
      className={cn(
        "glass sheen relative flex flex-col items-center justify-between gap-2 overflow-hidden rounded-3xl p-5 ring-1 transition-shadow duration-300 outline-none focus-visible:ring-2",
        active ? "ring-(--tone)/50 shadow-[0_0_44px_-18px_var(--tone)]" : "ring-border",
      )}
    >
      {active && (
        <motion.span
          layoutId="vital-glow"
          aria-hidden
          className="absolute inset-0 -z-10 bg-[radial-gradient(80%_60%_at_50%_0%,color-mix(in_oklch,var(--tone)_18%,transparent),transparent)]"
          transition={{ type: "spring", stiffness: 300, damping: 30 }}
        />
      )}
      {children}
    </motion.button>
  )
}

export function Caption({ icon: Icon, label, color }: { icon: LucideIcon; label: string; color: string }) {
  return (
    <span className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
      <Icon className="size-3.5" style={{ color }} />
      {label}
    </span>
  )
}
