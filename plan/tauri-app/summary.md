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
- Lint: `tsc` and strict type-aware oxlint (`--deny-warnings`). Generated shadcn primitives in
  `ui/src/components/ui` are vendored and not linted. No formatter.

## Steps

1. Scaffold `ui/` with Luma and all components. Done.
2. Rust: implemented Tauri builder, contract commands and events, isolated
   in-memory fixtures, ksni tray and zbus instance wiring. Bindings generate
   in a Rust test; stale output fails `./ops ui:bindings:check`. Native build, binding drift check, 40 Rust tests and strict Clippy passed.
   WebKitGTK clipboard writes use the official clipboard-manager fallback.
   Tauri is pinned to stable 2.12.4; Specta 2 and tauri-specta 2 are pinned
   to matching rc.25 releases because v2 has no stable release yet.
3. UI: design system, welcome, setup, overview, settings, first boot.
4. Packaging: implemented WebKitGTK 4.1 runtime/bootstrap and development
   package commands for Arch, Debian/Ubuntu and Fedora. Source installs and
   the Arch recipe build React before Cargo; release archives include the
   runtime dependency helper. Real package transactions remain unverified.
5. Tests and `./ops`: implemented `ui:dev`, `ui:check`, `ui:bindings`,
   `ui:bindings:check`, `ui:test` and `gui:smoke`. Chromium covers setup/apply/first boot, overview launch/shutdown and settings
   navigation. Native OmaBox smoke passed WebGL, Lottie pixels and clipboard
   with no browser errors. Tray tests passed registration, close/reopen, quit
   without a tray host and second-launch single-instance behavior.
   `./ops check` passed including package and installer fixtures.
6. Removed GPUI views and `gpui-kit`; retained tray and instance logic under
   `src/app/`. Updated README and AGENTS.md for the Rust/React split and
   oxlint workflow.
