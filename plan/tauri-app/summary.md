# Tauri app

The GPUI window becomes a Tauri 2 webview with a React UI. The CLI, setup, VM,
guest and desktop code stay in Rust unchanged; the GUI only calls them.

## Decisions

- **[chosen]** Tauri + React, replacing GPUI. GPUI could not clip rounded
  corners, blur, or draw SVG gradients, and its tests needed OCR and screen
  coordinates. Cost: WebKitGTK on every distro and a heavier window.
- **[chosen]** One `ws` binary. Bare `ws` opens the Tauri window; every other
  command stays the JSON CLI. The built UI is embedded in the binary.
- **[chosen]** UI in `ui/`: Vite, React 19, TypeScript, Tailwind 4, every
  shadcn component in the Luma style (Radix base), Recharts through shadcn
  charts, Motion for transitions, PixiJS for the WebGL backdrop (CSS fallback
  when WebGL is missing), dotLottie for hand-made Lottie moments, NumberFlow
  for animated numbers, Zustand for UI state.
- **[chosen]** Typed IPC with `specta` and `tauri-specta`; bindings generate
  into `ui/src/bindings.ts`. `ui/src/lib/contract.ts` is the spec until then.
- **[chosen]** Keep the existing ksni tray and zbus single instance; they are
  tested and avoid libappindicator. Closing hides the window when a tray host
  exists, otherwise quits.
- **[chosen]** Tests: Playwright against the Vite build with mocked IPC for
  interaction (roles and labels, no OCR); one OmaBox smoke run of the real
  binary for WebKitGTK rendering.
- Lint: `tsc` and strict type-checked ESLint. Generated shadcn primitives in
  `ui/src/components/ui` are vendored and not linted. No formatter.

## Ownership

- Lead agent (creative): theme, shell, every screen, custom components,
  motion, WebGL and Lottie in `ui/src`, plus the browser mock data in
  `ui/src/lib/mock.ts` (except `bindings.ts`).
- Subagent (mechanical): Tauri in Rust, commands and events, bindings,
  Rust fixtures, wiring the mock into tests, packaging and installer, `./ops`,
  tests, removal of GPUI, docs.

## Steps

1. Scaffold `ui/` with Luma and all components. Done.
2. Rust: Tauri builder, commands and events per the contract, fixture mode,
   tray and instance wiring, bindings.
3. UI: design system, welcome, setup, overview, settings, first boot.
4. Packaging: WebKitGTK runtime per distro, Node and pnpm for source builds,
   AUR, release archive.
5. Tests and `./ops`: `ui:dev`, `ui:check`, Playwright, OmaBox smoke.
6. Remove GPUI and `gpui-kit`; update README and AGENTS.md.
