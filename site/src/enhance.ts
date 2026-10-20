// Motion layer, loaded after first paint: smooth scroll, scene mounting, and the controls under each scene.
import { gsap } from "gsap";
import { DrawSVGPlugin } from "gsap/DrawSVGPlugin";
import { ScrollTrigger } from "gsap/ScrollTrigger";
import { SplitText } from "gsap/SplitText";
import Lenis from "lenis";
import { build as boot } from "./scenes/boot.ts";
import { build as disk } from "./scenes/disk.ts";
import { build as display } from "./scenes/display.ts";
import type { Build, Scene } from "./scenes/motion.ts";
import { build as name } from "./scenes/name.ts";
import { build as profile } from "./scenes/profile.ts";
import { build as wall } from "./scenes/wall.ts";
import { startPrompt, startTerminal, type Transcript } from "./terminal.ts";

gsap.registerPlugin(ScrollTrigger, SplitText, DrawSVGPlugin);

const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;
const scenes = new Map<string, Scene>();
// Every scene is built before ScrollTrigger measures, so pins and starts see final heights.
const builders: Record<string, Build> = { wall, name, disk, boot, display, profile };
// Story scenes follow the scroll; the rest play once they come into view.
const scrubbed = new Set(["wall", "name"]);
// The value each control starts on, so a rebuilt scene matches its buttons.
const chosen = new Map<string, string>();

if (!calm) {
  const lenis = new Lenis({ autoRaf: false, anchors: true });
  lenis.on("scroll", () => ScrollTrigger.update());
  gsap.ticker.add((time) => lenis.raf(time * 1000));
  gsap.ticker.lagSmoothing(0);
}

// Header state and reading progress.
const bar = document.querySelector(".bar");
const progress = document.querySelector(".bar-progress");
ScrollTrigger.create({
  start: 0,
  end: "max",
  onUpdate: (self) => {
    bar?.classList.toggle("is-scrolled", self.scroll() > 8);
    document.body.classList.toggle("is-scrolled-past", self.scroll() > 80);
    if (progress) gsap.set(progress, { scaleX: self.progress });
  },
});

if (!calm) {
  // Only hide what is still below the fold, so nothing visible blinks out.
  const below = gsap.utils.toArray<HTMLElement>("[data-reveal]").filter((element) => element.getBoundingClientRect().top > innerHeight);
  gsap.set(below, { opacity: 0, y: 28 });
  ScrollTrigger.batch(below, { start: "top 88%", once: true, onEnter: (batch) => gsap.to(batch, { opacity: 1, y: 0, duration: 0.9, ease: "power3.out", stagger: 0.08 }) });
  for (const heading of document.querySelectorAll(".section-head h2, .get h2")) {
    // A resize re-splits the lines; once a heading has risen it stays put.
    let reveal: gsap.core.Tween | null = null;
    let risen = false;
    SplitText.create(heading, {
      type: "lines",
      mask: "lines",
      autoSplit: true,
      onSplit: (self) => {
        reveal?.scrollTrigger?.kill();
        reveal?.kill();
        if (risen) return;
        reveal = gsap.from(self.lines, { yPercent: 105, duration: 1, stagger: 0.09, ease: "power4.out", onStart: () => (risen = true), scrollTrigger: { trigger: heading, start: "top 88%", once: true } });
      },
    });
  }
}

// Hero: light through the mark's panes.
const canvas = document.querySelector<HTMLCanvasElement>("[data-light]");
const hero = document.querySelector<HTMLElement>(".hero");
const heroMark = document.querySelector(".hero-mark");
if (!calm && canvas && hero && heroMark) void import("./shader.ts").then(({ startLight }) => startLight(canvas, hero, heroMark));

function markSteps(key: string, story: gsap.core.Timeline) {
  const steps = document.querySelectorAll(`[data-story="${key}"] li`);
  if (!steps.length) return;
  let active = 0;
  for (const [label, time] of Object.entries(story.labels)) {
    if (label.startsWith("step") && story.time() >= time - 0.01) active = Number(label.slice(4));
  }
  steps.forEach((step, i) => step.classList.toggle("is-active", i === active));
}

