import { motion } from "motion/react"
import { cn } from "@/lib/utils"

type Props = { size?: number; live?: boolean; className?: string }

function breathing(pane: number, opacity: number, index: number): React.CSSProperties & { "--rest": number } {
  return { width: pane, height: pane, borderRadius: pane * 0.3, opacity, "--rest": opacity, animationDelay: `${String([0, 0.3, 0.9, 0.6][index] ?? 0)}s` }
}

/** Four rounded panes; they breathe in turn while Windows runs, as a CSS animation the compositor runs. */
export function WindowsMark({ size = 40, live = false, className }: Props) {
  const pane = size * 0.41
  const gap = size * 0.18 - 1
  return (
    <div className={cn("grid grid-cols-2", className)} style={{ width: size, height: size, gap }}>
      {[1, 0.8, 0.8, 0.58].map((opacity, index) => live ? (
        <div key={`pane-${String(index)}`} className="animate-breathe bg-primary motion-reduce:animate-none" style={breathing(pane, opacity, index)} />
      ) : (
        <motion.div
          key={`pane-${String(index)}`}
          className="bg-primary"
          style={{ width: pane, height: pane, borderRadius: pane * 0.3 }}
          initial={{ opacity: 0, scale: 0.6 }}
          animate={{ opacity, scale: 1 }}
          transition={{ type: "spring", stiffness: 260, damping: 18, delay: index * 0.06 }}
        />
      ))}
    </div>
  )
}
