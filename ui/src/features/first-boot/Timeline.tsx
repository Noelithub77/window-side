import { Check, type LucideIcon } from "lucide-react"
import { motion } from "motion/react"
import { cn } from "@/lib/utils"

export type Look = { label: string; icon: LucideIcon; color: string }

const GAP = 132

/** Horizontal stage track: a gradient fills toward the current stage, done stages check off. */
export function Timeline({ steps, current }: { steps: Look[]; current: number }) {
  const width = (steps.length - 1) * GAP
  return (
    <ol className="relative flex" aria-label="First boot" style={{ width: width + 40 }}>
      <span aria-hidden className="absolute top-5 left-5 h-0.5 -translate-y-1/2 rounded-full bg-foreground/10" style={{ width }} />
      <motion.span
        aria-hidden
        className="absolute top-5 left-5 h-0.5 -translate-y-1/2 rounded-full bg-gradient-to-r from-mint to-lavender"
        initial={false}
        animate={{ width: Math.max(0, current) * GAP }}
        transition={{ type: "spring", stiffness: 140, damping: 22 }}
      />
      {steps.map(({ label, icon: Icon, color }, index) => {
        const done = index < current
        const active = index === current
        return (
          <li
            key={label}
            aria-current={active ? "step" : undefined}
            className="absolute flex w-24 -translate-x-1/2 flex-col items-center gap-2"
            style={{ left: 20 + index * GAP }}
          >
            <motion.span
              className={cn(
                "relative z-10 grid size-10 place-items-center rounded-full bg-secondary ring-1 transition-colors duration-300 [&_svg]:size-4",
                done && "text-mint ring-mint/40",
                !active && !done && "text-muted-foreground ring-border",
              )}
              style={active ? { background: color, color: "var(--ink)", boxShadow: `0 0 26px -4px ${color}` } : undefined}
              animate={{ scale: active ? 1.1 : 1 }}
              transition={{ type: "spring", stiffness: 300, damping: 20 }}
            >
              {done ? <Check strokeWidth={3} /> : <Icon />}
            </motion.span>
            <span className={cn("text-xs", active ? "font-medium text-foreground" : "text-muted-foreground")}>{label}</span>
          </li>
        )
      })}
      <span aria-hidden className="h-16" />
    </ol>
  )
}
