import { useState } from "react"
import { Gpu, RotateCcw, Terminal } from "lucide-react"
import { motion } from "motion/react"
import { WindowsMark } from "@/components/app/WindowsMark"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogMedia,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import type { BootProfile } from "@/bindings"
import { useApp } from "@/lib/store"
import { cn } from "@/lib/utils"

const OWNERS: { profile: BootProfile; label: string; hint: string }[] = [
  { profile: "normal", label: "Linux", hint: "Linux keeps the graphics card" },
  { profile: "gpu", label: "Windows", hint: "Windows gets the graphics card" },
]

/** Who owns the graphics card; switching restarts the computer into that boot profile. */
export function GpuOwner({ profile }: { profile: BootProfile }) {
  const run = useApp((state) => state.run)
  const busy = useApp((state) => state.busy)
  const [pending, setPending] = useState<BootProfile | null>(null)
  const target = OWNERS.find((owner) => owner.profile === pending)

  return (
    <>
      <fieldset className="glass flex items-center gap-1 rounded-full p-1 ring-1 ring-border">
        <legend className="sr-only">Graphics card owner</legend>
        <Gpu className="mx-2 size-4 text-rose" aria-hidden />
        {OWNERS.map((owner) => {
          const selected = owner.profile === profile
          return (
            <Tooltip key={owner.profile}>
              <TooltipTrigger asChild>
                <button
                  type="button"
                  aria-pressed={selected}
                  disabled={busy !== null}
                  onClick={() => {
                    if (!selected) setPending(owner.profile)
                  }}
                  className={cn(
                    "relative flex h-8 items-center gap-2 rounded-full px-3.5 text-xs font-medium transition-colors disabled:opacity-50",
                    selected ? "text-ink" : "text-muted-foreground hover:text-foreground",
                  )}
                >
                  {selected && (
                    <motion.span
                      layoutId="gpu-owner"
                      className="absolute inset-0 rounded-full bg-rose shadow-[0_0_24px_-6px_var(--rose)]"
                      transition={{ type: "spring", stiffness: 420, damping: 34 }}
                    />
                  )}
                  <span className="relative flex items-center gap-2">
                    {owner.profile === "gpu" ? (
                      <WindowsMark size={12} className={selected ? "[&>*]:bg-ink" : "[&>*]:bg-muted-foreground"} />
                    ) : (
                      <Terminal className="size-3.5" />
                    )}
                    {owner.label}
                  </span>
                </button>
              </TooltipTrigger>
              <TooltipContent>{owner.hint}</TooltipContent>
            </Tooltip>
          )
        })}
      </fieldset>

      <AlertDialog
        open={target !== undefined}
        onOpenChange={(open) => {
          if (!open) setPending(null)
        }}
      >
        <AlertDialogContent size="sm">
          <AlertDialogHeader>
            <AlertDialogMedia className="bg-rose/15 text-rose">
              <RotateCcw />
            </AlertDialogMedia>
            <AlertDialogTitle>Restart the computer?</AlertDialogTitle>
            <AlertDialogDescription>{target?.hint}.</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              className="bg-rose text-ink hover:bg-rose/85"
              onClick={() => {
                if (pending) void run("Restarting", { kind: "reboot", profile: pending })
              }}
            >
              Restart
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
