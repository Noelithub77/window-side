// First boot: SATA Windows already knows, the driver CD, the switch to VirtIO, and a rollback if the agent never answers.
import { gsap } from "gsap";
import { box, check, chip, cross, disc, drive } from "../icons.ts";
import { drawOn, ink, type Point, Sketch } from "../sketch.ts";
import { type Build, choose, travel } from "./motion.ts";

export const build: Build = (svg, narrow) => {
  const s = new Sketch(svg, narrow ? 420 : 1200, narrow ? 1000 : 400, 41);
  const at: readonly Point[] = narrow ? [0, 1, 2, 3].map((i) => [230, 100 + i * 250]) : [0, 1, 2, 3].map((i) => [150 + i * 300, 150]);
  const p = (i: number): Point => at[i] ?? [0, 0];
  const story = gsap.timeline({ paused: true });
  const size = narrow ? 17 : 19;
  const caption = (i: number, text: string, color: string = ink.soft) => s.text(p(i)[0], p(i)[1] + 78, text, { size, color });

  const nodes = [
    s.group(null, () => {
      drive(s, p(0)[0], p(0)[1], 170, 46, [{ label: "sata", color: ink.peach, share: 1 }]);
      caption(0, "a disk windows knows");
    }),
    s.group(null, () => {
      disc(s, p(1)[0], p(1)[1], 40, ink.sky);
      caption(1, "you run the guest tools");
    }),
    s.group(null, () => {
      chip(s, p(2)[0], p(2)[1], 76, "virtio", ink.mint);
      caption(2, "fast virtual hardware");
    }),
    s.group(null, () => {
      box(s, p(3)[0], p(3)[1], 170, 60, "guest agent", { stroke: ink.mint }, size);
      caption(3, "answers from windows");
    }),
  ];
  const step = (i: number): Point[] => {
    const [ax, ay] = p(i);
    const [bx, by] = p(i + 1);
    return narrow ? [[ax, ay + 100], [bx, by - 56]] : [[ax + 100, ay], [bx - 100, by]];
  };
  const links = [0, 1, 2].map((i) => s.arrow(step(i), { stroke: i === 0 ? ink.peach : ink.mint, width: 1.8 }));

  const ok = s.group(null, () => {
    check(s, p(3)[0], p(3)[1] - 66, 26);
    s.text(p(3)[0], p(3)[1] + 104, "saved as done", { size: size - 2, color: ink.mint });
  });
  const fail = s.group(null, () => {
    cross(s, p(2)[0] + 48, p(2)[1] - 48, 18);
    const back: Point[] = narrow
      ? [[p(2)[0] - 70, p(2)[1]], [40, (p(0)[1] + p(2)[1]) / 2], [p(0)[0] - 100, p(0)[1] + 10]]
      : [[p(2)[0], p(2)[1] + 106], [p(1)[0], p(1)[1] + 196], [p(0)[0], p(0)[1] + 106]];
    s.arrow(back, { stroke: ink.rose, width: 1.8, dash: true });
    const [tx, ty] = narrow ? [60, (p(0)[1] + p(2)[1]) / 2 + 60] : [p(1)[0], p(1)[1] + 232];
    s.text(tx, ty, narrow ? "restore\nsata" : "restore the saved sata setup", { size: size - 1, color: ink.rose, anchor: narrow ? "start" : "middle" });
  });
  const agent = nodes[3];

  gsap.set(fail, { opacity: 0 });
  nodes.forEach((node, i) => {
    story.addLabel(`step${i}`, i ? ">-0.1" : 0);
    drawOn(story, node, `step${i}`, 0.7);
    const link = links[i];
    if (link) {
      drawOn(story, link.group, ">-0.2", 0.4);
      travel(story, s, link.guide, ink.mint, ">-0.2", 0.5);
    }
  });
  drawOn(story, [ok, fail], ">", 0.5);

  return {
    story,
    act: (value) => {
      choose({ ok: [ok], fail: [fail] }, value);
      if (agent) gsap.to(agent, { opacity: value === "fail" ? 0.3 : 1, duration: 0.45 });
    },
  };
};
