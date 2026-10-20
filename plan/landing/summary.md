# Landing page

A static page in `site/` that explains window-side the way the eko and
bunkialo pages explain theirs: one idea per section, hand-drawn scenes, real
recordings, and a light WebGL hero.

## Decisions

- **[chosen]** Hosted at window-side.allpyq.in as a static Worker, set up with
  `cf` autoconfig: `cloudflare.config.ts`, `@cloudflare/vite-plugin`, then
  `cf deploy --prebuilt` (`./ops site:deploy`).
- **[chosen]** eko stack: Vite + TypeScript, rough.js for Excalidraw-style
  scenes, GSAP + Lenis for motion. No framework. The hero is raw WebGL (ogl
  tree-shook poorly at 15 KB gzipped; raw is 2.4 KB). Pixi was rejected for weight.
- **[chosen]** Recordings by GPT-6-Luna children: browser fixtures
  (`site/scripts/record-app.mjs`), OmaBox native captures
  (`site/scripts/record-native.sh`) and CLI transcripts (`site/src/data/cli.json`).
  Real Windows through Looking Glass is captured last, because the VM is
  memory-heavy.
- **[chosen]** Nunito (variable, Latin subset) for all text, Excalifont for
  scenes, mint and lavender pastels on ink.
- **[chosen]** Lead with what people use: the app, GPU passthrough, automatic
  sign-in and SSH for agents. Install mechanics (disk sharing, first boot,
  boot profiles) live in the docs, not on the page.

## Loading

`index.html` holds all copy and the critical CSS, so the hero paints before
scripts. `main.ts` wires the copy button and lazy video; `enhance.ts` loads on
idle with motion, scenes and the shader. Reduced motion shows final frames.

## Sections

Why (dual boot is a wall) · the app (overview and setup recordings, settings,
tray, Omarchy panel) · GPU passthrough · sign-in · SSH and agents · your Linux · get.

## Status

Deployed. Desktop (1440) and phone (390) layouts checked in headless Chromium.
Still to do: replace the derived `metrics`, `exec`, `ssh` and `updates`
transcripts with real output, and capture real Windows through
Looking Glass, both from a running VM.