/** Builds one scene and drives it: scrubbed, pinned, or played on view. */
function mount(element: Element, narrow: boolean, pin: boolean) {
  const key = element.getAttribute("data-scene") ?? "";
  const svg = element instanceof SVGSVGElement ? element : element.querySelector("svg");
  const build = builders[key];
  if (!svg || !build) return;
  const scene = build(svg, narrow);
  scenes.set(key, scene);
  const value = chosen.get(key);
  if (value) scene.act?.(value);
  let inView = false;
  let finished = false;

  const sync = () => {
    const live = inView && finished && !calm;
    svg.classList.toggle("is-live", live);
    if (live) scene.loop?.play();
    else scene.loop?.pause();
  };
  const scrub = (amount: number) => {
    scene.story.progress(amount);
    finished = amount > 0.98;
    markSteps(key, scene.story);
    sync();
  };

  ScrollTrigger.create({
    trigger: element,
    start: "top bottom",
    end: "bottom top",
    onToggle: (self) => {
      inView = self.isActive;
      sync();
    },
  });
  if (calm) {
    scene.story.progress(1);
    markSteps(key, scene.story);
    return;
  }
  const section = element.closest("section");
  if (!scrubbed.has(key)) {
    scene.story.eventCallback("onComplete", () => {
      finished = true;
      sync();
    });
    ScrollTrigger.create({ trigger: element, start: "top 75%", once: true, onEnter: () => void scene.story.play() });
  } else if (pin && section) {
    ScrollTrigger.create({ trigger: section.querySelector(".wrap") ?? section, start: "center center", end: "+=160%", pin: true, onUpdate: (self) => scrub(self.progress) });
  } else {
    ScrollTrigger.create({ trigger: element, start: "top 85%", end: key === "name" ? "center 40%" : "bottom 60%", onUpdate: (self) => scrub(self.progress) });
  }
}

for (const tab of document.querySelectorAll<HTMLElement>('[data-act][aria-selected="true"]')) {
  if (tab.dataset.act && tab.dataset.value) chosen.set(tab.dataset.act, tab.dataset.value);
}

const media = gsap.matchMedia();
media.add({ narrow: "(max-width: 760px)", tall: "(min-height: 700px)" }, (context) => {
  const narrow = context.conditions?.narrow ?? false;
  const tall = context.conditions?.tall ?? false;
  scenes.clear();
  for (const element of document.querySelectorAll("[data-scene]")) {
    mount(element, narrow, !narrow && tall && Boolean(element.closest("[data-pin-section]")));
  }
});

// Controls under the scenes: tabs keep one selected, buttons just act.
document.addEventListener("click", (event) => {
  const control = event.target instanceof Element ? event.target.closest<HTMLElement>("[data-act]") : null;
  const key = control?.dataset.act;
  const value = control?.dataset.value;
  if (!control || !key || !value) return;
  if (control.getAttribute("role") === "tab") {
    for (const tab of document.querySelectorAll(`[data-act="${key}"]`)) tab.setAttribute("aria-selected", String(tab === control));
    chosen.set(key, value);
  }
  scenes.get(key)?.act?.(value);
});

// A soft spotlight follows the pointer across cards.
document.addEventListener(
  "pointermove",
  (event) => {
    const card = event.target instanceof Element ? event.target.closest<HTMLElement>(".trio article") : null;
    if (!card) return;
    const box = card.getBoundingClientRect();
    card.style.setProperty("--mx", `${event.clientX - box.left}px`);
    card.style.setProperty("--my", `${event.clientY - box.top}px`);
  },
  { passive: true },
);

// Agents: the terminal plays real transcripts, the prompt cycles what you could ask.
const term = document.querySelector<HTMLElement>("[data-term]");
const tabs = document.querySelector<HTMLElement>("[data-term-tabs]");
const prompt = document.querySelector<HTMLElement>("[data-prompt]");
const plan = document.querySelector<HTMLElement>("[data-plan]");
if (prompt && plan) startPrompt(prompt, plan, calm);
if (term && tabs) {
  void import("./data/cli.json").then(({ default: data }) => {
    const transcripts: Transcript[] = data.map(({ id, title, command, output, durationMs }) => ({ id, title, command, output, durationMs }));
    startTerminal(term, tabs, transcripts, calm);
  });
}

// The closing mark opens its pane once it comes into view.
const finalPane = document.querySelector(".get-mark .pane-4");
if (!calm && finalPane) {
  gsap.set(finalPane, { transformPerspective: 400 });
  ScrollTrigger.create({ trigger: ".get-mark", start: "top 85%", once: true, onEnter: () => void gsap.fromTo(finalPane, { rotateY: 0 }, { rotateY: -55, duration: 1.1, ease: "power3.inOut", yoyo: true, repeat: 1, repeatDelay: 0.5 }) });
}

// Type "open" anywhere: every pane swings open and light sweeps across the page.
let typed = "";
document.addEventListener("keydown", (event) => {
  if (event.metaKey || event.ctrlKey || event.altKey || event.key.length !== 1) return;
  typed = (typed + event.key.toLowerCase()).slice(-4);
  if (typed !== "open" || calm) return;
  typed = "";
  const panes = document.querySelectorAll(".mark .pane-4");
  gsap.set(panes, { transformPerspective: 400, transformOrigin: "100% 50%" });
  gsap.fromTo(panes, { rotateY: 0 }, { rotateY: -70, duration: 0.8, ease: "power3.inOut", yoyo: true, repeat: 1, repeatDelay: 0.7 });
  const sweep = document.createElement("i");
  sweep.className = "sweep";
  document.body.append(sweep);
  gsap.fromTo(sweep, { xPercent: -100, opacity: 1 }, { xPercent: 100, duration: 1.6, ease: "power2.inOut", onComplete: () => sweep.remove() });
});

void document.fonts.ready.then(() => ScrollTrigger.refresh());
