import { expect, test, type Page } from "@playwright/test"

async function switchPage(page: Page, target: "Settings" | "Overview") {
  return page.evaluate(async (name) => {
    const button = [...document.querySelectorAll<HTMLButtonElement>("nav button")].find((item) => item.textContent.trim() === name)
    if (!button) throw new Error(`Missing navigation: ${name}`)
    const started = performance.now()
    button.click()
    let mounted: number | null = null
    return new Promise<{ mounted: number; readable: number }>((resolve, reject) => {
      function sample() {
        if (performance.now() - started >= 3000) {
          reject(new Error(`${name} did not become readable`))
          return
        }
        const content = name === "Settings"
          ? [...document.querySelectorAll("h2")].find((item) => item.textContent === "Appearance")
          : document.querySelector("h1")
        if (content) {
          mounted ??= performance.now() - started
          let opacity = 1
          for (let element: Element | null = content; element; element = element.parentElement) {
            opacity *= Number(getComputedStyle(element).opacity)
          }
          if (opacity >= 0.95) {
            resolve({ mounted, readable: performance.now() - started })
            return
          }
        }
        requestAnimationFrame(sample)
      }
      requestAnimationFrame(sample)
    })
  }, target)
}

for (const fixture of ["stopped", "status"]) {
  test(`tab navigation responds promptly with Windows ${fixture}`, async ({ page }, testInfo) => {
    await page.goto(`/?fixture=${fixture}`)
    await expect(page.getByRole("heading", { name: "Windows", exact: true })).toBeVisible()
    await page.waitForTimeout(1200)
    const measurements: { target: string; mounted: number; readable: number }[] = []
    await (["Settings", "Overview", "Settings", "Overview"] as const).reduce(async (previous, target) => {
      await previous
      measurements.push({ target, ...await switchPage(page, target) })
      await page.waitForTimeout(400)
    }, Promise.resolve())
    console.log(JSON.stringify({ fixture, measurements }))
    await testInfo.attach("navigation.json", { body: JSON.stringify(measurements, null, 2), contentType: "application/json" })
    for (const measurement of measurements) {
      expect.soft(measurement.mounted, `${measurement.target} mount delay`).toBeLessThan(200)
      expect.soft(measurement.readable, `${measurement.target} readable delay`).toBeLessThan(400)
    }
  })
}

test("rapid tab changes leave only the last requested page interactive", async ({ page }, testInfo) => {
  await page.goto("/?fixture=status")
  await expect(page.getByRole("heading", { name: "Windows", exact: true })).toBeVisible()
  await page.evaluate(async () => {
    await new Promise<void>((resolve) => {
      const names = ["Settings", "Overview", "Settings", "Overview", "Settings"]
      names.forEach((name, index) => setTimeout(() => {
        const button = [...document.querySelectorAll<HTMLButtonElement>("nav button")].find((item) => item.textContent.trim() === name)
        button?.click()
        if (index === names.length - 1) resolve()
      }, 40 * index))
    })
  })
  await expect(page.getByRole("button", { name: "Change", exact: true })).toBeVisible()
  await expect(page.locator("main")).toHaveCount(1)
  await expect(page.locator("main > [inert]")).toHaveCount(0)
  await expect(page.getByRole("button", { name: "Open", exact: true })).toHaveCount(0)
  await page.screenshot({ path: testInfo.outputPath("settings.png") })
})
