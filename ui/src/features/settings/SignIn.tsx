import { useState } from "react"
import { Check, KeyRound, Trash2 } from "lucide-react"
import { AnimatePresence, motion } from "motion/react"
import { toast } from "sonner"
import { Panel } from "@/components/app/Panel"
import { Button } from "@/components/ui/button"
import { InputGroup, InputGroupAddon, InputGroupButton, InputGroupInput } from "@/components/ui/input-group"
import { Spinner } from "@/components/ui/spinner"
import { api, message } from "@/lib/api"
import { useApp } from "@/lib/store"

/** The Windows password or PIN kept in the keyring for automatic sign-in. */
export function SignIn({ saved, delay }: { saved: boolean; delay: number }) {
  const refresh = useApp((state) => state.refresh)
  const [secret, setSecret] = useState("")
  const [working, setWorking] = useState(false)

  async function act(task: () => Promise<unknown>) {
    setWorking(true)
    try {
      await task()
      setSecret("")
      await refresh()
    } catch (error) {
      toast.error(message(error))
    } finally {
      setWorking(false)
    }
  }

  return (
    <Panel
      title="Sign-in"
      icon={KeyRound}
      color="var(--peach)"
      delay={delay}
      aside={
        saved && (
          <span className="flex items-center gap-1.5 rounded-full bg-mint/12 px-2.5 py-1 text-xs font-medium text-mint">
            <Check className="size-3.5" />
            Saved
          </span>
        )
      }
    >
      <AnimatePresence mode="wait" initial={false}>
        {saved ? (
          <motion.div key="saved" className="flex items-center gap-3" initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }}>
            <div className="flex gap-1.5" aria-hidden>
              {Array.from({ length: 8 }, (_, dot) => (
                <motion.span
                  key={`dot-${String(dot)}`}
                  className="size-2 rounded-full bg-foreground/40"
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{ delay: dot * 0.04, type: "spring", stiffness: 500, damping: 20 }}
                />
              ))}
            </div>
            <div className="flex-1" />
            <Button variant="ghost" size="sm" className="rounded-full text-muted-foreground" disabled={working} onClick={() => void act(api.forgetCredential)}>
              {working ? <Spinner /> : <Trash2 />}
              Forget
            </Button>
          </motion.div>
        ) : (
          <motion.form
            key="form"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onSubmit={(event) => {
              event.preventDefault()
              void act(() => api.saveCredential(secret))
            }}
          >
            <InputGroup className="h-11 rounded-full">
              <InputGroupInput type="password" aria-label="Windows password or PIN" placeholder="Password or PIN" autoComplete="off" value={secret} onChange={(event) => { setSecret(event.target.value) }} />
              <InputGroupAddon align="inline-end">
                <InputGroupButton type="submit" variant="default" className="rounded-full" disabled={!secret || working}>
                  {working ? <Spinner /> : <Check />}
                  Save
                </InputGroupButton>
              </InputGroupAddon>
            </InputGroup>
          </motion.form>
        )}
      </AnimatePresence>
    </Panel>
  )
}
