// Hero light: the mark is a window, and light through its four panes falls across the page with dust in the beam.

const vertex = /* glsl */ `
attribute vec2 position;
void main() { gl_Position = vec4(position, 0.0, 1.0); }
`;

const fragment = /* glsl */ `
precision highp float;
uniform float uTime;
uniform vec2 uMark;
uniform float uSize;
uniform vec2 uSun;
const vec3 mint = vec3(0.522, 0.910, 0.776);
const vec3 sky = vec3(0.569, 0.820, 0.953);
const vec3 lavender = vec3(0.737, 0.682, 0.976);
const vec3 peach = vec3(0.976, 0.757, 0.592);

float hash(float n) { return fract(sin(n * 91.345) * 47453.21); }
float noise(vec2 p) {
  vec2 i = floor(p); vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  float a = hash(i.x + i.y * 57.0), b = hash(i.x + 1.0 + i.y * 57.0);
  float c = hash(i.x + (i.y + 1.0) * 57.0), d = hash(i.x + 1.0 + (i.y + 1.0) * 57.0);
  return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}
float box(vec2 p, vec2 half, float r) {
  vec2 q = abs(p) - half + r;
  return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
  vec2 p = gl_FragCoord.xy;
  float unit = uSize;
  // Where the window's light lands: below the mark, pushed sideways by the sun.
  vec2 land = uMark + vec2(uSun.x * 1.6, -2.9) * unit;
  vec2 q = (p - land) / unit;
  q.x -= q.y * uSun.x * 0.9;
  q.y *= 1.75;
  q *= 0.62;
  float pane = 0.46;
  float gap = 0.06;
  float soft = 0.16 + 0.06 * abs(uSun.x);
  vec3 color = vec3(0.0);
  float alpha = 0.0;
  vec2 centers[4];
  centers[0] = vec2(-pane - gap, pane + gap) * 0.5;
  centers[1] = vec2(pane + gap, pane + gap) * 0.5;
  centers[2] = vec2(-pane - gap, -pane - gap) * 0.5;
  centers[3] = vec2(pane + gap, -pane - gap) * 0.5;
  vec3 tints[4];
  tints[0] = mint; tints[1] = sky; tints[2] = lavender; tints[3] = peach;
  // Clouds pass the window now and then.
  float shade = 0.78 + 0.22 * noise(vec2(uTime * 0.18, 3.0));
  for (int i = 0; i < 4; i++) {
    float d = box(q - centers[i], vec2(pane * 0.5), 0.13);
    float lit = smoothstep(soft, -soft, d) * (i == 3 ? 0.62 : 1.0);
    color += tints[i] * lit;
    alpha += lit;
  }
  color *= 0.11 * shade;
  alpha *= 0.11 * shade;

  // The beam between the window and the patch, faint and wide.
  vec2 ab = land - uMark;
  float t = clamp(dot(p - uMark, ab) / dot(ab, ab), 0.0, 1.0);
  float width = mix(0.55, 1.5, t) * unit;
  float beam = smoothstep(width, 0.0, distance(p, uMark + ab * t)) * (1.0 - t * 0.6) * smoothstep(0.0, 0.12, t);
  color += mix(mint, lavender, t) * beam * 0.035;
  alpha += beam * 0.035;

  // Dust drifting through the light.
  for (int i = 0; i < 26; i++) {
    float n = float(i);
    float life = fract(uTime * (0.025 + hash(n) * 0.03) + hash(n + 3.1));
    vec2 along = uMark + ab * (0.15 + 0.85 * hash(n + 9.7));
    vec2 spot = along + vec2((hash(n + 1.7) - 0.5) * 2.2 * unit + sin(uTime * 0.4 + n) * 0.12 * unit, (life - 0.5) * 0.9 * unit);
    float d = distance(p, spot);
    float glow = smoothstep(3.0, 0.0, d) + smoothstep(14.0, 0.0, d) * 0.25;
    float fade = sin(life * 3.14159);
    float tt = clamp(dot(spot - uMark, ab) / dot(ab, ab), 0.0, 1.0);
    float inside = smoothstep(mix(0.55, 1.5, tt) * unit, 0.0, distance(spot, uMark + ab * tt));
    color += vec3(1.0, 0.97, 0.92) * glow * fade * inside * 0.55;
    alpha += glow * fade * inside * 0.55;
  }
  gl_FragColor = vec4(color, clamp(alpha, 0.0, 1.0));
}
`;

