import { Cpu, FileCode2, Gpu, HardDrive, Info, MemoryStick, Package, Users } from "lucide-react"
import type { LucideIcon } from "lucide-react"
import { motion } from "motion/react"
import { IconTile } from "@/components/app/IconTile"
import { Accordion, AccordionContent, AccordionItem, AccordionTrigger } from "@/components/ui/accordion"
import { Badge } from "@/components/ui/badge"
import { CodeBlock, CodeBlockContent, CodeBlockHeader, CodeBlockTitle } from "@/components/ui/code-block"
import { Skeleton } from "@/components/ui/skeleton"
import type { Plan } from "@/lib/contract"

const FACT_ICONS: Record<string, { icon: LucideIcon; color: string }> = {
  Windows: { icon: HardDrive, color: "var(--sky)" },
  Graphics: { icon: Gpu, color: "var(--rose)" },
  Memory: { icon: MemoryStick, color: "var(--lavender)" },
  Processor: { icon: Cpu, color: "var(--mint)" },
}

function File({ path, contents }: { path: string; contents: string }) {
  return (
    <CodeBlock size="sm" className="bg-ink/60">
      <CodeBlockHeader>
        <CodeBlockTitle className="font-mono text-xs">{path}</CodeBlockTitle>
      </CodeBlockHeader>
      <CodeBlockContent lineNumbers className="max-h-56 text-xs">
        {contents}
      </CodeBlockContent>
    </CodeBlock>
  )
}

export function ReviewStep({ plan }: { plan: Plan | null }) {
  if (!plan) {
    return (
      <div className="grid grid-cols-2 gap-3">
        {[0, 1, 2, 3].map((slot) => (
          <Skeleton key={`fact-${String(slot)}`} className="h-24 rounded-3xl" />
        ))}
      </div>
    )
  }
  return (
    <div className="flex flex-col gap-4">
      <div className="grid grid-cols-2 gap-3">
        {plan.facts.map((fact, index) => {
          const look = FACT_ICONS[fact.label] ?? { icon: Info, color: "var(--mint)" }
          return (
            <motion.div
              key={fact.label}
              initial={{ opacity: 0, y: 12, scale: 0.97 }}
              animate={{ opacity: 1, y: 0, scale: 1 }}
              transition={{ delay: index * 0.07, type: "spring", stiffness: 260, damping: 22 }}
              className="glass sheen flex items-center gap-4 rounded-3xl p-5 ring-1 ring-border"
            >
              <IconTile icon={look.icon} color={look.color} />
              <div className="flex min-w-0 flex-col">
                <span className="text-xs text-muted-foreground">{fact.label}</span>
                <span className="font-medium text-balance">{fact.value}</span>
              </div>
            </motion.div>
          )
        })}
      </div>
      {plan.notes.length > 0 && (
        <div className="flex flex-col gap-2 px-1">
          {plan.notes.map((note) => (
            <div key={note} className="flex items-start gap-2.5 text-sm text-muted-foreground">
              <Info className="mt-0.5 size-4 shrink-0 text-sky" />
              {note}
            </div>
          ))}
        </div>
      )}
      <Accordion type="single" collapsible className="glass rounded-3xl px-5 ring-1 ring-border">
        <AccordionItem value="changes" className="border-none">
          <AccordionTrigger className="hover:no-underline">
            <span className="flex items-center gap-2.5">
              <FileCode2 className="size-4 text-muted-foreground" />
              System changes
              <Badge variant="secondary" className="rounded-full tabular">
                {plan.files.length + 1}
              </Badge>
            </span>
          </AccordionTrigger>
          <AccordionContent className="flex flex-col gap-3">
            {(plan.packages.length > 0 || plan.groups.length > 0) && (
              <div className="flex flex-wrap gap-2">
                {plan.packages.map((name) => (
                  <Badge key={name} variant="outline" className="gap-1.5 rounded-full">
                    <Package className="size-3" />
                    {name}
                  </Badge>
                ))}
                {plan.groups.map((name) => (
                  <Badge key={name} variant="outline" className="gap-1.5 rounded-full">
                    <Users className="size-3" />
                    {name}
                  </Badge>
                ))}
              </div>
            )}
            {plan.files.map((file) => (
              <File key={file.path} path={file.path} contents={file.contents} />
            ))}
            <File path="libvirt domain" contents={plan.domainXml} />
          </AccordionContent>
        </AccordionItem>
      </Accordion>
    </div>
  )
}
