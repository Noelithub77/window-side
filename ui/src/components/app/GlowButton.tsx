import type { ComponentProps } from "react"
import { motion } from "motion/react"
import { Spinner } from "@/components/ui/spinner"
import { cn } from "@/lib/utils"

type Props = ComponentProps<"button"> & { loading?: boolean; tone?: "primary" | "danger" }

/** The one primary action of a view: a pill with a slow light sweep. */
export function GlowButton({ className, children, loading = false, disabled, tone = "primary", ...props }: Props) {
  return (
    <button
      type="button"
      disabled={disabled || loading}
      className={cn(
        "group relative isolate inline-flex h-11 items-center justify-center gap-2 overflow-hidden rounded-full px-6 text-sm font-semibold transition-[transform,box-shadow,opacity] duration-300 active:scale-[0.97] disabled:opacity-45",
        tone === "primary"
          ? "bg-primary text-primary-foreground shadow-[0_10px_40px_-12px_var(--mint)] hover:shadow-[0_14px_50px_-10px_var(--mint)]"
          : "bg-destructive text-ink shadow-[0_10px_40px_-12px_var(--destructive)]",
        "[&_svg]:size-4 [&_svg]:transition-transform hover:[&_svg:last-child]:translate-x-0.5",
        className,
      )}
      {...props}
    >
      <motion.span
        aria-hidden
        className="absolute inset-y-0 -left-1/2 -z-10 w-1/2 -skew-x-12 bg-gradient-to-r from-transparent via-white/45 to-transparent"
        animate={{ x: ["0%", "420%"] }}
        transition={{ duration: 2.8, repeat: Infinity, repeatDelay: 2.2, ease: "easeInOut" }}
      />
      {loading && <Spinner />}
      {children}
    </button>
  )
}
