# Frontend performance

## Scope and decisions

- **[chosen]** Short fade and slight movement for page navigation, approved by Noel. Preserve the palette, glass panels, glows, layout and background animation.
- Keep the current React, Motion, Zustand and Pixi stack; add no dependencies.
- Use built Chromium fixtures for repeatable timing and isolated OmaBox for native WebKitGTK rendering. Do not touch the live desktop or start a VM.

## Baseline

`pnpm -C ui exec playwright test tests/navigation.spec.ts --workers=1` failed both navigation timing tests before changes (the rapid-switch correctness test passed).

Eight tab switches across stopped and running fixtures took 577–649 ms to mount the destination. Settings reached 95% effective opacity after 986–1031 ms; Overview after 892–933 ms. Timing starts at the browser's click handler and samples animation frames; it excludes Playwright locator overhead.

The built entry JavaScript was 1436.70 kB (420.27 kB gzip). Its eager dependency graph included all screens, Pixi, charts and the setup celebration renderer; the bundled Lottie WebAssembly asset was 1238.07 kB. These are asset sizes, not measured startup delays.

## Findings and work

1. **Largest measured delay: sequential page animations.** The old `AnimatePresence mode="wait"` held the destination behind a spring exit. Pages now overlap a 120 ms exit with a 180 ms entry. Outgoing content is inert and hidden from accessibility navigation. Settings cards use a 200 ms entrance with a shorter stagger.
2. **Additional transition rendering cost: page-wide blur.** Removed the animated blur and scale from the page wrapper, preserving the glass panels, shader, colours and glows. This reduces rendering work; its individual frame-time saving was not isolated from the transition sequencing change.
3. **Unnecessary work on live updates.** App view/mood hooks previously subscribed to the whole store, so metrics events re-rendered the app and active screen. They now select derived scalar values. Settings subscribes only to displayed fields and shallow-compares the VM summary, avoiding uptime/process changes. Setup and Welcome also use focused subscriptions. Render counts were not separately profiled.
4. **Redundant backdrop drawing.** Reduced motion previously froze shader time but retained the 30 fps draw loop. It now draws on mood, palette or size changes and stops between changes. Normal motion retains the original shader, resolution and 30 fps cap. Drawing pauses when the document reports hidden and resumes when visible.
5. **Unneeded startup code.** The setup celebration and renderer now load at Review; its local WebAssembly is precompiled there for prompt playback after Apply. The entry JavaScript fell from 1436.70 to 1259.53 kB (420.27 to 384.39 kB gzip), about 12% smaller before compression. Network coverage confirms the WebAssembly is not requested through the Resources step and is requested once on Review. This is reduced startup work, not a measured cold-start speedup.

## Results

Eight switches, four per VM fixture, using the same production build and Chromium workflow as the baseline:

| Measurement | Before | After | Median reduction |
| --- | --- | --- | --- |
| Settings mounted | 609–649 ms | 20–40 ms | 94% |
| Settings readable | 986–1031 ms | 243–266 ms | 75% |
| Overview mounted | 577–638 ms | 16–32 ms | 95% |
| Overview readable | 892–933 ms | 168–191 ms | 80% |

"Readable" means the Appearance heading in Settings, or the Windows heading in Overview, reaches 95% effective opacity across its ancestors. This measures the visible transition delay, not complete pixel stability, input latency, or a field Core Web Vital. Small samples establish this local improvement, not a universal performance guarantee.

The fixture pages share a GPU. Concurrent workers distorted the timing budgets, so the browser suite now uses one worker. The 200 ms mount / 400 ms readable regression budgets remain unchanged.

## Verification

- `./ops ui:check`: TypeScript, strict type-aware oxlint and production build passed. The remaining large-chunk warning is retained.
- `./ops ui:test`: all 8 tests passed, including navigation timing with stopped/running fixtures, rapid tab changes, setup, first boot, and launch/shutdown.
- Backdrop coverage checks idle reduced-motion draw counts, resize, mood changes, live preference changes, and simulated document visibility changes.
- `./ops check`: 49 Rust tests, strict Clippy, native build, first-boot, plugin, icon-packaging and installer fixtures passed.
- `./ops gui:smoke --output /tmp/window-side-perf-native-final-20261020`: native WebKitGTK in an isolated OmaBox passed WebGL, Lottie pixels, clipboard and icon checks with no reported browser errors. Inspected that screenshot and the Chromium Settings screenshot.
- The first native smoke caught late Lottie loading after code splitting; preparing the renderer on Review fixed it without relaxing the smoke check.

The installed live app was not replaced or driven. Native navigation latency, GPU frame times, real guest activity and physical desktop smoothness remain unmeasured. No VM was started or changed.

## Remaining candidates

The entry bundle is still about 1.26 MB, with Pixi and charts eagerly included. The animated shader and 22 px glass filters remain continuous graphics work while visible. These are candidates for a later native CPU/GPU trace, not confirmed remaining causes of lag. Further shader resolution/cadence changes would need visual comparison and a design decision; this pass preserves them.

## API references

Context7's Pixi search returned adjacent projects rather than the core API, so the API review used the installed package declarations and current primary documentation:

- [Motion presence modes](https://motion.dev/docs/react-animate-presence#mode)
- [Zustand shallow selectors](https://zustand.docs.pmnd.rs/learn/guides/prevent-rerenders-with-use-shallow.html)
- [Pixi application lifecycle](https://pixijs.download/release/docs/app.Application.html)
- [React lazy loading](https://react.dev/reference/react/lazy)
