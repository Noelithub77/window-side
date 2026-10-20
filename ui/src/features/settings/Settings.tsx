import { useState } from "react"
import { useShallow } from "zustand/react/shallow"
import NumberFlow from "@number-flow/react"
import { CircleCheck, Cpu, Download, Gpu, HardDrive, type LucideIcon, MemoryStick, MonitorPlay, MonitorSmartphone, RefreshCw, SlidersHorizontal } from "lucide-react"
import { IconTile } from "@/components/app/IconTile"
import { Panel } from "@/components/app/Panel"
import { WindowsMark } from "@/components/app/WindowsMark"
import { Button } from "@/components/ui/button"
import { ScrollArea } from "@/components/ui/scroll-area"
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group"
import type { VmSummary } from "@/bindings"
import { gib } from "@/lib/format"
import { useApp } from "@/lib/store"
import { Appearance } from "@/features/settings/Appearance"
import { SignIn } from "@/features/settings/SignIn"

function Spec({ icon, color, value, label }: { icon: LucideIcon; color: string; value: string; label: string }) {
  return (
    <div className="flex items-center gap-3 rounded-2xl bg-foreground/4 p-3">
      <IconTile icon={icon} color={color} size="sm" />
      <div className="flex min-w-0 flex-col">
        <span className="truncate text-sm font-medium tabular">{value}</span>
        <span className="text-[11px] text-muted-foreground">{label}</span>
      </div>
    </div>
  )
}

function Machine({ vm }: { vm: VmSummary }) {
  const edit = useApp((state) => state.edit)
  const busy = useApp((state) => state.busy)
  return (
    <Panel
      title="Windows"
      icon={HardDrive}
      color="var(--mint)"
      aside={
        <Button variant="ghost" size="sm" className="rounded-full" disabled={busy !== null} onClick={() => { edit(true) }}>
          <SlidersHorizontal />
          Change
        </Button>
      }
    >
      <div className="grid grid-cols-2 gap-2">
        <Spec icon={MemoryStick} color="var(--lavender)" value={gib(vm.memoryMib)} label="Memory" />
        <Spec icon={Cpu} color="var(--mint)" value={String(vm.vcpus)} label="Threads" />
        <Spec icon={Gpu} color="var(--rose)" value={vm.gpu ? "Passthrough" : "Virtual"} label="Graphics" />
        <Spec icon={HardDrive} color="var(--sky)" value={vm.diskMode === "whole" ? "Whole drive" : "Partition"} label="Storage" />
      </div>
    </Panel>
  )
}

const RESOLUTIONS = [
  { id: "1080", label: "1080p", width: 1920, height: 1080 },
  { id: "1440", label: "1440p", width: 2560, height: 1440 },
  { id: "2160", label: "4K", width: 3840, height: 2160 },
]

function Display({ running, gpu, delay }: { running: boolean; gpu: boolean; delay: number }) {
  const run = useApp((state) => state.run)
  const busy = useApp((state) => state.busy)
  const [picked, setPicked] = useState("")
  return (
    <Panel title="Display" icon={MonitorSmartphone} color="var(--sky)" delay={delay}>
      <ToggleGroup
        type="single"
        variant="outline"
        className="w-full"
        value={picked}
        disabled={!running || busy !== null}
        aria-label="Resolution"
        onValueChange={(value) => {
          const target = RESOLUTIONS.find((item) => item.id === value)
          if (!target) return
          setPicked(value)
          void run("Resizing", { kind: "resolution", width: target.width, height: target.height })
        }}
      >
        {RESOLUTIONS.map((item) => (
          <ToggleGroupItem key={item.id} value={item.id} className="h-10 flex-1 font-medium tabular">
            {item.label}
          </ToggleGroupItem>
        ))}
      </ToggleGroup>
      {gpu && (
        <div className="flex items-center gap-3 rounded-2xl bg-foreground/4 py-2 pr-2 pl-4">
          <MonitorPlay className="size-4 text-muted-foreground" />
          <span className="text-sm">Looking Glass</span>
          <div className="flex-1" />
          <Button variant="ghost" size="sm" className="rounded-full" disabled={busy !== null} onClick={() => void run("Setting up Looking Glass", { kind: "lookingGlassSetup" })}>
            Set up
          </Button>
        </div>
      )}
    </Panel>
  )
}

function Updates({ running, pending, delay }: { running: boolean; pending: number | null; delay: number }) {
  const run = useApp((state) => state.run)
  const busy = useApp((state) => state.busy)
  const count = pending ?? 0
  return (
    <Panel title="Windows Update" icon={Download} color="var(--peach)" delay={delay}>
      <div className="flex items-center gap-4">
        {count > 0 ? (
          <div className="flex items-baseline gap-2 font-heading">
            <NumberFlow value={count} className="text-4xl font-semibold text-peach tabular" />
            <Download className="size-4 text-muted-foreground" aria-label="waiting" />
          </div>
        ) : (
          <CircleCheck className={pending === 0 ? "size-7 text-mint" : "size-7 text-muted-foreground/40"} aria-label={pending === 0 ? "Up to date" : "Not checked"} />
        )}
        <div className="flex-1" />
        {count > 0 ? (
          <Button className="rounded-full" disabled={!running || busy !== null} onClick={() => void run("Installing updates", { kind: "updatesInstall" })}>
            <Download />
            Install
          </Button>
        ) : (
          <Button variant="outline" className="rounded-full" disabled={!running || busy !== null} onClick={() => void run("Checking updates", { kind: "updatesCheck" })}>
            <RefreshCw className={busy === "Checking updates" ? "animate-spin" : undefined} />
            Check
          </Button>
        )}
      </div>
    </Panel>
  )
}

export function Settings() {
  const vm = useApp(useShallow((state) => state.status?.vm ?? null))
  const running = useApp((state) => state.status?.state === "running")
  const pending = useApp((state) => state.status?.pendingUpdates ?? null)
  const credentialSaved = useApp((state) => state.status?.credentialSaved ?? null)
  const shell = useApp((state) => state.shell)
  return (
    <ScrollArea className="min-h-0 flex-1">
      <div className="mx-auto grid w-full max-w-5xl grid-cols-2 items-start gap-4 px-8 pt-2 pb-10">
        <div className="flex flex-col gap-4">
          {vm && <Machine vm={vm} />}
          <Display running={running} gpu={Boolean(vm?.gpu)} delay={0.03} />
          <Updates running={running} pending={pending} delay={0.06} />
        </div>
        <div className="flex flex-col gap-4">
          {credentialSaved !== null && <SignIn saved={credentialSaved} delay={0.02} />}
          {shell && <Appearance shell={shell} delay={0.05} />}
          {shell && (
            <div className="flex items-center justify-center gap-2 pt-2 text-xs text-muted-foreground/70 tabular">
              <WindowsMark size={12} />
              {shell.version}
            </div>
          )}
        </div>
      </div>
    </ScrollArea>
  )
}
