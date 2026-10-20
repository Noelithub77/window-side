import "./style.css";

const calm = matchMedia("(prefers-reduced-motion: reduce)").matches;

// Copy the install command without the prompt sign.
document.addEventListener("click", (event) => {
  const button = event.target instanceof Element ? event.target.closest<HTMLButtonElement>("[data-copy]") : null;
  const command = button?.parentElement?.querySelector("[data-command]")?.textContent.replace(/^\$\s*/, "").trim();
  if (!button || !command) return;
  const label = button.querySelector("[data-copy-label]");
  const done = (text: string) => {
    if (label) label.textContent = text;
    button.classList.toggle("is-done", text === "Copied");
    window.setTimeout(() => {
      if (label) label.textContent = "Copy";
      button.classList.remove("is-done");
    }, 1800);
  };
  navigator.clipboard.writeText(command).then(() => done("Copied"), () => done("Select it"));
});

// Recordings load only near the screen and pause off it; reduced motion waits for a tap.
const videos = [...document.querySelectorAll<HTMLVideoElement>("video[data-lazy]")];
const load = (video: HTMLVideoElement) => {
  if (video.dataset.loaded) return;
  video.dataset.loaded = "1";
  for (const source of video.querySelectorAll<HTMLSourceElement>("source[data-src]")) source.src = source.dataset.src ?? "";
  video.load();
};
const watch = new IntersectionObserver(
  (entries) => {
    for (const { target, isIntersecting } of entries) {
      if (!(target instanceof HTMLVideoElement)) continue;
      if (isIntersecting) {
        load(target);
        if (!calm) void target.play().catch(() => undefined);
      } else target.pause();
    }
  },
  { rootMargin: "200px 0px" },
);
for (const video of videos) {
  watch.observe(video);
  if (calm) {
    video.controls = true;
    video.loop = false;
  }
}

// The app tabs swap which recording plays in the one frame.
document.addEventListener("click", (event) => {
  const tab = event.target instanceof Element ? event.target.closest<HTMLButtonElement>("[data-video]") : null;
  const name = tab?.dataset.video;
  const video = document.querySelector<HTMLVideoElement>("[data-frame] video");
  if (!tab || !name || !video) return;
  for (const other of document.querySelectorAll("[data-video]")) other.setAttribute("aria-selected", String(other === tab));
  video.poster = `/media/app/${name}.webp`;
  video.parentElement?.style.setProperty("--focus", tab.dataset.focus ?? "50% 0");
  for (const source of video.querySelectorAll<HTMLSourceElement>("source[data-src]")) {
    source.dataset.src = source.dataset.src?.replace(/\/media\/app\/[a-z-]+\./, `/media/app/${name}.`);
    if (video.dataset.loaded) source.src = source.dataset.src ?? "";
  }
  if (!video.dataset.loaded) return;
  video.load();
  if (!calm) void video.play().catch(() => undefined);
});

// Let the hero paint before requesting the motion libraries.
const idle = (callback: () => void, timeout: number) =>
  "requestIdleCallback" in window ? requestIdleCallback(callback, { timeout }) : setTimeout(callback, 120);
requestAnimationFrame(() => idle(() => void import("./enhance.ts"), 1800));