type Light = { stop: () => void };

/** Compiles the one fullscreen program; null when WebGL is missing or the shader fails. */
function program(gl: WebGLRenderingContext): WebGLProgram | null {
  const shader = (type: number, source: string) => {
    const part = gl.createShader(type);
    if (!part) return null;
    gl.shaderSource(part, source);
    gl.compileShader(part);
    return gl.getShaderParameter(part, gl.COMPILE_STATUS) ? part : null;
  };
  const vs = shader(gl.VERTEX_SHADER, vertex);
  const fs = shader(gl.FRAGMENT_SHADER, fragment);
  const linked = gl.createProgram();
  if (!vs || !fs) return null;
  gl.attachShader(linked, vs);
  gl.attachShader(linked, fs);
  gl.linkProgram(linked);
  return gl.getProgramParameter(linked, gl.LINK_STATUS) ? linked : null;
}

/** Starts the light behind `host`, coming through `mark`. */
export function startLight(canvas: HTMLCanvasElement, host: HTMLElement, mark: Element): Light | null {
  // Half resolution: the light is soft, and phones keep their battery.
  const dpr = Math.min(window.devicePixelRatio || 1, 2) * 0.5;
  const gl = canvas.getContext("webgl", { alpha: true, premultipliedAlpha: false, antialias: false, powerPreference: "low-power" });
  const linked = gl ? program(gl) : null;
  if (!gl || !linked) return null;
  gl.useProgram(linked);
  gl.bindBuffer(gl.ARRAY_BUFFER, gl.createBuffer());
  // One triangle that covers the screen.
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW);
  const position = gl.getAttribLocation(linked, "position");
  gl.enableVertexAttribArray(position);
  gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0);
  const uniform = (name: string) => gl.getUniformLocation(linked, name);
  const uTime = uniform("uTime");
  const uMark = uniform("uMark");
  const uSize = uniform("uSize");
  const uSun = uniform("uSun");

  // Shader coordinates are in device pixels with y up.
  const measure = () => {
    const box = host.getBoundingClientRect();
    const m = mark.getBoundingClientRect();
    canvas.width = Math.round(box.width * dpr);
    canvas.height = Math.round(box.height * dpr);
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.uniform2f(uMark, (m.left + m.width / 2 - box.left) * dpr, (box.bottom - (m.top + m.height / 2)) * dpr);
    gl.uniform1f(uSize, Math.max(m.width, 90) * dpr);
  };
  // The sun follows the pointer slowly; on touch it sways by itself.
  let target = 0.18;
  const pointer = (event: PointerEvent) => {
    if (event.pointerType !== "mouse") return;
    const box = host.getBoundingClientRect();
    target = ((event.clientX - box.left) / box.width - 0.5) * 0.9;
  };

  let frame = 0;
  let visible = true;
  let sun = target;
  const start = performance.now();
  const render = (now: number) => {
    frame = requestAnimationFrame(render);
    const t = (now - start) / 1000;
    sun += (target + Math.sin(t * 0.21) * 0.06 - sun) * 0.03;
    gl.uniform1f(uTime, t);
    gl.uniform2f(uSun, sun, 0);
    gl.drawArrays(gl.TRIANGLES, 0, 3);
  };
  const resume = () => {
    cancelAnimationFrame(frame);
    if (visible && !document.hidden) frame = requestAnimationFrame(render);
  };

  const resize = new ResizeObserver(measure);
  resize.observe(host);
  const view = new IntersectionObserver(([entry]) => {
    visible = entry?.isIntersecting ?? false;
    resume();
  });
  view.observe(host);
  document.addEventListener("visibilitychange", resume);
  host.addEventListener("pointermove", pointer, { passive: true });
  measure();
  // The mark settles after its CSS intro; measure again once it has.
  window.setTimeout(measure, 1600);
  canvas.classList.add("is-live");

  return {
    stop: () => {
      cancelAnimationFrame(frame);
      resize.disconnect();
      view.disconnect();
      document.removeEventListener("visibilitychange", resume);
      host.removeEventListener("pointermove", pointer);
    },
  };
}
