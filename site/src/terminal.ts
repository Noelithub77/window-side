// The agents section: a terminal that types real `ws` commands, and a prompt that cycles what you could ask.
import { gsap } from "gsap";

export type Transcript = { id: string; title: string; command: string; output: string; durationMs: number };

const escape = (text: string) => text.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");

/** Colors JSON keys, strings, numbers and literals; anything else stays plain. */
export function highlight(line: string): string {
  return escape(line).replace(/("(?:[^"\\]|\\.)*")(\s*:)?|(-?\b\d+(?:\.\d+)?\b)|\b(true|false|null)\b/g, (match, string?: string, colon?: string, number?: string, literal?: string) => {
    if (string) return colon ? `<span class="term-key">${string}</span>${colon}` : `<span class="term-str">${string}</span>`;
    if (number ?? literal) return `<span class="term-num">${number ?? literal}</span>`;
    return match;
  });
}

/** Runs `timeline` only while `element` is on screen. */
function whileVisible(element: Element, timeline: () => gsap.core.Timeline | null) {
  new IntersectionObserver(([entry]) => {
    const current = timeline();
    if (entry?.isIntersecting) current?.resume();
    else current?.pause();
  }, { threshold: 0.2 }).observe(element);
}

/** Types a string into `write` over `seconds`, as part of a timeline. */
function typeInto(timeline: gsap.core.Timeline, text: string, seconds: number, write: (value: string) => void, position?: gsap.Position) {
  const state = { c: 0 };
  timeline.to(state, { c: text.length, duration: seconds, ease: "none", onUpdate: () => write(text.slice(0, Math.round(state.c))) }, position);
}

/** Plays transcripts in order while visible; tabs jump to one. */
export function startTerminal(body: HTMLElement, tabs: HTMLElement, transcripts: readonly Transcript[], calm: boolean) {
  let timeline: gsap.core.Timeline | null = null;
  const show = (entry: Transcript) => {
    body.innerHTML = `<span class="term-cmd">$ ${escape(entry.command)}</span>\n${entry.output.split("\n").map(highlight).join("\n")}`;
  };

  const play = (index: number) => {
    const entry = transcripts[index];
    if (!entry) return;
    buttons.forEach((button, j) => button.setAttribute("aria-selected", String(j === index)));
    timeline?.kill();
    if (calm) {
      show(entry);
      return;
    }
    const command = document.createElement("span");
    command.className = "term-cmd";
    body.replaceChildren(command);
    timeline = gsap.timeline({ onComplete: () => play((index + 1) % transcripts.length) });
    typeInto(timeline, entry.command, entry.command.length * 0.045, (value) => (command.textContent = `$ ${value}`));
    timeline.to({}, { duration: Math.min(1.1, Math.max(0.26, entry.durationMs / 1000)) });
    const lines = entry.output.split("\n");
    const gap = lines.length > 12 ? 0.02 : 0.035;
    for (const line of lines) timeline.call(() => body.insertAdjacentHTML("beforeend", `\n${highlight(line)}`), [], `+=${gap}`);
    timeline.to({}, { duration: 3.6 });
    if (!started) timeline.pause();
  };

  const buttons = transcripts.map((entry, i) => {
    const button = document.createElement("button");
    button.type = "button";
    button.setAttribute("role", "tab");
    button.textContent = entry.title;
    button.addEventListener("click", () => play(i));
    tabs.append(button);
    return button;
  });

  let started = false;
  const first = transcripts[0];
  if (first) show(first);
  buttons[0]?.setAttribute("aria-selected", "true");
  new IntersectionObserver(([entry]) => {
    if (!entry?.isIntersecting || started) return;
    started = true;
    play(0);
  }, { threshold: 0.35 }).observe(body);
  whileVisible(body, () => timeline);
}

// Each ask, and the `ws` commands an agent would run for it.
const asks = [
  { ask: "Build the Windows release of this app and run its tests.", plan: ["ws start", "ws copy app.zip 'C:\\build\\app.zip'", "ws exec --json '.\\build.ps1 -Test'"] },
  { ask: "Does the installer still work on Windows 11? Check the logs.", plan: ["ws copy setup.exe 'C:\\Users\\Public\\setup.exe'", "ws exec --json --file install-check.ps1", "ws ssh --get 'C:/Logs/setup.log' ."] },
  { ask: "Which Windows updates are waiting? Don’t install them yet.", plan: ["ws updates check", "ws updates status"] },
  { ask: "Copy the build over and time how long it takes to start.", plan: ["ws copy app.exe 'C:\\build\\app.exe'", "ws exec --json 'Measure-Command { C:\\build\\app.exe --smoke }'", "ws metrics"] },
  { ask: "Set up window-side on this laptop. Show me the plan first.", plan: ["ws setup detect", "ws setup plan", "then you approve: ws setup apply"] },
];

const steps = (plan: readonly string[]) =>
  plan.map((line) => {
    const item = document.createElement("li");
    item.textContent = line;
    return item;
  });

/** Types each ask and shows its plan, holds, and erases, forever while visible. */
export function startPrompt(element: HTMLElement, list: HTMLElement, calm: boolean) {
  if (calm) return;
  const write = (value: string) => (element.textContent = value);
  const timeline = gsap.timeline({ repeat: -1, paused: true });
  // The page starts on the first ask, so each step erases one ask and types the next.
  asks.forEach(({ ask }, i) => {
    const next = asks[(i + 1) % asks.length] ?? asks[0];
    if (!next) return;
    const erase = { c: ask.length };
    timeline.to({}, { duration: 3.4 });
    timeline.to(list, { opacity: 0, duration: 0.25 });
    timeline.to(erase, { c: 0, duration: ask.length * 0.008, ease: "none", onUpdate: () => write(ask.slice(0, Math.round(erase.c))) });
    timeline.call(() => list.replaceChildren(...steps(next.plan)));
    typeInto(timeline, next.ask, next.ask.length * 0.045, write);
    timeline.to(list, { opacity: 1, duration: 0.4 });
  });
  whileVisible(element, () => timeline);
}
