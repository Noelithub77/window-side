import { defineConfig } from "cf/config";

// A static Worker: the Vite build is the whole site, served from window-side.allpyq.in.
export default defineConfig({
  worker: {
    name: "window-side",
    compatibilityDate: "2026-10-06",
    domains: ["window-side.allpyq.in"],
    observability: { enabled: true },
  },
});
