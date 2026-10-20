import { expect, test } from "@playwright/test"

test("welcome guides setup through review and apply", async ({ page }) => {
  await page.goto("/?fixture=setup")
  await expect(page.getByRole("heading", { name: "Windows, right beside Linux" })).toBeVisible()
  await page.getByRole("button", { name: "Get started" }).click()
  await expect(page.getByRole("heading", { name: "Choose Windows" })).toBeVisible()
  await page.getByRole("button", { name: "Continue", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Graphics", exact: true })).toBeVisible()
  await page.getByRole("button", { name: "Continue", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Resources", exact: true })).toBeVisible()
  await page.locator("footer").getByRole("button", { name: "Review", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Review", exact: true })).toBeVisible()
  await page.getByRole("button", { name: "Apply", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Start Windows", exact: true })).toBeVisible({ timeout: 10000 })
  await page.getByRole("button", { name: "Start", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Install drivers", exact: true })).toBeVisible()
})

test("ready first boot finishes into the overview", async ({ page }) => {
  await page.goto("/?fixture=first-boot-ready")
  await page.getByRole("button", { name: "Finish", exact: true }).click()
  await expect(page.getByRole("button", { name: "Open", exact: true })).toBeVisible()
})
