// Display: headless through the guest agent, a console window, or a passed-through GPU shown by Looking Glass.
import { gsap } from "gsap";
import { appWindow, box, chip, laptop, panes } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, choose, stream } from "./motion.ts";

export const build: Build = (svg, narrow) => {
  const L = narrow
    ? { w: 420, h: 860, vm: [210, 110] as Point, vmW: 240, vmH: 160, host: [210, 680] as Point, hostW: 300, gpu: [92, 300] as Point, shm: [250, 430] as Point, text: 17 }
    : { w: 1200, h: 460, vm: [240, 200] as Point, vmW: 270, vmH: 180, host: [940, 200] as Point, hostW: 320, gpu: [240, 380] as Point, shm: [600, 360] as Point, text: 20 };
  const s = new Sketch(svg, L.w, L.h, 53);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const [vx, vy] = L.vm;
  const [hx, hy] = L.host;
  const screenH = L.hostW * 0.62;

  const base = s.group(null, () => {
    appWindow(s, vx, vy, L.vmW, L.vmH, "windows vm", { stroke: ink.mint });
    panes(s, vx, vy + 12, 66);
    laptop(s, hx, hy, L.hostW, { stroke: ink.lavender });
    s.text(hx, hy + screenH / 2 + L.hostW * 0.09 + 34, "your linux desktop", { size: L.text, color: ink.lavender });
  });

  // The route from the VM to Linux for the console and headless modes.
  const route: Point[] = narrow ? [[vx, vy + L.vmH / 2 + 8], [hx, hy - screenH / 2 - 12]] : [[vx + L.vmW / 2 + 8, vy - 10], [(vx + hx) / 2, vy - 60], [hx - L.hostW / 2 - 12, hy - 10]];
  const label: Point = narrow ? [vx + 16, (vy + hy) / 2] : [(vx + hx) / 2, vy - 82];
  const anchor = narrow ? "start" : "middle";

  const headless = s.group(null, () => {
    const path = s.arrow(route, { stroke: ink.sky, width: 1.8 }, "both");
    s.text(label[0], label[1], "guest agent", { size: L.text, color: ink.sky, anchor });
    s.text(hx, hy - 14, "{ \"ok\": true }", { size: L.text + 2, color: ink.sky });
    s.text(hx, hy + 22, "no window, just answers", { size: L.text - 3, color: ink.soft });
    stream(loop, s, path.guide, ink.sky, 3, 1.8, 5);
  });

  const consoleView = s.group(null, () => {
    const path = s.arrow(route, { stroke: ink.lavender, width: 1.8 });
    s.text(label[0], label[1], "virtual display", { size: L.text, color: ink.lavender, anchor });
    appWindow(s, hx, hy, L.hostW * 0.72, screenH * 0.72, "console", { stroke: ink.lavender });
    panes(s, hx, hy + 10, 40);
    stream(loop, s, path.guide, ink.lavender, 3, 2.2, 5);
  });

  const glass = s.group(null, () => {
    chip(s, L.gpu[0], L.gpu[1], 64, "gpu", ink.peach);
    s.text(L.gpu[0] + (narrow ? 0 : 70), L.gpu[1] + (narrow ? 66 : 6), "draws windows", { size: L.text - 3, color: ink.peach, anchor: narrow ? "middle" : "start" });
    box(s, L.shm[0], L.shm[1], narrow ? 200 : 220, 58, "shared memory", { stroke: ink.peach, fill: ink.peachFill }, L.text - 1);
    const shmEdge = narrow ? 100 : 110;
    const toShm = s.arrow(narrow ? [[L.gpu[0] + 20, L.gpu[1] + 84], [L.shm[0] - shmEdge + 10, L.shm[1] - 10]] : [[L.gpu[0] + 44, L.gpu[1] - 22], [L.shm[0] - shmEdge - 6, L.shm[1]]], { stroke: ink.peach, width: 1.8 });
    const toLinux = s.arrow(narrow ? [[L.shm[0], L.shm[1] + 34], [hx + 20, hy - screenH / 2 - 12]] : [[L.shm[0] + shmEdge + 6, L.shm[1]], [hx - L.hostW / 2 + 30, hy + screenH / 2 - 10]], { stroke: ink.peach, width: 1.8 });
    appWindow(s, hx, hy, L.hostW * 0.72, screenH * 0.72, "looking glass", { stroke: ink.peach });
    panes(s, hx, hy + 10, 40);
    s.text(L.shm[0], L.shm[1] + (narrow ? -42 : 58), "frames copied, not streamed", { size: L.text - 3, color: ink.soft });
    stream(loop, s, toShm.guide, ink.peach, 2, 0.9, 5);
    stream(loop, s, toLinux.guide, ink.peach, 3, 1.1, 5);
  });

  const groups = { headless: [headless], console: [consoleView], glass: [glass] };
  gsap.set([headless, consoleView], { opacity: 0 });
  story.addLabel("step0", 0);
  drawOn(story, base, "step0", 0.9);
  drawOn(story, [headless, consoleView, glass], ">-0.2", 0.9);
  return { story, loop, act: (value) => choose(groups, value) };
};
