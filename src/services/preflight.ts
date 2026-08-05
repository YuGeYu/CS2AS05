import { invoke } from '@tauri-apps/api/core'
import type { RoundPositions } from '@/types/demo'

interface PreflightStatus { enabled: boolean; elapsedMs: number; sessionId: string | null }
interface ViewerSnapshot { mounted: boolean; mode: 'viewer' | 'heatmap' | null; playing: boolean; tick: number; points: number; rafCallbacks: number; mounts: number; unmounts: number }
interface ViewerMeasurement { label: string; startedAt: number; tickStart: number; rafStart: number; lastFrameAt: number | null; buckets: number[]; longFrames: number }

const snapshot: ViewerSnapshot = { mounted: false, mode: null, playing: false, tick: 0, points: 0, rafCallbacks: 0, mounts: 0, unmounts: 0 }
let measurement: ViewerMeasurement | null = null
let enabled = false

async function record(event: string, data: Record<string, unknown> = {}) {
  if (!import.meta.env.DEV || !enabled) return null
  return invoke('preflight_record_event', { event, data })
}

function viewport() {
  return { width: window.innerWidth, height: window.innerHeight, dpi: window.devicePixelRatio || 1 }
}

export async function initializePreflight() {
  if (!import.meta.env.DEV) return false
  const status = await invoke<PreflightStatus>('preflight_status')
  enabled = status.enabled
  if (enabled) installGlobalApi()
  return enabled
}

export async function recordInteractiveReady(data: Record<string, unknown>) {
  return record('interactive_ready', { ...data, viewport: viewport() })
}

export function viewerMounted(mode: 'viewer' | 'heatmap') {
  if (!enabled) return
  snapshot.mounted = true; snapshot.mode = mode; snapshot.mounts++
  void record('viewer_mounted', { mode, mounts: snapshot.mounts, viewport: viewport() })
}

export function viewerLoaded(mode: 'viewer' | 'heatmap', tick: number, points: number) {
  if (!enabled) return
  snapshot.mode = mode; snapshot.tick = tick; snapshot.points = points
  void record('viewer_loaded', { mode, tick, points, viewport: viewport() })
}

export function viewerPlaying(playing: boolean) {
  if (!enabled || snapshot.playing === playing) return
  snapshot.playing = playing
  void record(playing ? 'viewer_play_started' : 'viewer_paused', { tick: snapshot.tick, points: snapshot.points })
}

export function viewerFrame(time: number, tick: number, points: number) {
  if (!enabled) return
  snapshot.rafCallbacks++; snapshot.tick = tick; snapshot.points = points
  if (!measurement) return
  const elapsed = time - measurement.startedAt
  const bucket = Math.max(0, Math.floor(elapsed / 1000))
  measurement.buckets[bucket] = (measurement.buckets[bucket] || 0) + 1
  if (measurement.lastFrameAt != null && time - measurement.lastFrameAt > 34) measurement.longFrames++
  measurement.lastFrameAt = time
}

export function viewerUnmounted() {
  if (!enabled) return
  snapshot.mounted = false; snapshot.playing = false; snapshot.unmounts++
  void record('viewer_unmounted', { mode: snapshot.mode, tick: snapshot.tick, points: snapshot.points, mounts: snapshot.mounts, unmounts: snapshot.unmounts })
  snapshot.mode = null
}

function startViewerMeasurement(label: string) {
  measurement = { label, startedAt: performance.now(), tickStart: snapshot.tick, rafStart: snapshot.rafCallbacks, lastFrameAt: null, buckets: [], longFrames: 0 }
  return { ...snapshot }
}

async function stopViewerMeasurement() {
  if (!measurement) throw new Error('viewer measurement was not started')
  const finishedAt = performance.now()
  const result = {
    label: measurement.label,
    elapsedMs: finishedAt - measurement.startedAt,
    rafCallbacks: snapshot.rafCallbacks - measurement.rafStart,
    fps: (snapshot.rafCallbacks - measurement.rafStart) * 1000 / (finishedAt - measurement.startedAt),
    oneSecondBuckets: measurement.buckets,
    minimumBucketFps: measurement.buckets.length ? Math.min(...measurement.buckets) : 0,
    longFrames: measurement.longFrames,
    tickStart: measurement.tickStart,
    tickEnd: snapshot.tick,
    points: snapshot.points,
    mounted: snapshot.mounted,
    playing: snapshot.playing,
    viewport: viewport(),
  }
  measurement = null
  await record('viewer_measurement', result)
  return result
}

async function measureRoundPositions(demoId = 4, roundNumber = 1, samplingHz = 8, warmups = 3, iterations = 30) {
  const samples: Array<{ index: number; elapsedMs: number; points: number; roundNumber: number }> = []
  for (let index = -warmups; index < iterations; index++) {
    const startedAt = performance.now()
    const result = await invoke<RoundPositions>('get_round_positions', { demoId, roundNumber, samplingHz })
    const sample = { index: Math.max(0, index), elapsedMs: performance.now() - startedAt, points: result.points.length, roundNumber: result.roundNumber }
    if (index >= 0) samples.push(sample)
  }
  const result = { demoId, roundNumber, samplingHz, warmups, iterations, samples }
  await record('round_positions_measurement', result)
  return result
}

function installGlobalApi() {
  window.__CS2AS_PREFLIGHT__ = {
    snapshot: () => ({ ...snapshot }),
    startViewerMeasurement,
    stopViewerMeasurement,
    measureRoundPositions,
  }
}

declare global {
  interface Window {
    __CS2AS_PREFLIGHT__?: {
      snapshot: () => ViewerSnapshot
      startViewerMeasurement: (label: string) => ViewerSnapshot
      stopViewerMeasurement: () => Promise<Record<string, unknown>>
      measureRoundPositions: (demoId?: number, roundNumber?: number, samplingHz?: number, warmups?: number, iterations?: number) => Promise<Record<string, unknown>>
    }
  }
}
