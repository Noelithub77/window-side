import type { ReactNode } from "react"
import { Check } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { cn } from "@/lib/utils"

type Props = {
  name: string
  selected: boolean
  disabled?: boolean
  onSelect: () => void
  children: ReactNode
  className?: string
  label: string
}

/** A radio drawn as a surface: lifts on hover, glows and checks itself when chosen. */
export function ChoiceCard({ name, selected, disabled = false, onSelect, children, className, label }: Props) {
  return (
    <motion.label
      whileHover={disabled ? undefined : { y: -2 }}
      whileTap={disabled ? undefined : { scale: 0.985 }}
      className={cn(
        "glass sheen relative block w-full cursor-pointer rounded-3xl p-5 text-left ring-1 transition-[box-shadow,background-color] duration-300 has-[:focus-visible]:ring-2 has-[:focus-visible]:ring-ring",
        selected ? "ring-primary/70" : "ring-border hover:ring-foreground/20",
        disabled && "cursor-not-allowed opacity-50",
        className,
      )}
    >
      <input type="radio" name={name} aria-label={label} className="sr-only" checked={selected} disabled={disabled} onChange={onSelect} />
      <AnimatePresence>
        {selected && (
          <motion.span
            aria-hidden
            className="pointer-events-none absolute inset-0 rounded-3xl bg-[radial-gradient(120%_90%_at_0%_0%,color-mix(in_oklch,var(--mint)_16%,transparent),transparent_60%)]"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          />
        )}
      </AnimatePresence>
      <span
        aria-hidden
        className={cn(
          "absolute top-5 right-5 grid size-5 place-items-center rounded-full ring-1 transition-colors",
          selected ? "bg-primary ring-primary" : "ring-foreground/20",
        )}
      >
        <AnimatePresence>
          {selected && (
            <motion.span initial={{ scale: 0, rotate: -40 }} animate={{ scale: 1, rotate: 0 }} exit={{ scale: 0 }} transition={{ type: "spring", stiffness: 500, damping: 20 }}>
              <Check className="size-3 text-primary-foreground" strokeWidth={3} />
            </motion.span>
          )}
        </AnimatePresence>
      </span>
      <div className="relative">{children}</div>
    </motion.label>
  )
}
