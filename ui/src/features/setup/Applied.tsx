import { motion } from "motion/react"
import { Lottie } from "@/components/app/Lottie"
import applied from "@/assets/lottie/applied.json"

/** Full-page moment after setup is applied, before first boot takes over. */
export function Applied() {
  return (
    <motion.div
      className="absolute inset-0 z-30 grid place-items-center bg-ink/60 backdrop-blur-md"
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
    >
      <div className="flex flex-col items-center gap-2">
        <Lottie data={applied} size={220} />
        <motion.span
          className="font-heading text-2xl font-semibold"
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.5 }}
        >
          Ready
        </motion.span>
      </div>
    </motion.div>
  )
}
