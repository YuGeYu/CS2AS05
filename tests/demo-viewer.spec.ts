import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import MatchViewer2D from '@/features/demo/components/MatchViewer2D.vue'
import * as api from '@/services/tauri/demo'
import type { RoundPositions } from '@/types/demo'

vi.mock('@/services/tauri/demo', () => ({
  getRoundPositions: vi.fn(),
  ensureSpatialAnalysis: vi.fn(),
}))

const rounds = [{ roundNumber: 1, startTick: 1, freezeEndTick: 2, endTick: 3, officialEndTick: 4, winnerSide: 'CT', reason: null, eventCount: 1 }]
const result = (roundNumber: number, ticks = [10, 20]): RoundPositions => ({
  demoFileId: 1, roundNumber, samplingHz: 8,
  points: ticks.map(tick => ({ tick, stableKey: 'bot:1', name: 'BOT', x: -2476, y: 3239, z: 1, yaw: 0, health: 100, armor: 0, teamNumber: 2, alive: true, weapon: 'ak47' })),
})

describe('MatchViewer2D lifecycle', () => {
  const callbacks = new Map<number, FrameRequestCallback>()
  let nextRaf = 1
  let disconnected = 0

  beforeEach(() => {
    callbacks.clear(); nextRaf = 1; disconnected = 0
    vi.mocked(api.getRoundPositions).mockResolvedValue(result(1))
    vi.stubGlobal('requestAnimationFrame', vi.fn((callback: FrameRequestCallback) => { const id = nextRaf++; callbacks.set(id, callback); return id }))
    vi.stubGlobal('cancelAnimationFrame', vi.fn((id: number) => callbacks.delete(id)))
    vi.stubGlobal('ResizeObserver', class { constructor(private callback: ResizeObserverCallback) {} observe() { this.callback([], this as unknown as ResizeObserver) } disconnect() { disconnected++ } unobserve() {} })
    vi.stubGlobal('Image', class {
      onload: null | (() => void) = null; onerror: null | (() => void) = null; complete = true; naturalWidth = 1024; private value = ''
      set src(value: string) { this.value = value; if (value) queueMicrotask(() => this.onload?.()) }
      get src() { return this.value }
    })
    vi.spyOn(HTMLCanvasElement.prototype, 'getContext').mockReturnValue({
      setTransform: vi.fn(), clearRect: vi.fn(), fillRect: vi.fn(), drawImage: vi.fn(), beginPath: vi.fn(), arc: vi.fn(), fill: vi.fn(), stroke: vi.fn(),
      createRadialGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
    } as unknown as CanvasRenderingContext2D)
    Object.defineProperty(HTMLElement.prototype, 'clientWidth', { configurable: true, get: () => 640 })
    Object.defineProperty(HTMLElement.prototype, 'clientHeight', { configurable: true, get: () => 640 })
  })

  afterEach(() => { vi.restoreAllMocks(); vi.unstubAllGlobals() })

  it('leaves no RAF at the last frame, pause, visibility change, or unmount', async () => {
    const wrapper = mount(MatchViewer2D, { props: { demoId: 1, mapName: 'de_dust2', rounds, mode: 'viewer' } })
    await flushPromises()
    const play = wrapper.get('button[aria-label="播放"]')
    await play.trigger('click')
    expect(callbacks.size).toBe(1)
    let [id, callback] = [...callbacks.entries()][0]; callbacks.delete(id); callback(1000)
    ;[id, callback] = [...callbacks.entries()][0]; callbacks.delete(id); callback(2000)
    expect(callbacks.size).toBe(0)
    await flushPromises()
    await wrapper.get('button[aria-label="播放"]').trigger('click')
    expect(callbacks.size).toBe(1)
    Object.defineProperty(document, 'hidden', { configurable: true, value: true })
    document.dispatchEvent(new Event('visibilitychange'))
    expect(callbacks.size).toBe(0)
    wrapper.unmount()
    expect(callbacks.size).toBe(0)
    expect(disconnected).toBe(1)
  })

  it('ignores a stale round response and keeps the latest request', async () => {
    let resolveFirst!: (value: RoundPositions) => void
    vi.mocked(api.getRoundPositions).mockImplementationOnce(() => new Promise(resolve => { resolveFirst = resolve })).mockResolvedValueOnce(result(1, [99]))
    const wrapper = mount(MatchViewer2D, { props: { demoId: 1, mapName: 'de_dust2', rounds, mode: 'viewer' } })
    await flushPromises()
    await wrapper.findAll('select')[1].setValue('16')
    await flushPromises()
    resolveFirst(result(1, [10]))
    await flushPromises()
    expect(wrapper.text()).toContain('99')
    expect(wrapper.text()).not.toContain('10')
    wrapper.unmount()
  })
})
