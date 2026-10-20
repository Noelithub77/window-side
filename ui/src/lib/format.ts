export function gib(mib: number): string {
  return `${Math.round(mib / 1024)} GiB`
}

export function bytes(value: number): string {
  const units = ["B", "KiB", "MiB", "GiB", "TiB"]
  let amount = value
  let unit = 0
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024
    unit += 1
  }
  return `${amount.toFixed(amount >= 100 || unit === 0 ? 0 : 1)} ${units[unit] ?? "B"}`
}

export function uptime(seconds: number | null): string {
  if (seconds === null) return "—"
  const minutes = Math.floor(Math.max(0, seconds) / 60)
  return minutes >= 60 ? `${Math.floor(minutes / 60)}h ${minutes % 60}m` : `${minutes}m`
}

export function plural(count: number, word: string): string {
  return `${count} ${word}${count === 1 ? "" : "s"}`
}
