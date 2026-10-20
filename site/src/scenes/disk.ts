// How it works: the same Windows drive boots natively or starts in KVM; a shared drive goes through a joined virtual disk.
import { gsap } from "gsap";
import { appWindow, box, drive, panes, type Partition } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, choose, stream } from "./motion.ts";

/** Center of partition `index` in a drive drawn by `drive()`. */
function partitionCenter(x: number, w: number, parts: readonly Partition[], index: number): number {
  const inner = w - 12;
  const before = parts.slice(0, index).reduce((sum, part) => sum + part.share, 0);
  return x - w / 2 + 6 + (before + (parts[index]?.share ?? 0) / 2) * inner;
}

export const build: Build = (svg, narrow) => {
  const L = narrow
    ? { w: 420, h: 520, nx: 105, vx: 315, top: 92, boxY: 250, driveY: 440, boxW: 186, boxH: 60, winW: 172, winH: 104, own: [110, 310], ownW: 186, sharedW: 400, text: 17 }
    : { w: 1200, h: 520, nx: 250, vx: 950, top: 100, boxY: 250, driveY: 440, boxW: 250, boxH: 64, winW: 210, winH: 124, own: [430, 780], ownW: 300, sharedW: 660, text: 20 };
  const s = new Sketch(svg, L.w, L.h, 17);
  const story = gsap.timeline({ paused: true });
  const loop = gsap.timeline({ paused: true });
  const mid = L.w / 2;
  const vdiskY = (L.boxY + L.boxH / 2 + L.driveY - 25) / 2;

  const doors = s.group(null, () => {
    s.text(L.nx, L.top - 62, "native boot", { size: L.text, color: ink.mint });
    panes(s, L.nx, L.top, 70);
    box(s, L.nx, L.boxY, L.boxW, L.boxH, "boot loader", { stroke: ink.soft }, L.text);
    s.text(L.vx, L.top - 82, "beside linux", { size: L.text, color: ink.lavender });
    appWindow(s, L.vx, L.top, L.winW, L.winH, "kvm", { stroke: ink.lavender });
    panes(s, L.vx, L.top + 10, 44);
    box(s, L.vx, L.boxY, L.boxW, L.boxH, "kvm + qemu", { stroke: ink.lavender }, L.text);
  });
  const up = (x: number): Point => [x, L.boxY - L.boxH / 2 - 4];
  const links = [s.arrow([up(L.nx), [L.nx, L.top + 46]], { stroke: ink.mint, width: 1.8 }), s.arrow([up(L.vx), [L.vx, L.top + L.winH / 2 + 4]], { stroke: ink.lavender, width: 1.8 })];
  const under = (x: number): Point => [x, L.boxY + L.boxH / 2 + 4];

  // Its own drive: the VM gets the whole Windows drive.
  const ownLeft = L.own[0] ?? 0;
  const ownRight = L.own[1] ?? 0;
  const own = s.group(null, () => {
    drive(s, ownLeft, L.driveY, L.ownW, 50, [{ label: "linux", color: ink.lavender, share: 1 }], { stroke: ink.faint });
    drive(s, ownRight, L.driveY, L.ownW, 50, [{ label: "windows", color: ink.mint, share: 1 }], { stroke: ink.faint });
    s.text(ownRight, L.driveY + 56, "the whole drive", { size: L.text - 2, color: ink.mint });
    const boot = s.arrow([[ownRight - 30, L.driveY - 28], [(ownRight + L.nx) / 2, L.driveY - 70], under(L.nx + 30)], { stroke: ink.mint, width: 1.8 });
    const vm = s.arrow([[ownRight + 30, L.driveY - 28], [(ownRight + L.vx) / 2 + 14, (L.driveY + L.boxY) / 2 + 10], under(L.vx)], { stroke: ink.lavender, width: 1.8 });
    stream(loop, s, boot.guide, ink.mint, 3, 2.6, 5);
    stream(loop, s, vm.guide, ink.lavender, 3, 2.2, 5);
  });

  // Shared with Linux: a small boot copy joined to the Windows partition, so Linux partitions stay out of reach.
  const parts: Partition[] = [{ label: "efi", color: ink.peach, share: 0.14 }, { label: "linux", color: ink.lavender, share: 0.44 }, { label: "windows", color: ink.mint, share: 0.42 }];
  const windowsX = partitionCenter(mid, L.sharedW, parts, 2);
  const linuxX = partitionCenter(mid, L.sharedW, parts, 1);
  const shared = s.group(null, () => {
    drive(s, mid, L.driveY, L.sharedW, 50, parts, { stroke: ink.faint });
    s.text(linuxX, L.driveY + 56, "never shown to the vm", { size: L.text - 2, color: ink.lavender });
    box(s, L.vx, vdiskY, narrow ? 150 : 210, 48, narrow ? "boot copy + win" : "boot copy + windows", { stroke: ink.mint, fill: ink.mintFill }, L.text - 3);
    const boot = s.arrow([[windowsX - 20, L.driveY - 28], [(windowsX + L.nx) / 2, L.driveY - 76], under(L.nx + 30)], { stroke: ink.mint, width: 1.8 });
    const join = s.arrow([[windowsX + 14, L.driveY - 28], [L.vx - (narrow ? 20 : 40), vdiskY + 28]], { stroke: ink.mint, width: 1.8 });
    const vm = s.arrow([[L.vx, vdiskY - 26], under(L.vx)], { stroke: ink.lavender, width: 1.8 });
    stream(loop, s, boot.guide, ink.mint, 3, 2.6, 5);
    stream(loop, s, join.guide, ink.mint, 2, 1.4, 5);
    stream(loop, s, vm.guide, ink.lavender, 2, 1, 5);
  });

  const groups = { own: [own], shared: [shared] };
  gsap.set(shared, { opacity: 0 });
  story.addLabel("step0", 0);
  drawOn(story, doors, "step0", 0.9);
  drawOn(story, links.map((link) => link.group), ">-0.2", 0.5);
  drawOn(story, [own, shared], ">-0.1", 0.9);
  return { story, loop, act: (value) => choose(groups, value) };
};
