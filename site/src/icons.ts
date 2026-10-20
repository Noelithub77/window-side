// The recurring characters of every diagram, drawn around a center point.
import { ink, type Sketch, type Style } from "./sketch.ts";

/** Screen centered on (cx, cy); the keyboard base hangs below it. */
export function laptop(s: Sketch, cx: number, cy: number, w: number, style: Style = {}): SVGGElement {
  const h = w * 0.62;
  return s.group("laptop", () => {
    s.rect(cx - w / 2, cy - h / 2, w, h, { fill: ink.greyFill, ...style }, 8);
    const top = cy + h / 2 + 3;
    s.polygon([[cx - w / 2 - w * 0.09, top + w * 0.09], [cx + w / 2 + w * 0.09, top + w * 0.09], [cx + w / 2, top], [cx - w / 2, top]], { fill: ink.greyFill, ...style });
  });
}

/** A desktop window with a title bar; the title sits after the three dots. */
export function appWindow(s: Sketch, cx: number, cy: number, w: number, h: number, title: string, style: Style = {}): SVGGElement {
  const x = cx - w / 2;
  const y = cy - h / 2;
  const bar = Math.min(30, h * 0.2);
  return s.group("window", () => {
    s.rect(x, y, w, h, { fill: ink.bg, ...style }, 12);
    s.line([x + 4, y + bar], [x + w - 4, y + bar], { stroke: style.stroke ?? ink.line, width: 1.4, rough: 0.5 });
    [ink.rose, ink.peach, ink.mint].forEach((color, i) => s.shape("circle", { cx: x + 16 + i * 14, cy: y + bar / 2, r: 4, fill: color }));
    if (title) s.text(x + 62, y + bar / 2 + 6, title, { size: 16, color: ink.soft, anchor: "start" });
  });
}

/** Four rounded panes: the window-side mark, and the stand-in for Windows itself. */
export function panes(s: Sketch, cx: number, cy: number, size: number, color: string = ink.mint): SVGGElement {
  const gap = size * 0.08;
  const pane = (size - gap) / 2;
  return s.group("panes", () => {
    [0, 1, 2, 3].forEach((i) => {
      const x = cx - size / 2 + (i % 2) * (pane + gap);
      const y = cy - size / 2 + Math.floor(i / 2) * (pane + gap);
      s.rect(x, y, pane, pane, { stroke: color, fill: i === 3 ? ink.greyFill : `${color}33`, rough: 0.7 }, pane * 0.28);
    });
  });
}

/** A terminal window with a prompt, the stand-in for Linux. */
export function terminal(s: Sketch, cx: number, cy: number, w: number, color: string = ink.lavender): SVGGElement {
  const h = w * 0.66;
  return s.group("terminal", () => {
    s.rect(cx - w / 2, cy - h / 2, w, h, { stroke: color, fill: ink.bg }, 10);
    const x = cx - w / 2 + w * 0.14;
    s.polyline([[x, cy - h * 0.14], [x + w * 0.1, cy], [x, cy + h * 0.14]], { stroke: color, width: 2.6, rough: 0.5 });
    s.line([x + w * 0.16, cy + h * 0.16], [x + w * 0.36, cy + h * 0.16], { stroke: color, width: 2.6, rough: 0.5 });
  });
}

export type Partition = { label: string; color: string; share: number };

/** A drive as a long rounded bar split into labelled partitions. */
export function drive(s: Sketch, cx: number, cy: number, w: number, h: number, parts: readonly Partition[], style: Style = {}): SVGGElement {
  const x = cx - w / 2;
  return s.group("drive", () => {
    let at = x + 6;
    const inner = w - 12;
    parts.forEach((part) => {
      const pw = inner * part.share;
      s.rect(at, cy - h / 2 + 6, pw - 4, h - 12, { stroke: part.color, fill: `${part.color}24`, rough: 0.6 }, 8);
      if (part.label) s.text(at + pw / 2 - 2, cy + 6, part.label, { size: 16, color: part.color });
      at += pw;
    });
    s.rect(x, cy - h / 2, w, h, { fill: "none", ...style }, 14);
  });
}

/** A chip with pins on every side. */
export function chip(s: Sketch, cx: number, cy: number, size: number, label: string, color: string): SVGGElement {
  const half = size / 2;
  return s.group("chip", () => {
    for (const offset of [-0.25, 0, 0.25]) {
      const o = offset * size;
      s.line([cx + o, cy - half - 9], [cx + o, cy - half], { stroke: color, width: 1.6, rough: 0.4 });
      s.line([cx + o, cy + half], [cx + o, cy + half + 9], { stroke: color, width: 1.6, rough: 0.4 });
      s.line([cx - half - 9, cy + o], [cx - half, cy + o], { stroke: color, width: 1.6, rough: 0.4 });
      s.line([cx + half, cy + o], [cx + half + 9, cy + o], { stroke: color, width: 1.6, rough: 0.4 });
    }
    s.rect(cx - half, cy - half, size, size, { stroke: color, fill: ink.bg }, 8);
    s.text(cx, cy + 6, label, { size: 17, color });
  });
}

/** A circular arrow, as on a restart button. */
export function cycle(s: Sketch, cx: number, cy: number, r: number, color: string): SVGGElement {
  return s.group("cycle", () => {
    const a = -Math.PI * 0.35;
    const b = Math.PI * 1.35;
    s.path(`M${cx + r * Math.cos(a)} ${cy + r * Math.sin(a)}A${r} ${r} 0 1 1 ${cx + r * Math.cos(b)} ${cy + r * Math.sin(b)}`, { stroke: color, width: 2.6 });
    const tx = cx + r * Math.cos(a);
    const ty = cy + r * Math.sin(a);
    s.polyline([[tx - 12, ty - 6], [tx, ty], [tx - 2, ty + 13]], { stroke: color, width: 2.6, rough: 0.5 });
  });
}

export function check(s: Sketch, cx: number, cy: number, size: number, color: string = ink.mint): SVGGElement {
  return s.polyline([[cx - size * 0.45, cy], [cx - size * 0.1, cy + size * 0.35], [cx + size * 0.5, cy - size * 0.4]], { stroke: color, width: 3, rough: 0.5 });
}

/** A labeled box; text is centered unless the box is tall enough for a title row. */
export function box(s: Sketch, cx: number, cy: number, w: number, h: number, label: string, style: Style = {}, size = 20): SVGGElement {
  return s.group("box", () => {
    s.rect(cx - w / 2, cy - h / 2, w, h, style);
    const lines = label.split("\n").length;
    s.text(cx, cy + size * 0.35 - ((lines - 1) * size * 1.2) / 2, label, { size, color: style.stroke ?? ink.line });
  });
}

/** A small filled dot that travels along guides: crisp, so it reads as data, not ink. */
export function packet(s: Sketch, color: string, r = 6): SVGCircleElement {
  return s.shape("circle", { cx: 0, cy: 0, r, fill: color, opacity: 0, class: "packet" });
}
