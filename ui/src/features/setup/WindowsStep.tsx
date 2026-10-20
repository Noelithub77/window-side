import { HardDrive, SearchX } from "lucide-react"
import { motion } from "motion/react"
import { Badge } from "@/components/ui/badge"
import { ChoiceCard } from "@/components/app/ChoiceCard"
import { IconTile } from "@/components/app/IconTile"
import type { Choices, Detection } from "@/bindings"
import { bytes } from "@/lib/format"

type Props = { detection: Detection; choices: Choices; change: (next: Partial<Choices>) => void }

/** How much of the drive Windows takes, sweeping in on first show. */
function DiskBar({ size, diskSize }: { size: number; diskSize: number }) {
  const share = Math.min(1, size / Math.max(1, diskSize))
  return (
    <div className="flex flex-col gap-2">
      <div className="flex h-2.5 gap-1 overflow-hidden rounded-full bg-foreground/8">
        <motion.div
          className="rounded-full bg-gradient-to-r from-mint to-sky"
          initial={{ width: 0 }}
          animate={{ width: `${(share * 100).toFixed(1)}%` }}
          transition={{ type: "spring", stiffness: 60, damping: 18, delay: 0.15 }}
        />
      </div>
      <div className="flex justify-between font-mono text-[11px] text-muted-foreground tabular">
        <span>{bytes(size)}</span>
        <span>{bytes(diskSize)}</span>
      </div>
    </div>
  )
}

export function WindowsStep({ detection, choices, change }: Props) {
  if (!detection.windows.length) {
    return (
      <div className="flex flex-col items-center gap-3 py-16 text-muted-foreground">
        <IconTile icon={SearchX} color="var(--peach)" size="lg" />
        No Windows install found
      </div>
    )
  }
  return (
    <div role="radiogroup" aria-label="Windows install" className="flex flex-col gap-3">
      {detection.windows.map((install, index) => (
        <motion.div key={install.partition} initial={{ opacity: 0, y: 12 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: index * 0.07 }}>
          <ChoiceCard
            name="windows"
            label={install.diskModel}
            selected={choices.windows === install.partition}
            onSelect={() => {
              change({ windows: install.partition })
            }}
          >
            <div className="flex flex-col gap-4 pr-8">
              <div className="flex items-center gap-3">
                <IconTile icon={HardDrive} color={index === 0 ? "var(--mint)" : "var(--sky)"} />
                <div className="flex min-w-0 flex-1 flex-col">
                  <span className="truncate font-medium">{install.diskModel}</span>
                  <span className="font-mono text-xs text-muted-foreground">{install.partition}</span>
                </div>
                {install.dedicatedDrive && (
                  <Badge variant="secondary" className="rounded-full">
                    Whole drive
                  </Badge>
                )}
              </div>
              <DiskBar size={install.size} diskSize={install.diskSize} />
            </div>
          </ChoiceCard>
        </motion.div>
      ))}
    </div>
  )
}
