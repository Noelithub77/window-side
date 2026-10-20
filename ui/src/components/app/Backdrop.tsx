import { useEffect, useRef } from "react"
import { Application, Filter, GlProgram, Sprite, Texture, UniformGroup } from "pixi.js"

export type Mood = "idle" | "live" | "busy" | "gpu"

type Rgb = [number, number, number]

/** Three light colours per mood; the shader drifts between them over ink. */
const MOODS: Record<Mood, { colors: [Rgb, Rgb, Rgb]; energy: number }> = {
  idle: { colors: [[0.62, 0.58, 0.92], [0.45, 0.62, 0.86], [0.85, 0.66, 0.62]], energy: 0.35 },
  live: { colors: [[0.48, 0.9, 0.76], [0.66, 0.6, 0.95], [0.5, 0.74, 0.95]], energy: 0.75 },
  busy: { colors: [[0.95, 0.76, 0.55], [0.66, 0.6, 0.95], [0.48, 0.9, 0.76]], energy: 1 },
  gpu: { colors: [[0.96, 0.6, 0.76], [0.66, 0.6, 0.95], [0.48, 0.9, 0.76]], energy: 0.8 },
}

const vertex = `
in vec2 aPosition;
out vec2 vTextureCoord;
out vec2 vUv;
uniform vec4 uInputSize;
uniform vec4 uOutputFrame;
uniform vec4 uOutputTexture;

void main(void) {
  vec2 position = aPosition * uOutputFrame.zw + uOutputFrame.xy;
  position.x = position.x * (2.0 / uOutputTexture.x) - 1.0;
  position.y = position.y * (2.0 * uOutputTexture.z / uOutputTexture.y) - uOutputTexture.z;
  gl_Position = vec4(position, 0.0, 1.0);
  vTextureCoord = aPosition * (uOutputFrame.zw * uInputSize.zw);
  vUv = aPosition;
}
`

// Soft fbm aurora curtains, a few twinkling motes, vignette and film grain.
const fragment = `
precision highp float;
in vec2 vUv;
out vec4 finalColor;
uniform float uTime;
uniform float uEnergy;
uniform float uAspect;
uniform vec3 uA;
uniform vec3 uB;
uniform vec3 uC;

float hash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float noise(vec2 p) {
  vec2 i = floor(p); vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x), mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}
float fbm(vec2 p) {
  float v = 0.0; float a = 0.5;
  for (int i = 0; i < 5; i++) { v += a * noise(p); p = p * 2.03 + vec2(1.7, 9.2); a *= 0.5; }
  return v;
}

void main(void) {
  vec2 uv = vUv;
  vec2 p = vec2(uv.x * uAspect, uv.y);
  float t = uTime * (0.025 + 0.05 * uEnergy);
  vec3 ink = vec3(0.058, 0.056, 0.085);

  float warp = fbm(p * 1.4 + vec2(t, -t * 0.6));
  float bands = fbm(vec2(p.x * 1.1 + warp * 1.6 + t * 0.8, p.y * 0.35 - t * 0.3));
  float curtain = smoothstep(0.35, 0.95, bands) * smoothstep(1.05, 0.05, uv.y);

  vec3 hue = mix(uA, uB, smoothstep(0.15, 0.85, uv.x + 0.25 * sin(t * 2.0 + warp * 3.0)));
  hue = mix(hue, uC, smoothstep(0.55, 1.0, warp));

  float glow = curtain * (0.34 + 0.26 * uEnergy);
  float halo = 0.22 * exp(-3.2 * length(vec2((uv.x - 0.8) * uAspect, uv.y - 0.02)));
  halo += 0.16 * exp(-3.8 * length(vec2((uv.x - 0.1) * uAspect, uv.y - 0.1)));
  vec3 color = ink + hue * (glow + halo);

  vec2 cell = floor(p * 46.0);
  float star = hash(cell);
  vec2 local = fract(p * 46.0) - 0.5;
  float twinkle = 0.5 + 0.5 * sin(uTime * (0.6 + star * 1.4) + star * 40.0);
  float mote = step(0.985, star) * smoothstep(0.09, 0.0, length(local)) * twinkle;
  color += hue * mote * (0.18 + 0.2 * uEnergy) * smoothstep(1.0, 0.2, uv.y);

  float vignette = smoothstep(1.25, 0.35, length((uv - vec2(0.5, 0.42)) * vec2(uAspect * 0.8, 1.0)));
  color *= mix(0.7, 1.0, vignette);
  color += (hash(uv * 900.0 + fract(uTime)) - 0.5) * 0.018;
  finalColor = vec4(color, 1.0);
}
`

