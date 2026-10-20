// The name: four panes of one window, and the lighter one swings open onto Windows.
import { gsap } from "gsap";
import { panes } from "../icons.ts";
import { drawOn, ink, Sketch } from "../sketch.ts";
import type { Build } from "./motion.ts";

export const build: Build = (svg) => {
  const s = new Sketch(svg, 520, 470, 31);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const size = 124;
  const spots = [[130, 96], [266, 96], [130, 232], [266, 232]] as const;

  // Behind the fourth pane: Windows, lit from inside.
  const inside = s.group(null, () => {
    s.shape("rect", { x: 266, y: 232, width: size, height: size, rx: 34, fill: "url(#ws-glow)" });
    s.rect(274, 240, size - 16, size - 16, { stroke: ink.mint, fill: ink.mintFill, rough: 0.6 }, 26);
    panes(s, 328, 294, 46);
  });
  const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
  defs.innerHTML = '<radialGradient id="ws-glow"><stop offset="0" stop-color="#85e8c6" stop-opacity=".5"/><stop offset="1" stop-color="#85e8c6" stop-opacity="0"/></radialGradient>';
  svg.prepend(defs);

  const glass = spots.map(([x, y], i) =>
    s.shape("rect", { x, y, width: size, height: size, rx: 34, fill: "url(#ws-mint)", opacity: [1, 0.82, 0.82, 0.6][i] ?? 1, class: "packet" }),
  );
  const swing = glass[3];

  const linux = s.group(null, () => {
    s.text(92, 40, "your linux desktop", { size: 21, color: ink.lavender });
    s.arrow([[96, 52], [100, 80], [128, 110]], { stroke: ink.lavender, width: 1.8 });
  });
  const windows = s.group(null, () => {
    s.text(410, 436, "windows, right here", { size: 21, color: ink.mint });
    s.arrow([[420, 412], [414, 382], [372, 340]], { stroke: ink.mint, width: 1.8 });
  });
  const light = s.group("light", () => {
    s.path("M392 252 L512 300 M392 300 L506 352 M392 346 L496 404", { stroke: ink.peach, width: 1.6, dash: true });
  });

  gsap.set(inside, { opacity: 0 });
  story.addLabel("step0", 0);
  drawOn(story, linux, "step0", 0.8);
  story.addLabel("step1", ">");
  story.to(inside, { opacity: 1, duration: 0.3 }, "step1");
  if (swing) story.to(swing, { scaleX: 0.14, skewY: -14, svgOrigin: "390 294", duration: 1, ease: "power2.inOut" }, "step1");
  story.addLabel("step2", ">-0.2");
  drawOn(story, [windows, light], "step2", 0.8);
  story.to({}, { duration: 0.3 });

  loop.to(light, { opacity: 0.4, duration: 1.8, ease: "sine.inOut", repeat: -1, yoyo: true });
  return { story, loop };
};
