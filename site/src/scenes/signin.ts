// Sign-in: the password waits in the Linux keyring, is typed on the VM keyboard at start, and Windows opens ready.
import { gsap } from "gsap";
import { appWindow, check, panes } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, stream, travel } from "./motion.ts";

export const build: Build = (svg, narrow) => {
  const L = narrow
    ? { w: 420, h: 820, ring: [210, 100] as Point, keys: [210, 400] as Point, win: [210, 680] as Point, text: 17 }
    : { w: 1200, h: 360, ring: [200, 170] as Point, keys: [600, 170] as Point, win: [990, 170] as Point, text: 20 };
  const s = new Sketch(svg, L.w, L.h, 71);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const [rx, ry] = L.ring;
  const [kx, ky] = L.keys;
  const [wx, wy] = L.win;

  const ring = s.group(null, () => {
    s.rect(rx - 90, ry - 64, 180, 128, { stroke: ink.lavender, fill: ink.lavenderFill }, 20);
    // A key: bow, shaft and two teeth.
    s.circle(rx - 34, ry, 34, { stroke: ink.lavender, width: 2.2 });
    s.line([rx - 17, ry], [rx + 46, ry], { stroke: ink.lavender, width: 2.2 });
    s.line([rx + 30, ry], [rx + 30, ry + 14], { stroke: ink.lavender, width: 2.2 });
    s.line([rx + 44, ry], [rx + 44, ry + 18], { stroke: ink.lavender, width: 2.2 });
    s.text(rx, ry + 100, "linux keyring", { size: L.text, color: ink.lavender });
    s.text(rx, ry + 100 + L.text * 1.2, "saved once", { size: L.text - 3, color: ink.soft });
  });

  const keyboard = s.group(null, () => {
    s.rect(kx - 110, ky - 50, 220, 100, { stroke: ink.sky, fill: ink.skyFill }, 16);
    for (let row = 0; row < 3; row += 1) {
      const count = row === 2 ? 1 : 7;
      for (let col = 0; col < count; col += 1) {
        const w = row === 2 ? 110 : 20;
        const x = row === 2 ? kx - 55 : kx - 91 + col * 26;
        s.rect(x, ky - 36 + row * 26, w, 18, { stroke: ink.sky, width: 1.4, rough: 0.6 }, 5);
      }
    }
    s.text(kx, ky + 86, "vm keyboard", { size: L.text, color: ink.sky });
    s.text(kx, ky + 86 + L.text * 1.2, "typed when windows starts", { size: L.text - 3, color: ink.soft });
  });

  const desktop = s.group(null, () => {
    appWindow(s, wx, wy, 230, 150, "windows", { stroke: ink.mint });
    panes(s, wx, wy + 12, 52);
    s.text(wx, wy + 110, "signed in, ready", { size: L.text, color: ink.mint });
  });
  const done = s.group(null, () => check(s, wx + 96, wy - 96, 30));

  const toKeys = s.arrow(narrow ? [[rx, ry + 150], [kx, ky - 64]] : [[rx + 104, ry], [kx - 124, ky]], { stroke: ink.lavender, width: 1.8 });
  const toWin = s.arrow(narrow ? [[kx, ky + 136], [wx, wy - 88]] : [[kx + 124, ky], [wx - 128, wy]], { stroke: ink.sky, width: 1.8 });
  stream(loop, s, toKeys.guide, ink.lavender, 4, 2.2, 4);
  stream(loop, s, toWin.guide, ink.sky, 4, 2.2, 4);

  gsap.set(done, { opacity: 0, transformOrigin: "50% 50%" });
  drawOn(story, ring, 0, 0.8);
  drawOn(story, toKeys.group, ">-0.2", 0.5);
  travel(story, s, toKeys.guide, ink.lavender, "<", 0.6);
  drawOn(story, keyboard, ">-0.1", 0.8);
  drawOn(story, toWin.group, ">-0.2", 0.5);
  travel(story, s, toWin.guide, ink.sky, "<", 0.6);
  drawOn(story, desktop, ">-0.1", 0.8);
  story.fromTo(done, { opacity: 0, scale: 0.4 }, { opacity: 1, scale: 1, duration: 0.5, ease: "back.out(2)" }, ">-0.1");
  return { story, loop };
};
