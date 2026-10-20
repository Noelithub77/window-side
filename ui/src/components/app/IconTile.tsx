import type { LucideIcon } from "lucide-react"
import { cn } from "@/lib/utils"

type Props = { icon: LucideIcon; color?: string; className?: string; size?: "sm" | "md" | "lg" }

const SIZES = { sm: "size-8 rounded-xl [&_svg]:size-4", md: "size-11 rounded-2xl [&_svg]:size-5", lg: "size-14 rounded-[1.25rem] [&_svg]:size-6" }

/** A soft tinted square holding one icon; `color` is a CSS colour such as `var(--mint)`. */
export function IconTile({ icon: Icon, color = "var(--mint)", className, size = "md" }: Props) {
  return (
    <span
      className={cn("grid shrink-0 place-items-center", SIZES[size], className)}
      style={{ color, background: `color-mix(in oklch, ${color} 15%, transparent)`, boxShadow: `inset 0 0 0 1px color-mix(in oklch, ${color} 18%, transparent)` }}
    >
      <Icon />
    </span>
  )
}
