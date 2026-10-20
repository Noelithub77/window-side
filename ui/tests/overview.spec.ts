import { expect, test } from "@playwright/test"

test("overview launches and cleanly shuts down the mock Windows session", async ({ page }) => {
  await page.goto("/?fixture=stopped")
  await page.getByRole("button", { name: "Launch", exact: true }).click()
  await expect(page.getByRole("button", { name: "Open", exact: true })).toBeVisible()
  await page.getByRole("button", { name: "Power", exact: true }).click()
  await page.getByRole("menuitem", { name: "Shut down", exact: true }).click()
  await expect(page.getByRole("button", { name: "Launch", exact: true })).toBeVisible()
})

test("settings opens resource setup and can cancel back", async ({ page }) => {
  await page.goto("/?fixture=stopped")
  await page.getByRole("button", { name: "Settings", exact: true }).click()
  await page.getByRole("button", { name: "Change", exact: true }).click()
  await expect(page.getByRole("heading", { name: "Choose Windows", exact: true })).toBeVisible()
  await page.getByRole("button", { name: "Cancel", exact: true }).click()
  await expect(page.getByRole("button", { name: "Change", exact: true })).toBeVisible()
})
