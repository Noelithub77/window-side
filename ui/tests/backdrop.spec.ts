import { expect, test } from "@playwright/test"

test("reduced-motion backdrop redraws on changes instead of continuously", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" })
  await page.addInitScript(() => {
    const original: unknown = Object.getOwnPropertyDescriptor(WebGL2RenderingContext.prototype, "drawElements")?.value
    if (typeof original !== "function") throw new Error("WebGL draw instrumentation is unavailable")
    WebGL2RenderingContext.prototype.drawElements = function (...args: Parameters<WebGL2RenderingContext["drawElements"]>) {
      const root = document.documentElement
      root.dataset.draws = String(Number(root.dataset.draws ?? 0) + 1)
      Reflect.apply(original, this, args)
    }
  })
  await page.goto("/?fixture=stopped")
  await expect(page.locator("canvas")).toBeVisible()
  await expect.poll(() => page.locator("html").getAttribute("data-draws")).not.toBeNull()
  await page.waitForTimeout(500)
  const draws = await page.locator("html").getAttribute("data-draws")
  await page.waitForTimeout(500)
  expect(await page.locator("html").getAttribute("data-draws")).toBe(draws)
  await page.setViewportSize({ width: 1152, height: 800 })
  await expect(page.locator("canvas")).toHaveAttribute("width", "576")
  await expect(page.locator("canvas")).toHaveAttribute("height", "400")
  await page.getByRole("button", { name: "Launch", exact: true }).click()
  await expect.poll(() => page.locator("html").getAttribute("data-draws")).not.toBe(draws)
  await page.emulateMedia({ reducedMotion: "no-preference" })
  await page.waitForTimeout(300)
  const moving = Number(await page.locator("html").getAttribute("data-draws"))
  await page.waitForTimeout(500)
  expect(Number(await page.locator("html").getAttribute("data-draws"))).toBeGreaterThan(moving + 2)
  await page.evaluate(() => {
    Object.defineProperty(document, "hidden", { configurable: true, value: true })
    document.dispatchEvent(new Event("visibilitychange"))
  })
  await page.waitForTimeout(200)
  const hidden = await page.locator("html").getAttribute("data-draws")
  await page.waitForTimeout(500)
  expect(await page.locator("html").getAttribute("data-draws")).toBe(hidden)
  await page.evaluate(() => {
    Reflect.deleteProperty(document, "hidden")
    document.dispatchEvent(new Event("visibilitychange"))
  })
  await expect.poll(() => page.locator("html").getAttribute("data-draws")).not.toBe(hidden)
})