type Props = { mood: Mood; tint?: Rgb | null }

/** Full-window WebGL aurora behind every page; plain gradients when WebGL is missing. */
export function Backdrop({ mood, tint = null }: Props) {
  const host = useRef<HTMLDivElement>(null)
  const target = useRef({ mood, tint })

  useEffect(() => {
    target.current = { mood, tint }
  }, [mood, tint])

  useEffect(() => {
    const element = host.current
    if (!element) return undefined
    const app = new Application()
    let ready = false
    let disposed = false
    const still = window.matchMedia("(prefers-reduced-motion: reduce)").matches

    void app
      .init({ resizeTo: element, preference: "webgl", antialias: false, resolution: 0.5, autoDensity: true, backgroundAlpha: 1 })
      .then(() => {
        if (disposed) {
          app.destroy(true)
          return
        }
        ready = true
        app.canvas.style.width = "100%"
        app.canvas.style.height = "100%"
        element.appendChild(app.canvas)
        const surface = new Sprite(Texture.WHITE)
        const start = MOODS[target.current.mood]
        const aurora = new UniformGroup({
          uTime: { value: 0, type: "f32" },
          uEnergy: { value: start.energy, type: "f32" },
          uAspect: { value: 1, type: "f32" },
          uA: { value: new Float32Array(start.colors[0]), type: "vec3<f32>" },
          uB: { value: new Float32Array(start.colors[1]), type: "vec3<f32>" },
          uC: { value: new Float32Array(start.colors[2]), type: "vec3<f32>" },
        })
        surface.filters = [new Filter({ glProgram: GlProgram.from({ vertex, fragment, name: "aurora" }), resources: { aurora } })]
        app.stage.addChild(surface)
        const uniforms = aurora.uniforms
        app.ticker.maxFPS = 30
        app.ticker.add((ticker) => {
          surface.width = app.screen.width
          surface.height = app.screen.height
          uniforms.uAspect = app.screen.width / Math.max(1, app.screen.height)
          const goal = MOODS[target.current.mood]
          const ease = Math.min(1, ticker.deltaMS / 900)
          uniforms.uEnergy += (goal.energy - uniforms.uEnergy) * ease
          const colors = [uniforms.uA, uniforms.uB, uniforms.uC]
          colors.forEach((color, slot) => {
            const wanted = slot === 0 && target.current.tint ? target.current.tint : goal.colors[slot]
            if (!wanted) return
            for (let channel = 0; channel < 3; channel += 1) {
              const current = color[channel] ?? 0
              color[channel] = current + ((wanted[channel] ?? 0) - current) * ease
            }
          })
          if (!still) uniforms.uTime += ticker.deltaMS / 1000
        })
      })
      .catch(() => {
        element.dataset.fallback = "true"
      })

    return () => {
      disposed = true
      if (ready) app.destroy(true)
    }
  }, [])

  return (
    <div
      ref={host}
      aria-hidden
      className="pointer-events-none fixed inset-0 -z-10 bg-ink data-[fallback=true]:bg-[radial-gradient(60%_50%_at_80%_0%,color-mix(in_oklch,var(--lavender)_22%,transparent),transparent),radial-gradient(50%_45%_at_10%_10%,color-mix(in_oklch,var(--mint)_18%,transparent),transparent)]"
    />
  )
}
