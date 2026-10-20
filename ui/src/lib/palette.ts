import { useEffect } from "react"
import { useApp } from "@/lib/store"

type Rgb = [number, number, number]

const OVERRIDES = ["--mint", "--lavender", "--rose", "--sky", "--peach", "--ink"] as const

function rgb(hex: string): Rgb | null {
  const match = /^#?([\da-f]{2})([\da-f]{2})([\da-f]{2})$/i.exec(hex)
  if (!match) return null
  return [Number.parseInt(match[1] ?? "0", 16) / 255, Number.parseInt(match[2] ?? "0", 16) / 255, Number.parseInt(match[3] ?? "0", 16) / 255]
}

/** Applies the Omarchy theme colours when chosen; returns the backdrop tint. */
export function usePalette(): Rgb | null {
  const shell = useApp((state) => state.shell)
  const palette = shell?.appearance === "omarchy" ? shell.palette : null

  useEffect(() => {
    const root = document.documentElement.style
    if (!palette) {
      OVERRIDES.forEach((name) => {
        root.removeProperty(name)
      })
      return
    }
    const [red, green, yellow, blue, magenta, cyan] = palette.colors
    const values: Record<(typeof OVERRIDES)[number], string | undefined> = {
      "--mint": palette.accent,
      "--lavender": magenta ?? blue,
      "--rose": red,
      "--sky": cyan ?? blue,
      "--peach": yellow ?? green,
      "--ink": palette.background,
    }
    OVERRIDES.forEach((name) => {
      const value = values[name]
      if (value) root.setProperty(name, value)
    })
  }, [palette])

  return palette ? rgb(palette.accent) : null
}
