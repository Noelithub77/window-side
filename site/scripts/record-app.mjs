import { createRequire } from 'node:module'
import { mkdir, writeFile, rm } from 'node:fs/promises'
import { join, resolve } from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'
import { execFile } from 'node:child_process'
import { promisify } from 'node:util'

const run = promisify(execFile)
const require = createRequire(resolve('ui/package.json'))
const { chromium } = require('@playwright/test')
const [baseUrl, outputArg] = process.argv.slice(2)
if (!baseUrl || !outputArg) throw new Error('Usage: node site/scripts/record-app.mjs <base-url> <output-dir>')
const output = resolve(outputArg)
const raw = '/tmp/ws-landing/ui'
await mkdir(output, { recursive: true })
await mkdir(raw, { recursive: true })
const browser = await chromium.launch({ headless: true })
const page = await browser.newPage({ viewport: { width: 1280, height: 860 }, deviceScaleFactor: 2, colorScheme: 'dark' })
await page.addStyleTag({ content: `* { cursor: none !important } html { scroll-behavior: auto !important } .capture-cursor { position: fixed; z-index: 999999; width: 14px; height: 14px; margin: -7px; border: 2px solid #d5e3ff; border-radius: 50%; background: #7aa2f7aa; box-shadow: 0 0 16px #7aa2f7aa; pointer-events: none; transition: transform 180ms ease-out; } .capture-cursor.tap { transform: scale(1.7); }` }).catch(() => {})
await page.addInitScript(() => {
  document.addEventListener('DOMContentLoaded', () => {
    const style = document.createElement('style'); style.textContent = `.capture-cursor { position: fixed; z-index: 999999; width: 14px; height: 14px; margin: -7px; border: 2px solid #d5e3ff; border-radius: 50%; background: #7aa2f7aa; box-shadow: 0 0 16px #7aa2f7aa; pointer-events: none; transition: transform 180ms ease-out; } .capture-cursor.tap { transform: scale(1.7); }`; document.head.append(style)
    const dot = document.createElement('i'); dot.className = 'capture-cursor'; document.body.append(dot)
    document.addEventListener('mousemove', event => { dot.style.left = `${event.clientX}px`; dot.style.top = `${event.clientY}px` })
    document.addEventListener('click', () => { dot.classList.add('tap'); setTimeout(() => dot.classList.remove('tap'), 220) })
  }, { once: true })
})

async function open(fixture) {
  await page.goto(`${baseUrl}/?fixture=${fixture}`, { waitUntil: 'networkidle' })
  await page.waitForTimeout(1000)
}
async function clickText(text) { await page.getByText(text, { exact: true }).first().click() }
async function still(name, selector = 'body') {
  await page.locator(selector).screenshot({ path: join(output, `still-${name}.png`), type: 'png' }).catch(async () => page.screenshot({ path: join(output, `still-${name}.png`) }))
}
async function capture(name, durationMs, choreography) {
  const dir = join(raw, name)
  await rm(dir, { recursive: true, force: true }); await mkdir(dir, { recursive: true })
  const cdp = await page.context().newCDPSession(page)
  const frames = []; let chain = Promise.resolve()
  cdp.on('Page.screencastFrame', frame => {
    const index = frames.length; const stamp = frame.metadata.timestamp
    chain = chain.then(() => writeFile(join(dir, `${String(index).padStart(5, '0')}.jpg`), Buffer.from(frame.data, 'base64')))
    frames.push({ index, stamp })
    void cdp.send('Page.screencastFrameAck', { sessionId: frame.sessionId })
  })
  await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 95, everyNthFrame: 1, maxWidth: 1280, maxHeight: 860 })
  const started = Date.now()
  await choreography()
  await page.waitForTimeout(Math.max(0, durationMs - (Date.now() - started)))
  await cdp.send('Page.stopScreencast'); await chain
  if (frames.length < 2) throw new Error(`${name}: screencast returned only ${frames.length} frame`)
  const manifest = join(dir, 'frames.txt')
  const lines = []
  for (let i = 0; i < frames.length; i++) {
    lines.push(`file '${String(i).padStart(5, '0')}.jpg'`)
    const next = frames[i + 1]?.stamp ?? frames[i].stamp + 1 / 30
    lines.push(`duration ${Math.max(1 / 120, next - frames[i].stamp).toFixed(6)}`)
  }
  lines.push(`file '${String(frames.length - 1).padStart(5, '0')}.jpg'`)
  await writeFile(manifest, `${lines.join('\n')}\n`)
  const mp4 = join(output, `${name}.mp4`)
  await run('ffmpeg', ['-y', '-f', 'concat', '-safe', '0', '-i', manifest, '-vf', 'fps=30,scale=1280:-2', '-c:v', 'libx264', '-crf', '26', '-preset', 'slow', '-pix_fmt', 'yuv420p', '-movflags', '+faststart', '-an', mp4], { cwd: dir })
  const webm = join(output, `${name}.webm`)
  await run('ffmpeg', ['-y', '-i', mp4, '-vf', 'scale=1280:-2', '-c:v', 'libsvtav1', '-crf', '44', '-preset', '5', '-pix_fmt', 'yuv420p', '-g', '240', '-an', webm])
  await run('ffmpeg', ['-y', '-i', mp4, '-frames:v', '1', '-vf', 'scale=1280:-2', '-c:v', 'libsvtav1', '-preset', '8', '-crf', '42', `${output}/${name}.avif`])
  await run('ffmpeg', ['-y', '-i', mp4, '-frames:v', '1', '-vf', 'scale=1280:-2', '-c:v', 'libwebp', '-quality', '90', `${output}/${name}.webp`])
  console.log(`${name}: ${frames.length} frames, ${((frames.at(-1).stamp - frames[0].stamp)).toFixed(2)}s`)
}

