import type { LucideIcon } from "lucide-react"
import { Check } from "lucide-react"
import { motion } from "motion/react"
import { cn } from "@/lib/utils"

export type Step = { label: string; icon: LucideIcon }

type Props = { steps: Step[]; current: number; onPick: (step: number) => void; locked: boolean }

const ROW = 64

/** Vertical progress: done steps check off, the line fills toward the current one. */
export function StepRail({ steps, current, onPick, locked }: Props) {
  return (
    <ol className="relative flex flex-col" aria-label="Setup steps">
      <span aria-hidden className="absolute top-5 left-5 w-0.5 -translate-x-1/2 rounded-full bg-foreground/10" style={{ height: (steps.length - 1) * ROW }} />
      <motion.span
        aria-hidden
        className="absolute top-5 left-5 w-0.5 -translate-x-1/2 rounded-full bg-gradient-to-b from-mint to-lavender"
        animate={{ height: current * ROW }}
        transition={{ type: "spring", stiffness: 160, damping: 24 }}
      />
      {steps.map(({ label, icon: Icon }, index) => {
        const done = index < current
        const active = index === current
        return (
          <li key={label} style={{ height: ROW }}>
            <button
              type="button"
              disabled={locked || index >= current}
              aria-current={active ? "step" : undefined}
              onClick={() => {
                onPick(index)
              }}
              className="group flex items-center gap-3 disabled:cursor-default"
            >
              <motion.span
                className={cn(
                  "relative z-10 grid size-10 place-items-center rounded-full ring-1 transition-colors duration-300 [&_svg]:size-4",
                  active && "bg-primary text-primary-foreground ring-primary shadow-[0_0_24px_-4px_var(--mint)]",
                  done && "bg-secondary text-mint ring-mint/40 group-hover:ring-mint",
                  !active && !done && "bg-secondary text-muted-foreground ring-border",
                )}
                animate={{ scale: active ? 1.06 : 1 }}
              >
                {done ? <Check strokeWidth={3} /> : <Icon />}
              </motion.span>
              <span className={cn("text-sm transition-colors", active ? "font-medium text-foreground" : "text-muted-foreground")}>{label}</span>
            </button>
          </li>
        )
      })}
    </ol>
  )
}
