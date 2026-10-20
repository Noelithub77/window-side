import { useState } from "react"
import { Check, Palette, PanelTop, Plus } from "lucide-react"
import { motion } from "motion/react"
import { toast } from "sonner"
import { ChoiceCard } from "@/components/app/ChoiceCard"
import { Panel } from "@/components/app/Panel"
import { Button } from "@/components/ui/button"
import { api, message } from "@/lib/api"
import type { Appearance as Look, Shell } from "@/bindings"
import { useApp } from "@/lib/store"

// Fixed copies of the pastel tokens: the live ones follow Omarchy while it is chosen.
const PASTEL = {
  ink: "oklch(0.155 0.014 285)",
  colors: ["oklch(0.86 0.105 170)", "oklch(0.79 0.105 292)", "oklch(0.8 0.1 355)", "oklch(0.83 0.08 232)", "oklch(0.85 0.085 58)"],
}

/** A tiny scene of overlapping glows in the theme's colours. */
function Swatch({ ink, colors }: { ink: string; colors: string[] }) {
  return (
    <div className="relative h-20 overflow-hidden rounded-2xl ring-1 ring-foreground/10" style={{ background: ink }}>
      {colors.slice(0, 5).map((color, index) => (
        <motion.span
          key={color}
          className="absolute size-14 rounded-full blur-md"
          style={{ background: color, left: `${String(index * 19 - 4)}%`, top: index % 2 === 0 ? "-20%" : "35%" }}
          animate={{ y: [0, index % 2 === 0 ? 6 : -6, 0] }}
          transition={{ duration: 4 + index, repeat: Infinity, ease: "easeInOut" }}
        />
      ))}
    </div>
  )
}

export function Appearance({ shell, delay }: { shell: Shell; delay: number }) {
  const setShell = useApp((state) => state.setShell)
  const run = useApp((state) => state.run)
  const busy = useApp((state) => state.busy)
  const [saving, setSaving] = useState(false)

  async function choose(look: Look) {
    if (look === shell.appearance || saving) return
    setSaving(true)
    try {
      setShell(await api.setAppearance(look))
    } catch (error) {
      toast.error(message(error))
    } finally {
      setSaving(false)
    }
  }

  const omarchy = shell.palette ? { ink: shell.palette.background, colors: [...new Set([shell.palette.accent, ...shell.palette.colors])] } : null
  return (
    <Panel title="Appearance" icon={Palette} color="var(--lavender)" delay={delay}>
      <div className="grid grid-cols-2 gap-3">
        <ChoiceCard name="appearance" label="Pastel" selected={shell.appearance === "pastel"} onSelect={() => void choose("pastel")} className="p-3">
          <Swatch {...PASTEL} />
          <span className="mt-2.5 block px-1 text-sm font-medium">Pastel</span>
        </ChoiceCard>
        {omarchy && (
          <ChoiceCard name="appearance" label="Omarchy" selected={shell.appearance === "omarchy"} onSelect={() => void choose("omarchy")} className="p-3">
            <Swatch {...omarchy} />
            <span className="mt-2.5 block px-1 text-sm font-medium">Omarchy</span>
          </ChoiceCard>
        )}
      </div>
      {shell.omarchy && (
        <div className="flex items-center gap-3 rounded-2xl bg-foreground/4 py-2 pr-2 pl-4">
          <PanelTop className="size-4 text-muted-foreground" />
          <span className="text-sm">Omarchy panel</span>
          <div className="flex-1" />
          {shell.panelInstalled ? (
            <span className="mr-1 flex items-center gap-1.5 rounded-full bg-mint/12 px-2.5 py-1 text-xs font-medium text-mint">
              <Check className="size-3.5" />
              Added
            </span>
          ) : (
            <Button variant="ghost" size="sm" className="rounded-full" disabled={busy !== null} onClick={() => void run("Adding to panel", { kind: "installPlugin" })}>
              <Plus />
              Add
            </Button>
          )}
        </div>
      )}
    </Panel>
  )
}
