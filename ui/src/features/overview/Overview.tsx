import { HardDriveDownload } from "lucide-react"
import { AnimatePresence, LayoutGroup } from "motion/react"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { useApp } from "@/lib/store"
import { LiveHero, RestHero } from "@/features/overview/Hero"
import { Vitals } from "@/features/overview/Vitals"

export function Overview() {
  const status = useApp((state) => state.status)
  const available = useApp((state) => state.metrics.available)
  if (!status) return null
  const live = status.state === "running" || status.state === "paused"
  return (
    <div className="mx-auto flex min-h-0 w-full max-w-6xl flex-1 flex-col gap-4 px-8 pt-2 pb-8">
      {status.storageError && (
        <Alert variant="destructive" className="glass">
          <HardDriveDownload />
          <AlertTitle>Windows drive</AlertTitle>
          <AlertDescription>{status.storageError}</AlertDescription>
        </Alert>
      )}
      <LayoutGroup>
        {live ? <LiveHero status={status} /> : <RestHero status={status} />}
        <AnimatePresence>{live && <Vitals waiting={!available} />}</AnimatePresence>
      </LayoutGroup>
    </div>
  )
}
