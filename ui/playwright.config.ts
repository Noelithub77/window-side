import { defineConfig } from "@playwright/test"

export default defineConfig({
  testDir: "./tests",
  outputDir: "./test-results",
  // Animated fixtures share the GPU; isolate navigation timing measurements.
  workers: 1,
  use: { baseURL: "http://127.0.0.1:51991", browserName: "chromium", viewport: { width: 1280, height: 900 }, trace: "retain-on-failure" },
  webServer: { command: "pnpm preview --host 127.0.0.1 --port 51991 --strictPort", url: "http://127.0.0.1:51991", reuseExistingServer: false },
})
