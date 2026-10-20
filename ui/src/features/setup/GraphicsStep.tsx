import { Gpu, Lock, Monitor } from "lucide-react"
import { motion } from "motion/react"
import { ChoiceCard } from "@/components/app/ChoiceCard"
import { IconTile } from "@/components/app/IconTile"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import type { Choices, Detection, GpuBlocker, GpuOption } from "@/lib/contract"

type Props = { detection: Detection; choices: Choices; change: (next: Partial<Choices>) => void }

const VENDOR: Record<GpuOption["vendor"], string> = {
  nvidia: "var(--mint)",
  amd: "var(--rose)",
  intel: "var(--sky)",
  other: "var(--lavender)",
}

const BLOCKER: Record<GpuBlocker, string> = {
  "host-display": "Linux draws its screen with it",
  "iommu-off": "IOMMU is off",
  "shared-group": "Shares its IOMMU group",
}

export function GraphicsStep({ detection, choices, change }: Props) {
  const options = [null, ...detection.gpus]
  return (
    <div role="radiogroup" aria-label="Graphics" className="grid grid-cols-2 gap-3">
      {options.map((gpu, index) => {
        const card = (
          <ChoiceCard
            name="gpu"
            label={gpu?.name ?? "Virtual display"}
            selected={choices.gpu === (gpu?.address ?? null)}
            disabled={gpu ? !gpu.usable : false}
            onSelect={() => {
              change({ gpu: gpu?.address ?? null })
            }}
            className="h-full"
          >
            <div className="flex flex-col gap-6 pr-6">
              <IconTile icon={gpu ? Gpu : Monitor} color={gpu ? VENDOR[gpu.vendor] : "var(--lavender)"} size="lg" />
              <div className="flex flex-col gap-1">
                <span className="font-medium leading-snug">{gpu?.name ?? "Virtual display"}</span>
                <span className="flex items-center gap-1.5 text-xs text-muted-foreground">
                  {gpu?.blocker && <Lock className="size-3" />}
                  {gpu ? (gpu.blocker ? BLOCKER[gpu.blocker] : "Passthrough") : "Linux keeps every GPU"}
                </span>
              </div>
            </div>
          </ChoiceCard>
        )
        return (
          <motion.div key={gpu?.address ?? "virtual"} initial={{ opacity: 0, scale: 0.96 }} animate={{ opacity: 1, scale: 1 }} transition={{ delay: index * 0.06 }}>
            {gpu?.blocker ? (
              <Tooltip>
                <TooltipTrigger asChild>
                  <div className="h-full">{card}</div>
                </TooltipTrigger>
                <TooltipContent className="font-mono">{gpu.address}</TooltipContent>
              </Tooltip>
            ) : (
              card
            )}
          </motion.div>
        )
      })}
    </div>
  )
}
