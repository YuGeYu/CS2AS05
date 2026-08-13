import { spawn, spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'

const args = Object.fromEntries(process.argv.slice(2).map(value => { const [key, ...rest] = value.split('='); return [key.replace(/^--/, ''), rest.join('=')] }))
const exePath = resolve(args.exe || 'src-tauri/target/debug/ai_pc_fac.exe')
const evidenceDir = resolve(args.evidence || `workspace/release-evidence/demo-preflight-closeout-${Date.now()}`)
const cdpPort = Number(args.port || 9223)
const ps = 'powershell.exe'
const sleep = ms => new Promise(resolvePromise => setTimeout(resolvePromise, ms))
const ensure = path => mkdirSync(path, { recursive: true })
const writeJson = (path, value) => { ensure(dirname(path)); writeFileSync(path, `${JSON.stringify(value, null, 2)}\n`) }
const parseJson = value => JSON.parse(String(value).replace(/^\uFEFF/, ''))
const sha256 = path => createHash('sha256').update(readFileSync(path)).digest('hex').toUpperCase()
const readJsonl = path => existsSync(path) ? readFileSync(path, 'utf8').split(/\r?\n/).filter(Boolean).map(line => JSON.parse(line)) : []
ensure(evidenceDir)
for (const name of ['baseline', 'runtime', 'windows', 'screenshots', 'database', 'automation']) ensure(join(evidenceDir, name))

function launchApp(runDir, sessionId) {
  ensure(runDir)
  return spawn(exePath, [], {
    cwd: dirname(exePath),
    windowsHide: false,
    env: { ...process.env, CS2AS_DEMO_PREFLIGHT_EVIDENCE_DIR: runDir, CS2AS_DEMO_PREFLIGHT_SESSION_ID: sessionId, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${cdpPort}` },
    stdio: ['ignore', 'pipe', 'pipe'],
  })
}

async function waitFor(predicate, timeoutMs = 20000, label = 'condition') {
  const started = Date.now()
  let lastError
  while (Date.now() - started < timeoutMs) {
    try { const value = await predicate(); if (value) return value } catch (error) { lastError = error }
    await sleep(100)
  }
  throw new Error(`Timed out waiting for ${label}${lastError ? `: ${lastError.message}` : ''}`)
}

async function waitForInteractive(runtimePath) {
  try {
    return await waitFor(() => readJsonl(runtimePath).find(record => record.event === 'interactive_ready'), 30000, 'interactive_ready')
  } catch (error) {
    let diagnostic = null
    try {
      const client = await connectTarget(target => target.type === 'page' && !target.url.includes('scoreboard.html'), 'diagnostic')
      diagnostic = await client.evaluate(`(async () => { try { return { body: document.body.innerText.slice(0, 1000), readyState: document.readyState, preflight: window.__CS2AS_PREFLIGHT__?.snapshot?.() || null, status: await window.__TAURI_INTERNALS__.invoke('preflight_status') } } catch (error) { return { body: document.body.innerText.slice(0, 1000), readyState: document.readyState, invokeError: String(error), stack: error?.stack } } })()`)
      client.close()
    } catch (diagnosticError) { diagnostic = { diagnosticError: diagnosticError.message } }
    throw new Error(`${error.message}; diagnostic=${JSON.stringify(diagnostic)}`)
  }
}

async function terminate(child, client) {
  if (client) {
    try { await Promise.race([client.evaluate(`window.__TAURI_INTERNALS__.invoke('preflight_exit_app')`), sleep(1000)]) } catch {}
  }
  if (child.exitCode == null) await Promise.race([new Promise(resolvePromise => child.once('exit', resolvePromise)), sleep(5000)])
  if (child.exitCode == null) child.kill()
  if (child.exitCode == null) await waitFor(() => child.exitCode != null, 5000, 'process exit')
}

class CdpClient {
  constructor(url) { this.url = url; this.id = 0; this.pending = new Map() }
  async connect() {
    this.ws = new WebSocket(this.url)
    this.ws.onmessage = event => { const message = JSON.parse(event.data); if (message.id && this.pending.has(message.id)) { const { resolve: done, reject } = this.pending.get(message.id); this.pending.delete(message.id); if (message.error) reject(new Error(message.error.message)); else done(message.result) } }
    await new Promise((done, reject) => { this.ws.onopen = done; this.ws.onerror = () => reject(new Error('CDP websocket failed')) })
    await this.send('Runtime.enable')
  }
  send(method, params = {}) {
    const id = ++this.id
    return new Promise((done, reject) => { this.pending.set(id, { resolve: done, reject }); this.ws.send(JSON.stringify({ id, method, params })) })
  }
  async evaluate(expression) {
    const response = await this.send('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true, userGesture: true })
    if (response.exceptionDetails) throw new Error(response.exceptionDetails.exception?.description || response.exceptionDetails.text)
    return response.result.value
  }
  close() { this.ws?.close() }
}

async function targets() {
  const response = await fetch(`http://127.0.0.1:${cdpPort}/json/list`)
  if (!response.ok) throw new Error(`CDP target list returned ${response.status}`)
  return response.json()
}

async function connectTarget(predicate, label) {
  const target = await waitFor(async () => (await targets()).find(predicate), 15000, `${label} CDP target`)
  const client = new CdpClient(target.webSocketDebuggerUrl); await client.connect(); return client
}

function runPowerShell(script, parameters, capture = true) {
  const command = ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve(script), ...parameters.map(String)]
  const result = spawnSync(ps, command, { cwd: resolve('.'), encoding: 'utf8', windowsHide: true, maxBuffer: 20 * 1024 * 1024, stdio: capture ? 'pipe' : 'ignore' })
  if (result.status !== 0) throw new Error(`${basename(script)} failed (${result.status}): ${result.stderr || result.stdout}`)
  return result.stdout?.trim()
}

function processSnapshot(pid) {
  return parseJson(runPowerShell('scripts/get-process-snapshot.ps1', ['-ProcessId', pid]))
}

async function setWindow(pid, kind, width, height, key) {
  const path = join(evidenceDir, 'windows', `${key}.json`)
  runPowerShell('scripts/set-tauri-window-rect.ps1', ['-ProcessId', pid, '-WindowKind', kind, '-Width', width, '-Height', height, '-ExpectedExePath', exePath, '-OutputJson', path])
  return path
}

async function pageState(client) {
  return client.evaluate(`(() => { const canvas = document.querySelector('canvas'); const rect = canvas?.getBoundingClientRect(); const preflight = window.__CS2AS_PREFLIGHT__?.snapshot?.() || null; return { title: document.title, bodyText: document.body.innerText.slice(0, 2000), scrollWidth: document.documentElement.scrollWidth, clientWidth: document.documentElement.clientWidth, overflowX: document.documentElement.scrollWidth > document.documentElement.clientWidth, activeElement: document.activeElement?.getAttribute('aria-label') || document.activeElement?.textContent?.trim() || null, canvas: rect ? { x: rect.x, y: rect.y, width: rect.width, height: rect.height } : null, preflight } })()`)
}

async function capture(client, windowJson, key, page, extra = {}) {
  const state = await pageState(client)
  const png = join(evidenceDir, 'screenshots', `${key}.png`)
  const parameters = ['-WindowEvidenceJson', windowJson, '-OutputPng', png, '-ExeSha256', candidate.exe.sha256, '-Page', page]
  for (const [name, value] of Object.entries({ DemoId: extra.demoId, Round: extra.round, Tick: state.preflight?.tick, Points: state.preflight?.points })) if (value) parameters.push(`-${name}`, value)
  if (state.canvas) parameters.push('-CanvasRectJson', JSON.stringify(state.canvas))
  runPowerShell('scripts/capture-window.ps1', parameters)
  writeJson(join(evidenceDir, 'screenshots', `${key}-dom.json`), state)
  return { key, png, metrics: parseJson(readFileSync(png.replace(/\.png$/, '.json'), 'utf8')), dom: state }
}

async function click(client, expression, label) {
  const clicked = await client.evaluate(`(() => { const element = ${expression}; if (!element) return false; element.click(); return true })()`)
  if (!clicked) throw new Error(`Unable to click ${label}`)
}

async function clickText(client, selector, text) {
  const literal = JSON.stringify(text)
  await click(client, `[...document.querySelectorAll(${JSON.stringify(selector)})].find(element => element.textContent.trim() === ${literal})`, text)
}

async function openDemoWorkspace(main) {
  const hasIntro = await main.evaluate(`Boolean(document.querySelector('.intro-overlay'))`)
  if (hasIntro) {
    await clickText(main, '.intro-overlay button', '跳过')
    await waitFor(() => main.evaluate(`!document.querySelector('.intro-overlay')`), 5000, 'intro close')
  }
  await click(main, `document.querySelector('button[aria-label="对局复盘"]')`, '对局复盘')
  await waitFor(() => main.evaluate(`Boolean(document.querySelector('.demo-table'))`), 10000, 'demo library')
}

async function openDust2Report(main) {
  const clicked = await main.evaluate(`(() => { const row = [...document.querySelectorAll('.demo-table tbody tr')].find(row => row.textContent.includes('de_dust2') && row.textContent.includes('已完成')); const button = [...(row?.querySelectorAll('button') || [])].find(item => item.textContent.trim() === '打开报告'); button?.click(); return Boolean(button) })()`)
  if (!clicked) throw new Error('completed dust2 report was not found')
  await waitFor(() => main.evaluate(`Boolean(document.querySelector('.match-tabs')) && document.querySelector('.report-summary h2')?.textContent.trim() === 'de_dust2'`), 15000, 'dust2 report')
}

async function showReportTab(main, label) {
  await clickText(main, '[data-report-tab]', label)
  await sleep(450)
}

async function waitViewer(main, mode = 'viewer') {
  await waitFor(() => main.evaluate(`(() => { const value = window.__CS2AS_PREFLIGHT__?.snapshot?.(); return value?.mounted && value?.mode === ${JSON.stringify(mode)} && value.points > 0 ? value : null })()`), 15000, `${mode} positions`)
}

const candidate = { exe: { path: exePath, bytes: readFileSync(exePath).length, sha256: sha256(exePath) }, cdpPort, generatedAt: new Date().toISOString() }
writeJson(join(evidenceDir, 'baseline', 'runtime-candidate.json'), candidate)

const coldStarts = []
for (let index = 1; index <= 5; index++) {
  const runDir = join(evidenceDir, 'runtime', `cold-${index}`); const sessionId = `cold_${index}_${Date.now()}`
  const child = launchApp(runDir, sessionId)
  const stdout = []; const stderr = []; child.stdout.on('data', value => stdout.push(value)); child.stderr.on('data', value => stderr.push(value))
  let client
  try {
    const ready = await waitForInteractive(join(runDir, 'runtime.jsonl'))
    const records = readJsonl(join(runDir, 'runtime.jsonl')); const started = records.find(record => record.event === 'process_started')
    coldStarts.push({ index, pid: child.pid, sessionId, processStartedMs: started.monotonicMs, interactiveReadyMs: ready.monotonicMs, coldStartInteractiveMs: ready.monotonicMs - started.monotonicMs, ready: ready.data })
    client = await connectTarget(target => target.type === 'page' && !target.url.includes('scoreboard.html'), 'main')
    await terminate(child, client)
  } catch (error) {
    writeJson(join(runDir, 'error.json'), { message: error.message, stack: error.stack })
    if (child.exitCode == null) child.kill()
    throw error
  } finally {
    client?.close(); writeFileSync(join(runDir, 'stdout.log'), Buffer.concat(stdout)); writeFileSync(join(runDir, 'stderr.log'), Buffer.concat(stderr))
  }
  await sleep(2000)
}
writeJson(join(evidenceDir, 'runtime', 'cold-starts.json'), coldStarts)

const finalDir = join(evidenceDir, 'runtime', 'final')
const finalSessionId = `final_${Date.now()}`
const child = launchApp(finalDir, finalSessionId)
const finalStdout = []; const finalStderr = []; child.stdout.on('data', value => finalStdout.push(value)); child.stderr.on('data', value => finalStderr.push(value))
let main
try {
  await waitForInteractive(join(finalDir, 'runtime.jsonl'))
  main = await connectTarget(target => target.type === 'page' && !target.url.includes('scoreboard.html'), 'main')
  await openDemoWorkspace(main)
  const screenshots = []
  for (const [width, height] of [[1440, 900], [1100, 700]]) {
    const size = `${width}x${height}`; const windowJson = await setWindow(child.pid, 'main', width, height, `main-${size}`)
    screenshots.push(await capture(main, windowJson, `main-${size}-library`, 'library'))
    await openDust2Report(main)
    await showReportTab(main, '总览')
    screenshots.push(await capture(main, windowJson, `main-${size}-overview`, 'overview', { demoId: 4 }))
    await showReportTab(main, '回合'); screenshots.push(await capture(main, windowJson, `main-${size}-rounds`, 'rounds', { demoId: 4 }))
    await showReportTab(main, '地图回放'); await waitViewer(main, 'viewer'); screenshots.push(await capture(main, windowJson, `main-${size}-viewer`, 'viewer', { demoId: 4, round: 1 }))
    await showReportTab(main, '热力图'); await waitViewer(main, 'heatmap'); screenshots.push(await capture(main, windowJson, `main-${size}-heatmap`, 'heatmap', { demoId: 4, round: 1 }))
    await clickText(main, '.demo-tabs button', '录像库'); await sleep(300)
  }

  await main.evaluate(`window.__TAURI_INTERNALS__.invoke('open_scoreboard', { reportId: 4 })`)
  const scoreboard = await connectTarget(target => target.type === 'page' && target.url.includes('scoreboard.html'), 'scoreboard')
  try {
    await waitFor(() => scoreboard.evaluate(`document.body.dataset.scoreboardReady === 'true' && Boolean(document.querySelector('.scoreboard-table-wrap'))`), 20000, 'scoreboard content')
  } catch (error) {
    const diagnostic = await scoreboard.evaluate(`({ body: document.body.innerText, dataset: { ...document.body.dataset }, html: document.body.innerHTML.slice(0, 3000) })`)
    writeJson(join(evidenceDir, 'runtime', 'scoreboard-diagnostic.json'), diagnostic)
    throw new Error(`${error.message}; scoreboard=${JSON.stringify(diagnostic)}`)
  }
  for (const [width, height] of [[1280, 800], [980, 640]]) {
    const size = `${width}x${height}`; const windowJson = await setWindow(child.pid, 'scoreboard', width, height, `scoreboard-${size}`)
    screenshots.push(await capture(scoreboard, windowJson, `scoreboard-${size}`, 'scoreboard', { demoId: 4 }))
  }
  scoreboard.close()

  await setWindow(child.pid, 'main', 1440, 900, 'main-performance-1440x900')
  await clickText(main, '.demo-tabs button', '对局报告'); await showReportTab(main, '地图回放'); await waitViewer(main, 'viewer')
  const ipc = await main.evaluate(`window.__CS2AS_PREFLIGHT__.measureRoundPositions(4, 1, 8, 3, 30)`)
  writeJson(join(evidenceDir, 'runtime', 'round-positions.json'), ipc)

  const cpu = {}
  async function measureState(label, prepare) {
    await prepare(); await sleep(2000); await main.evaluate(`window.__CS2AS_PREFLIGHT__.startViewerMeasurement(${JSON.stringify(label)})`)
    const output = join(evidenceDir, 'runtime', `cpu-${label}.json`)
    const sampler = spawn(ps, ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', resolve('scripts/sample-process.ps1'), '-ProcessId', child.pid, '-OutputJson', output, '-DurationMs', 10000, '-IntervalMs', 500, '-State', label], { windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] })
    await new Promise((done, reject) => { sampler.once('exit', code => code === 0 ? done() : reject(new Error(`CPU sampler ${label} failed with ${code}`))) })
    const viewer = await main.evaluate(`window.__CS2AS_PREFLIGHT__.stopViewerMeasurement()`)
    cpu[label] = { process: parseJson(readFileSync(output, 'utf8')), viewer }
  }
  await measureState('active', async () => { const snap = await main.evaluate(`window.__CS2AS_PREFLIGHT__.snapshot()`); if (!snap.playing) await click(main, `document.querySelector('button[aria-label="播放"]')`, 'play') })
  await measureState('paused', async () => { const snap = await main.evaluate(`window.__CS2AS_PREFLIGHT__.snapshot()`); if (snap.playing) await click(main, `document.querySelector('button[aria-label="暂停"]')`, 'pause') })
  await measureState('tab-away', async () => { await showReportTab(main, '回合'); await waitFor(() => main.evaluate(`!window.__CS2AS_PREFLIGHT__.snapshot().mounted`), 5000, 'viewer unmount') })
  writeJson(join(evidenceDir, 'runtime', 'cpu-and-fps.json'), cpu)

  const lifecycle = []; const baselineSnapshot = await main.evaluate(`window.__CS2AS_PREFLIGHT__.snapshot()`)
  for (let cycle = 1; cycle <= 20; cycle++) {
    await showReportTab(main, '地图回放'); await waitViewer(main, 'viewer'); await sleep(400)
    const entered = { process: processSnapshot(child.pid), viewer: await main.evaluate(`window.__CS2AS_PREFLIGHT__.snapshot()`) }
    await showReportTab(main, '回合'); await waitFor(() => main.evaluate(`!window.__CS2AS_PREFLIGHT__.snapshot().mounted`), 5000, 'viewer unmount'); await sleep(700)
    lifecycle.push({ cycle, entered, left: { process: processSnapshot(child.pid), viewer: await main.evaluate(`window.__CS2AS_PREFLIGHT__.snapshot()`) } })
  }
  writeJson(join(evidenceDir, 'runtime', 'lifecycle-20.json'), { baseline: baselineSnapshot, cycles: lifecycle })
  writeJson(join(evidenceDir, 'runtime', 'runtime-index.json'), { pid: child.pid, sessionId: finalSessionId, screenshots, ipc, cpu, lifecycle: { baseline: baselineSnapshot, cycles: lifecycle } })
  await terminate(child, main)
} catch (error) {
  writeJson(join(evidenceDir, 'runtime', 'runner-error.json'), { timestamp: new Date().toISOString(), message: error.message, stack: error.stack })
  if (child.exitCode == null) child.kill()
  throw error
} finally {
  main?.close(); writeFileSync(join(finalDir, 'stdout.log'), Buffer.concat(finalStdout)); writeFileSync(join(finalDir, 'stderr.log'), Buffer.concat(finalStderr))
}

console.log(JSON.stringify({ evidenceDir, candidate, coldStarts, finalPid: child.pid }, null, 2))
