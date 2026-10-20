// Boot profiles: the normal entry keeps both graphics chips on Linux; the GPU entry reserves the card for Windows.
import { gsap } from "gsap";
import { appWindow, box, chip, panes, terminal } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, choose, stream } from "./motion.ts";

export const build: Build = (svg, narrow) => {
  const L = narrow
    ? { w: 420, h: 760, menu: [210, 56] as Point, host: [210, 300] as Point, hostW: 250, hostH: 170, igpu: [160, 316] as Point, gpu: [262, 316] as Point, linux: [96, 620] as Point, win: [318, 620] as Point, vfio: [318, 470] as Point, text: 17 }
    : { w: 1200, h: 440, menu: [600, 52] as Point, host: [600, 270] as Point, hostW: 290, hostH: 190, igpu: [540, 286] as Point, gpu: [662, 286] as Point, linux: [190, 270] as Point, win: [1010, 270] as Point, vfio: [850, 380] as Point, text: 20 };
  const s = new Sketch(svg, L.w, L.h, 61);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const [mx, my] = L.menu;
  const menuW = narrow ? 300 : 340;

  const base = s.group(null, () => {
    s.rect(mx - menuW / 2, my - 34, menuW, 68, { stroke: ink.soft, fill: ink.greyFill }, 12);
    s.text(mx - menuW / 4, my + 7, "linux", { size: L.text, color: ink.line });
    s.text(mx + menuW / 4, my + 7, "linux · gpu", { size: L.text, color: ink.line });
    s.text(mx + menuW / 2 + 12, my + 7, narrow ? "" : "boot menu", { size: L.text - 3, color: ink.soft, anchor: "start" });
    const [hx, hy] = L.host;
    s.rect(hx - L.hostW / 2, hy - L.hostH / 2, L.hostW, L.hostH, { stroke: ink.soft, fill: ink.greyFill }, 18);
    s.text(hx, hy - L.hostH / 2 + 30, "your pc", { size: L.text - 2, color: ink.soft });
    chip(s, L.igpu[0], L.igpu[1], 60, "igpu", ink.sky);
    chip(s, L.gpu[0], L.gpu[1], 60, "gpu", ink.peach);
    terminal(s, L.linux[0], L.linux[1], narrow ? 120 : 150);
    s.text(L.linux[0], L.linux[1] - (narrow ? 58 : 70), "linux", { size: L.text, color: ink.lavender });
    appWindow(s, L.win[0], L.win[1], narrow ? 170 : 200, narrow ? 110 : 130, "windows", { stroke: ink.mint });
    panes(s, L.win[0], L.win[1] + 10, narrow ? 40 : 48);
  });

  const toLinux = (from: Point): Point[] => (narrow ? [[from[0] - 10, from[1] + 44], [L.linux[0] + 20, L.linux[1] - 50]] : [[from[0] - 40, from[1]], [L.linux[0] + 84, L.linux[1]]]);
  const pick = (left: boolean) => s.rect(mx + (left ? -menuW / 2 + 8 : 8), my - 26, menuW / 2 - 16, 52, { stroke: ink.mint, width: 2.4 }, 10);

  const normal = s.group(null, () => {
    pick(true);
    const a = s.arrow(toLinux(L.igpu), { stroke: ink.sky, width: 1.8 });
    const b = s.arrow(narrow ? [[L.gpu[0] - 30, L.gpu[1] + 44], [L.linux[0] + 50, L.linux[1] - 46]] : [[L.gpu[0] - 10, L.gpu[1] + 44], [(L.gpu[0] + L.linux[0]) / 2, L.host[1] + 150], [L.linux[0] + 60, L.linux[1] + 50]], { stroke: ink.peach, width: 1.8 });
    s.text(L.win[0], L.win[1] + (narrow ? 86 : 104), "virtual display only", { size: L.text - 3, color: ink.soft });
    stream(loop, s, a.guide, ink.sky, 2, 1.6, 5);
    stream(loop, s, b.guide, ink.peach, 2, 2, 5);
  });

  const gpu = s.group(null, () => {
    pick(false);
    const a = s.arrow(toLinux(L.igpu), { stroke: ink.sky, width: 1.8 });
    box(s, L.vfio[0], L.vfio[1], narrow ? 150 : 170, 50, "vfio-pci", { stroke: ink.peach, fill: ink.peachFill }, L.text - 1);
    const b = s.arrow(narrow ? [[L.gpu[0] + 20, L.gpu[1] + 44], [L.vfio[0], L.vfio[1] - 30]] : [[L.gpu[0] + 44, L.gpu[1] + 20], [L.vfio[0] - 90, L.vfio[1] - 6]], { stroke: ink.peach, width: 1.8 });
    const c = s.arrow(narrow ? [[L.vfio[0], L.vfio[1] + 30], [L.win[0], L.win[1] - 60]] : [[L.vfio[0] + 90, L.vfio[1] - 6], [L.win[0] - 30, L.win[1] + 70]], { stroke: ink.peach, width: 1.8 });
    s.text(L.vfio[0], L.vfio[1] + (narrow ? -40 : 52), "held from boot", { size: L.text - 3, color: ink.peach });
    stream(loop, s, a.guide, ink.sky, 2, 1.6, 5);
    stream(loop, s, b.guide, ink.peach, 2, 1.2, 5);
    stream(loop, s, c.guide, ink.peach, 2, 1.2, 5);
  });

  const groups = { normal: [normal], gpu: [gpu] };
  gsap.set(gpu, { opacity: 0 });
  story.addLabel("step0", 0);
  drawOn(story, base, "step0", 0.9);
  drawOn(story, [normal, gpu], ">-0.2", 0.8);
  return { story, loop, act: (value) => choose(groups, value) };
};
