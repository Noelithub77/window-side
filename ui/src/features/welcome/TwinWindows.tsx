import { motion } from "motion/react"

const LINES = [
  { width: 92, color: "var(--lavender)" },
  { width: 64, color: "var(--muted-foreground)" },
  { width: 110, color: "var(--muted-foreground)" },
  { width: 48, color: "var(--mint)" },
]

const SPARKS = [
  { x: 36, y: 50, size: 10, color: "var(--peach)", delay: 0 },
  { x: 300, y: 36, size: 16, color: "var(--peach)", delay: 0.8 },
  { x: 24, y: 176, size: 9, color: "var(--lavender)", delay: 1.4 },
  { x: 326, y: 182, size: 8, color: "var(--mint)", delay: 0.4 },
]

function Spark({ x, y, size, color, delay }: (typeof SPARKS)[number]) {
  return (
    <motion.path
      d={`M${x} ${y - size}Q${x} ${y} ${x + size} ${y}Q${x} ${y} ${x} ${y + size}Q${x} ${y} ${x - size} ${y}Q${x} ${y} ${x} ${y - size}Z`}
      fill={color}
      style={{ transformOrigin: `${x}px ${y}px` }}
      animate={{ scale: [0.6, 1.1, 0.6], opacity: [0.4, 1, 0.4], rotate: [0, 45, 0] }}
      transition={{ duration: 3.2, repeat: Infinity, delay, ease: "easeInOut" }}
    />
  )
}

/** Linux on the left, Windows sliding in beside it, joined by a soft beam. */
export function TwinWindows() {
  return (
    <svg viewBox="0 0 360 230" className="w-[min(420px,80vw)] overflow-visible" aria-hidden>
      <defs>
        <linearGradient id="beam" x1="0" x2="1">
          <stop offset="0" stopColor="var(--lavender)" stopOpacity="0" />
          <stop offset="0.5" stopColor="var(--mint)" stopOpacity="0.9" />
          <stop offset="1" stopColor="var(--mint)" stopOpacity="0" />
        </linearGradient>
        <linearGradient id="glass" x1="0" y1="0" x2="1" y2="1">
          <stop offset="0" stopColor="white" stopOpacity="0.1" />
          <stop offset="1" stopColor="white" stopOpacity="0.02" />
        </linearGradient>
        <filter id="soft" x="-50%" y="-50%" width="200%" height="200%">
          <feGaussianBlur stdDeviation="18" />
        </filter>
      </defs>

      <motion.ellipse
        cx="180" cy="118" rx="160" ry="72" fill="none" stroke="var(--border)" strokeDasharray="2 7" strokeWidth="1.5"
        style={{ transformOrigin: "180px 118px" }}
        animate={{ rotate: 360 }}
        transition={{ duration: 80, repeat: Infinity, ease: "linear" }}
      />
      <circle cx="120" cy="120" r="64" fill="var(--lavender)" opacity="0.22" filter="url(#soft)" />
      <circle cx="250" cy="130" r="70" fill="var(--mint)" opacity="0.2" filter="url(#soft)" />

      <motion.g
        initial={{ x: 20, opacity: 0 }}
        animate={{ x: 0, opacity: 1, y: [0, -4, 0] }}
        transition={{ x: { type: "spring", stiffness: 120, damping: 16 }, opacity: { duration: 0.5 }, y: { duration: 6, repeat: Infinity, ease: "easeInOut" } }}
      >
        <rect x="58" y="44" width="170" height="122" rx="18" fill="var(--secondary)" stroke="url(#glass)" />
        <rect x="58" y="44" width="170" height="122" rx="18" fill="url(#glass)" />
        {["var(--rose)", "var(--peach)", "var(--mint)"].map((color, index) => (
          <circle key={color} cx={78 + index * 12} cy="62" r="3.6" fill={color} />
        ))}
        {LINES.map((line, index) => (
          <motion.rect
            key={`${line.color}-${String(line.width)}`}
            x="78" y={84 + index * 17} height="6" rx="3" fill={line.color} opacity="0.75"
            initial={{ width: 0 }}
            animate={{ width: [0, line.width, line.width, 0] }}
            transition={{ duration: 7, times: [0, 0.18, 0.85, 1], repeat: Infinity, delay: 0.5 + index * 0.35, ease: "easeOut" }}
          />
        ))}
      </motion.g>

      <motion.rect
        x="150" y="118" width="80" height="3" rx="1.5" fill="url(#beam)"
        animate={{ opacity: [0.2, 1, 0.2], scaleX: [0.6, 1, 0.6] }}
        style={{ transformOrigin: "190px 119px" }}
        transition={{ duration: 2.6, repeat: Infinity, ease: "easeInOut" }}
      />

      <motion.g
        initial={{ x: 90, opacity: 0, rotate: 8 }}
        animate={{ x: 0, opacity: 1, rotate: 0, y: [0, 5, 0] }}
        transition={{ x: { type: "spring", stiffness: 90, damping: 14, delay: 0.25 }, rotate: { type: "spring", stiffness: 90, damping: 14, delay: 0.25 }, opacity: { duration: 0.6, delay: 0.25 }, y: { duration: 7, repeat: Infinity, ease: "easeInOut", delay: 1 } }}
        style={{ transformOrigin: "260px 128px" }}
      >
        <rect x="190" y="70" width="140" height="116" rx="20" fill="var(--secondary)" stroke="url(#glass)" />
        <rect x="200" y="80" width="120" height="96" rx="12" fill="var(--ink)" opacity="0.55" />
        {[0, 1, 2, 3].map((pane) => (
          <motion.rect
            key={pane}
            x={235 + (pane % 2) * 27} y={102 + Math.floor(pane / 2) * 27} width="23" height="23" rx="7"
            fill="var(--mint)"
            animate={{ opacity: [[1, 0.8, 0.8, 0.58][pane] ?? 1, 1, [1, 0.8, 0.8, 0.58][pane] ?? 1] }}
            transition={{ duration: 2.4, repeat: Infinity, delay: [0, 0.3, 0.9, 0.6][pane] ?? 0 }}
          />
        ))}
      </motion.g>

      {SPARKS.map((spark) => (
        <Spark key={`${spark.x}-${spark.y}`} {...spark} />
      ))}
    </svg>
  )
}
