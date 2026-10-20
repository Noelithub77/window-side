import { useCallback, useEffect, useState } from "react"
import { ArrowLeft, ArrowRight, ClipboardCheck, Cpu, Gpu, HardDrive, MemoryStick, RefreshCw, ShieldCheck } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"
import { GlowButton } from "@/components/app/GlowButton"
import { IconTile } from "@/components/app/IconTile"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { Skeleton } from "@/components/ui/skeleton"
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip"
import { api, message } from "@/lib/api"
import type { Choices, Plan } from "@/bindings"
import { gib } from "@/lib/format"
import { useApp } from "@/lib/store"
import { Applied } from "@/features/setup/Applied"
import { GraphicsStep } from "@/features/setup/GraphicsStep"
import { ResourcesStep } from "@/features/setup/ResourcesStep"
import { ReviewStep } from "@/features/setup/ReviewStep"
import { type Step, StepRail } from "@/features/setup/StepRail"
import { WindowsStep } from "@/features/setup/WindowsStep"

type Page = Step & { title: string; color: string }

const FIRST: Page = { label: "Windows", title: "Choose Windows", icon: HardDrive, color: "var(--sky)" }
const STEPS: Page[] = [
  FIRST,
  { label: "Graphics", title: "Graphics", icon: Gpu, color: "var(--rose)" },
  { label: "Resources", title: "Resources", icon: Cpu, color: "var(--mint)" },
  { label: "Review", title: "Review", icon: ClipboardCheck, color: "var(--lavender)" },
]

const LAST = STEPS.length - 1

function Detecting() {
  return (
    <div className="flex flex-col gap-3">
      {[0, 1].map((slot) => (
        <Skeleton key={`card-${String(slot)}`} className="h-32 rounded-3xl" />
      ))}
    </div>
  )
}

export function Setup() {
  const { detection, detecting, detect, status, edit, refresh } = useApp()
  const [step, setStep] = useState(0)
  const [direction, setDirection] = useState(1)
  const [edited, setEdited] = useState<Choices | null>(null)
  const [plan, setPlan] = useState<Plan | null>(null)
  const [planning, setPlanning] = useState(false)
  const [applying, setApplying] = useState(false)
  const [applied, setApplied] = useState(false)
  const choices = edited ?? detection?.choices ?? null
  const busy = detecting || planning || applying

  useEffect(() => {
    void detect()
  }, [detect])

  const go = useCallback(
    (next: number) => {
      setDirection(next > step ? 1 : -1)
      setStep(next)
    },
    [step],
  )

  const forward = useCallback(async () => {
    if (!choices || busy) return
    if (step < LAST - 1) {
      go(step + 1)
      return
    }
    if (step === LAST - 1) {
      go(LAST)
      setPlanning(true)
      try {
        setPlan(await api.plan(choices))
      } catch (error) {
        toast.error(message(error))
        go(LAST - 1)
      } finally {
        setPlanning(false)
      }
      return
    }
    setApplying(true)
    // The status event lands before apply resolves; editing keeps this view up for the celebration.
    const editing = useApp.getState().editing
    edit(true)
    try {
      await api.apply(choices)
      setApplied(true)
      window.setTimeout(() => {
        edit(false)
        void refresh()
      }, 2200)
    } catch (error) {
      edit(editing)
      toast.error(message(error))
    } finally {
      setApplying(false)
    }
  }, [busy, choices, edit, go, refresh, step])

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (!event.altKey) return
      if (event.key === "ArrowRight" && step < LAST) void forward()
      if (event.key === "ArrowLeft" && step > 0 && !busy) go(step - 1)
    }
    window.addEventListener("keydown", onKey)
    return () => {
      window.removeEventListener("keydown", onKey)
    }
  }, [busy, forward, go, step])

  function change(next: Partial<Choices>) {
    if (!choices) return
    setEdited({ ...choices, ...next })
    setPlan(null)
  }

  const current = STEPS[step] ?? FIRST
  const host = detection?.host
  let body = <Detecting />
  if (detection && choices && !detecting) {
    if (step === 0) body = <WindowsStep detection={detection} choices={choices} change={change} />
    else if (step === 1) body = <GraphicsStep detection={detection} choices={choices} change={change} />
    else if (step === 2) body = <ResourcesStep host={detection.host} choices={choices} change={change} />
    else body = <ReviewStep plan={plan} />
  }

  return (
    <div className="relative flex min-h-0 flex-1">
      <aside className="flex w-60 shrink-0 flex-col justify-between px-8 pt-6 pb-8">
        <StepRail steps={STEPS} current={step} onPick={go} locked={busy} />
        {host && (
          <div className="flex flex-col gap-2.5 text-xs text-muted-foreground tabular">
            <span className="flex items-center gap-2">
              <Cpu className="size-3.5" />
              {host.threads}
            </span>
            <span className="flex items-center gap-2">
              <MemoryStick className="size-3.5" />
              {gib(host.memoryMib)}
            </span>
          </div>
        )}
      </aside>

      <section className="flex min-w-0 flex-1 flex-col">
        <ScrollArea className="min-h-0 flex-1">
          <div className="mx-auto flex w-full max-w-3xl flex-col gap-6 px-8 pt-6 pb-10">
            <AnimatePresence mode="popLayout" initial={false} custom={direction}>
              <motion.div
                key={step}
                custom={direction}
                className="flex flex-col gap-6"
                initial={{ opacity: 0, x: direction * 48, filter: "blur(6px)" }}
                animate={{ opacity: 1, x: 0, filter: "blur(0px)" }}
                exit={{ opacity: 0, x: direction * -48, filter: "blur(6px)" }}
                transition={{ type: "spring", stiffness: 280, damping: 30 }}
              >
                <div className="flex items-center gap-4">
                  <IconTile icon={current.icon} color={current.color} size="lg" />
                  <h1 className="font-heading text-3xl font-semibold tracking-tight">{current.title}</h1>
                </div>
                {body}
              </motion.div>
            </AnimatePresence>
          </div>
        </ScrollArea>

        <footer className="flex items-center gap-2 px-8 py-5">
          <Button variant="ghost" className="rounded-full" disabled={busy || step === 0} onClick={() => { go(step - 1) }}>
            <ArrowLeft />
            Back
          </Button>
          <div className="flex-1" />
          {status?.configured && (
            <Button variant="ghost" className="rounded-full" disabled={busy} onClick={() => { edit(false) }}>
              Cancel
            </Button>
          )}
          {step === 0 && (
            <Tooltip>
              <TooltipTrigger asChild>
                <Button variant="outline" size="icon" className="rounded-full" aria-label="Scan again" disabled={busy} onClick={() => void detect()}>
                  <RefreshCw className={detecting ? "animate-spin" : undefined} />
                </Button>
              </TooltipTrigger>
              <TooltipContent>Scan again</TooltipContent>
            </Tooltip>
          )}
          <GlowButton onClick={() => void forward()} disabled={!choices || (step === LAST && !plan)} loading={planning || applying}>
            {step === LAST ? (
              <>
                <ShieldCheck />
                Apply
              </>
            ) : (
              <>
                {step === LAST - 1 ? "Review" : "Continue"}
                <ArrowRight />
              </>
            )}
          </GlowButton>
        </footer>
      </section>
      <AnimatePresence>{applied && <Applied />}</AnimatePresence>
    </div>
  )
}
