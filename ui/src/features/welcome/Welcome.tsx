import { useEffect, useState } from "react"
import { ArrowRight, Bot, Check, HardDrive, Package, ShieldCheck } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"
import { GlowButton } from "@/components/app/GlowButton"
import { Button } from "@/components/ui/button"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "@/components/ui/dropdown-menu"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import { api, message } from "@/lib/api"
import type { AgentId } from "@/bindings"
import { plural } from "@/lib/format"
import { useApp } from "@/lib/store"
import { TwinWindows } from "@/features/welcome/TwinWindows"
import { cn } from "@/lib/utils"

type Check = { icon: typeof HardDrive; label: string; ok: boolean | null; detail: string }

function Readiness() {
  const detection = useApp((state) => state.detection)
  const host = detection?.host
  const checks: Check[] = [
    {
      icon: HardDrive,
      label: "Windows",
      ok: detection ? detection.windows.length > 0 : null,
      detail: detection ? (detection.windows.length ? `Found on ${plural(detection.windows.length, "partition")}` : "No Windows install found") : "Looking…",
    },
    { icon: ShieldCheck, label: "IOMMU", ok: host ? host.iommu : null, detail: host?.iommu ? "Ready for GPU passthrough" : "Off: setup can turn it on" },
    {
      icon: Package,
      label: "Tools",
      ok: host ? host.missingPackages.length === 0 : null,
      detail: host?.missingPackages.length ? `Setup installs ${host.missingPackages.join(", ")}` : "Everything installed",
    },
  ]
  return (
    <div className="flex gap-2">
      {checks.map(({ icon: Icon, label, ok, detail }, index) => (
        <Tooltip key={label}>
          <TooltipTrigger asChild>
            <motion.div
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.5 + index * 0.08 }}
              className="glass flex items-center gap-2 rounded-full py-1.5 pr-3.5 pl-2 text-xs ring-1 ring-border"
            >
              <span className="relative grid size-6 place-items-center rounded-full bg-foreground/5">
                <Icon className="size-3.5 text-muted-foreground" />
                <span
                  className={cn(
                    "absolute -top-0.5 -right-0.5 size-2 rounded-full ring-2 ring-ink transition-colors",
                    ok === null ? "animate-pulse bg-muted-foreground" : ok ? "bg-mint" : "bg-peach",
                  )}
                />
              </span>
              {label}
            </motion.div>
          </TooltipTrigger>
          <TooltipContent>{detail}</TooltipContent>
        </Tooltip>
      ))}
    </div>
  )
}

function AgentMenu() {
  const agents = useApp((state) => state.shell?.agents ?? [])
  const [copied, setCopied] = useState(false)

  async function copy(agent: AgentId) {
    try {
      await navigator.clipboard.writeText(await api.agentPrompt(agent))
      setCopied(true)
      window.setTimeout(() => {
        setCopied(false)
      }, 2400)
    } catch (error) {
      toast.error(message(error))
    }
  }

  if (!agents.length) return null
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" className="rounded-full text-muted-foreground">
          <AnimatePresence mode="wait" initial={false}>
            <motion.span key={copied ? "copied" : "agent"} initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -6 }} className="flex items-center gap-2">
              {copied ? <Check className="text-mint" /> : <Bot />}
              {copied ? "Prompt copied" : "Set up with an agent"}
            </motion.span>
          </AnimatePresence>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="center">
        {agents.map((agent) => (
          <DropdownMenuItem key={agent.id} onSelect={() => void copy(agent.id)}>
            <Bot />
            {agent.name}
          </DropdownMenuItem>
        ))}
      </DropdownMenuContent>
    </DropdownMenu>
  )
}

export function Welcome() {
  const { detect, start } = useApp()
  useEffect(() => {
    void detect()
  }, [detect])

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-8 pb-10">
      <TwinWindows />
      <motion.div
        className="flex flex-col items-center gap-3 text-center"
        initial={{ opacity: 0, y: 12 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3, type: "spring", stiffness: 200, damping: 24 }}
      >
        <h1 className="font-heading text-5xl font-semibold tracking-tight text-balance">
          Windows, right beside{" "}
          <span className="bg-gradient-to-r from-mint via-sky to-lavender bg-clip-text text-transparent">Linux</span>
        </h1>
        <p className="text-muted-foreground">The Windows you already have. No reboot.</p>
      </motion.div>
      <Readiness />
      <div className="flex flex-col items-center gap-2">
        <GlowButton onClick={start}>
          Get started
          <ArrowRight />
        </GlowButton>
        <AgentMenu />
      </div>
    </div>
  )
}
