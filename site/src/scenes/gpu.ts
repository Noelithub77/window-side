// GPU passthrough: Windows draws on the real card, and Looking Glass copies each frame into a Linux window.
import { gsap } from "gsap";
import { appWindow, box, chip, laptop, panes } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, stream } from "./motion.ts";

export const build: Build = (svg, narrow) => {
  const L = narrow
    ? { w: 420, h: 900, vm: [210, 100] as Point, vmW: 240, vmH: 150, gpu: [210, 290] as Point, shm: [210, 450] as Point, host: [210, 710] as Point, hostW: 300, text: 17 }
    : { w: 1200, h: 470, vm: [230, 150] as Point, vmW: 260, vmH: 170, gpu: [230, 350] as Point, shm: [560, 350] as Point, host: [950, 290] as Point, hostW: 330, text: 20 };
  const s = new Sketch(svg, L.w, L.h, 53);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const [vx, vy] = L.vm;
  const [gx, gy] = L.gpu;
  const [sx, sy] = L.shm;
  const [hx, hy] = L.host;
  const screenH = L.hostW * 0.62;
  const shmHalf = narrow ? 100 : 115;

  const windows = s.group(null, () => {
    appWindow(s, vx, vy, L.vmW, L.vmH, "windows vm", { stroke: ink.mint });
    panes(s, vx, vy + 12, 60);
    chip(s, gx, gy, 64, "gpu", ink.peach);
    s.text(gx + (narrow ? 62 : 0), gy + (narrow ? 6 : 72), "your real card", { size: L.text - 3, color: ink.peach, anchor: narrow ? "start" : "middle" });
  });
  const render = s.arrow([[gx, gy - 44], [vx, vy + L.vmH / 2 + 8]], { stroke: ink.peach, width: 1.8 });

  const copy = s.group(null, () => {
    box(s, sx, sy, shmHalf * 2, 58, "shared memory", { stroke: ink.peach, fill: ink.peachFill }, L.text - 1);
    s.text(sx, sy + 60, "frames copied, not streamed", { size: L.text - 3, color: ink.soft });
  });
  const toShm = s.arrow(narrow ? [[gx, gy + 46], [sx, sy - 34]] : [[gx + 46, gy], [sx - shmHalf - 8, sy]], { stroke: ink.peach, width: 1.8 });

  const linux = s.group(null, () => {
    laptop(s, hx, hy, L.hostW, { stroke: ink.lavender });
    appWindow(s, hx, hy, L.hostW * 0.72, screenH * 0.72, "looking glass", { stroke: ink.peach });
    panes(s, hx, hy + 10, 40);
    s.text(hx, hy + screenH / 2 + L.hostW * 0.09 + 34, "your linux desktop", { size: L.text, color: ink.lavender });
  });
  const toLinux = s.arrow(narrow ? [[sx, sy + 76], [hx, hy - screenH / 2 - 14]] : [[sx + shmHalf + 8, sy], [hx - L.hostW / 2 - 14, sy]], { stroke: ink.peach, width: 1.8 });

  stream(loop, s, render.guide, ink.peach, 2, 1, 5);
  stream(loop, s, toShm.guide, ink.peach, 2, 0.9, 5);
  stream(loop, s, toLinux.guide, ink.peach, 3, 1.1, 5);
  drawOn(story, windows, 0, 0.8);
  drawOn(story, render.group, ">-0.2", 0.4);
  drawOn(story, [toShm.group, copy], ">-0.1", 0.6);
  drawOn(story, [toLinux.group, linux], ">-0.1", 0.8);
  return { story, loop };
};
