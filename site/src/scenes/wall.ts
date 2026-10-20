// Why: rebooting across the wall, keeping a second copy, or one install with two ways in.
import { gsap } from "gsap";
import { appWindow, cycle, drive, laptop, panes, terminal } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, stream, travel } from "./motion.ts";

export const build: Build = (svg, narrow) => {
  const s = new Sketch(svg, narrow ? 420 : 1200, narrow ? 1340 : 430, 11);
  const centers: readonly Point[] = narrow ? [0, 1, 2].map((i) => [210, 200 + i * 450]) : [0, 1, 2].map((i) => [200 + i * 400, 190]);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const at = (i: number): Point => centers[i] ?? [0, 0];

  const reboot = s.group(null, () => {
    const [x, y] = at(0);
    terminal(s, x - 100, y - 10, 100);
    s.text(x - 100, y + 60, "linux", { size: 17, color: ink.soft });
    s.rect(x - 14, y - 105, 28, 190, { stroke: ink.soft, fill: ink.greyFill }, 4);
    for (let h = -90; h < 80; h += 24) s.line([x - 12, y + h + 14], [x + 12, y + h], { stroke: ink.faint, width: 1.4, rough: 0.6 });
    panes(s, x + 100, y - 10, 76);
    s.text(x + 100, y + 60, "windows", { size: 17, color: ink.soft });
    cycle(s, x, y - 140, 20, ink.peach);
    s.text(x + 32, y - 132, "reboot", { size: 17, color: ink.peach, anchor: "start" });
    s.text(x, y + 160, "everything closes\non the way", { size: 21 });
  });

  const copy = s.group(null, () => {
    const [x, y] = at(1);
    laptop(s, x, y - 20, 210, { stroke: ink.lavender });
    appWindow(s, x, y - 18, 150, 100, "vm", { stroke: ink.peach });
    panes(s, x, y - 4, 40, ink.peach);
    s.text(x + 134, y - 92, "×2", { size: 28, color: ink.peach });
    s.text(x, y + 84, "apps · updates · licences", { size: 16, color: ink.soft });
    s.text(x, y + 160, "two of everything", { size: 21 });
  });

  const [cx, cy] = at(2);
  const both = s.group(null, () => {
    drive(s, cx, cy + 74, 300, 48, [{ label: "linux", color: ink.lavender, share: 0.44 }, { label: "windows", color: ink.mint, share: 0.56 }]);
    panes(s, cx - 92, cy - 92, 62);
    s.text(cx - 92, cy - 140, "boot natively", { size: 17, color: ink.mint });
    appWindow(s, cx + 88, cy - 92, 132, 92, "", { stroke: ink.lavender });
    panes(s, cx + 88, cy - 84, 34);
    s.text(cx + 88, cy - 152, "or beside linux", { size: 17, color: ink.lavender });
    s.text(cx, cy + 160, "one install,\ntwo ways in", { size: 21, color: ink.mint });
  });
  const native = s.arrow([[cx + 40, cy + 46], [cx - 30, cy + 8], [cx - 88, cy - 54]], { stroke: ink.mint, width: 1.8 });
  const vm = s.arrow([[cx + 72, cy + 46], [cx + 82, cy + 4], [cx + 88, cy - 40]], { stroke: ink.lavender, width: 1.8 });

  story.addLabel("step0", 0);
  drawOn(story, reboot, "step0", 1);
  story.to({}, { duration: 0.6 });
  story.addLabel("step1");
  story.to(reboot, { opacity: 0.22, duration: 0.4 }, "step1");
  drawOn(story, copy, "step1+=0.1", 1);
  story.to({}, { duration: 0.6 });
  story.addLabel("step2");
  story.to(copy, { opacity: 0.22, duration: 0.4 }, "step2");
  drawOn(story, both, "step2+=0.1", 1);
  drawOn(story, [native.group, vm.group], ">-0.3", 0.6);
  travel(story, s, native.guide, ink.mint, ">-0.1", 0.7);
  travel(story, s, vm.guide, ink.lavender, "<", 0.7);
  story.to({}, { duration: 0.4 });

  stream(loop, s, native.guide, ink.mint, 3, 2.6, 5);
  stream(loop, s, vm.guide, ink.lavender, 3, 2.2, 5);
  return { story, loop };
};
