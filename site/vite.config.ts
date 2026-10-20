import { cloudflare } from "@cloudflare/vite-plugin";
import { readFileSync } from "node:fs";
import { defineConfig, type Plugin } from "vite";

// The app icon is the single source for the mark; `<!--mark:CLASS-->` inlines it so CSS can move its panes.
const inlineMark = (): Plugin => ({
  name: "inline-mark",
  transformIndexHtml(html) {
    const svg = readFileSync(new URL("./src/mark.svg", import.meta.url), "utf8").trim();
    return html.replaceAll(/<!--mark:([\w-]+)-->/g, (_, className: string) => svg.replace("<svg ", `<svg class="mark ${className}" aria-hidden="true" `));
  },
});

export default defineConfig({
  plugins: [inlineMark(), cloudflare()],
  server: { port: 4310, strictPort: true },
  preview: { port: 4311, strictPort: true },
  build: { target: "es2022", modulePreload: { polyfill: false }, assetsInlineLimit: 0 },
});
