import type { LucideIcon } from "lucide-react"
import { motion } from "motion/react"
import { IconTile } from "@/components/app/IconTile"
import { cn } from "@/lib/utils"

type Props = {
  children: React.ReactNode
  delay?: number
  title?: string
  icon?: LucideIcon
  color?: string
  /** Sits at the end of the title row, such as a status chip or action. */
  aside?: React.ReactNode
  className?: string
}

/** A frosted card that rises in; optionally titled with a tinted icon. */
export function Panel({ children, delay = 0, title, icon, color, aside, className }: Props) {
  return (
    <motion.section
      initial={{ opacity: 0, y: 14 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay, type: "spring", stiffness: 200, damping: 24 }}
      className={cn("glass sheen flex flex-col gap-5 rounded-3xl p-6 ring-1 ring-border", className)}
    >
      {title && (
        <header className="flex items-center gap-3">
          {icon && <IconTile icon={icon} color={color} size="sm" />}
          <h2 className="font-heading text-base font-semibold">{title}</h2>
          <div className="flex-1" />
          {aside}
        </header>
      )}
      {children}
    </motion.section>
  )
}