await open('setup')
  await page.screenshot({ path: join(raw, 'still-welcome.png') })
await capture('welcome', 7500, async () => {
  await page.waitForTimeout(3200)
  await page.mouse.move(640, 610, { steps: 35 })
  await page.waitForTimeout(900)
})
await open('setup'); await clickText('Get started'); await page.waitForTimeout(450)
  await page.screenshot({ path: join(raw, 'still-setup-windows.png') })
await clickText('Samsung 990 Pro'); await clickText('Continue'); await page.waitForTimeout(450)
  await page.screenshot({ path: join(raw, 'still-setup-graphics.png') })
await clickText('Continue'); await page.waitForTimeout(450)
  await page.screenshot({ path: join(raw, 'still-setup-resources.png') })
await capture('setup', 11500, async () => {
  await page.waitForTimeout(900); await page.locator('[role="slider"]').first().focus(); await page.keyboard.press('ArrowRight'); await page.waitForTimeout(1000)
  await page.getByRole('button', { name: 'Review' }).last().click(); await page.waitForTimeout(1300); await page.screenshot({ path: join(raw, 'still-setup-review.png') })
  await page.waitForTimeout(600); await clickText('Apply'); await page.waitForTimeout(2600)
})
await open('status'); await page.screenshot({ path: join(raw, 'still-overview.png') })
await capture('overview', 8000, async () => {})
await open('stopped'); await page.screenshot({ path: join(raw, 'still-stopped.png') })
await capture('start', 9000, async () => { await page.waitForTimeout(1300); await clickText('Launch'); await page.waitForTimeout(6500) })
await open('first-boot-start'); await page.screenshot({ path: join(raw, 'still-first-boot.png') })
await capture('first-boot', 10500, async () => { await page.waitForTimeout(1300); await clickText('Start'); await page.waitForTimeout(8500) })
await open('status'); await clickText('Settings'); await page.waitForTimeout(600); await page.screenshot({ path: join(raw, 'still-settings.png') })
await capture('settings', 7500, async () => { await page.waitForTimeout(1100); await page.mouse.wheel(0, 410); await page.waitForTimeout(800); await clickText('Appearance'); await page.waitForTimeout(900); await page.mouse.wheel(0, -410) })
await open('setup'); await page.getByRole('button', { name: /Set up with an agent/ }).click(); await page.waitForTimeout(400); await page.screenshot({ path: join(raw, 'still-agent.png') })
await capture('agent', 7000, async () => { await page.waitForTimeout(1800) })

for (const name of ['welcome', 'setup-windows', 'setup-graphics', 'setup-resources', 'setup-review', 'overview', 'settings', 'first-boot', 'agent']) {
  const source = name.startsWith('setup-') ? join(raw, `still-${name}.png`) : join(raw, `still-${name}.png`)
  for (const [extension, args] of [['avif', ['-c:v', 'libsvtav1', '-preset', '8', '-crf', '42']], ['webp', ['-c:v', 'libwebp', '-quality', '92']]]) {
    await run('ffmpeg', ['-y', '-i', source, '-vf', 'scale=1600:-2', ...args, join(output, `still-${name}.${extension}`)])
  }
}
await browser.close()
