// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import ThreeStage from '@/components/three/ThreeStage.vue'
import type { ThreeStageController } from '@/components/three/stage'

describe('ThreeStage loop', () => {
  let callbacks: Map<number, FrameRequestCallback>
  let nextFrame: number
  let hidden = false
  let resizeCallback: ResizeObserverCallback | null

  beforeEach(() => {
    callbacks = new Map()
    nextFrame = 1
    resizeCallback = null
    Object.defineProperty(document, 'hidden', { configurable: true, get: () => hidden })
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
      width: 800, height: 600, top: 0, left: 0, right: 800, bottom: 600, x: 0, y: 0, toJSON: () => ({}),
    })
    vi.stubGlobal('requestAnimationFrame', vi.fn((callback: FrameRequestCallback) => {
      const id = nextFrame++
      callbacks.set(id, callback)
      return id
    }))
    vi.stubGlobal('cancelAnimationFrame', vi.fn((id: number) => callbacks.delete(id)))
    vi.stubGlobal('ResizeObserver', class {
      constructor(callback: ResizeObserverCallback) { resizeCallback = callback }
      observe() { resizeCallback?.([], this as unknown as ResizeObserver) }
      disconnect() { resizeCallback = null }
      unobserve() {}
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
  })

  function runFrame(time: number) {
    const entry = callbacks.entries().next().value as [number, FrameRequestCallback] | undefined
    expect(entry).toBeDefined()
    callbacks.delete(entry![0])
    entry![1](time)
  }

  it('keeps exactly one RAF scheduled across consecutive frames', async () => {
    const controller = { frame: vi.fn(), resize: vi.fn(), dispose: vi.fn() } satisfies ThreeStageController
    const wrapper = mount(ThreeStage, { props: { factory: vi.fn().mockResolvedValue(controller) } })
    await flushPromises()
    expect(callbacks.size).toBe(1)
    runFrame(0); runFrame(16); runFrame(32)
    expect(controller.frame).toHaveBeenCalledTimes(3)
    expect(callbacks.size).toBe(1)
    wrapper.unmount()
  })

  it('pauses while hidden and recovers on visibility or focus', async () => {
    const controller = { frame: vi.fn(), resize: vi.fn(), dispose: vi.fn() } satisfies ThreeStageController
    const wrapper = mount(ThreeStage, { props: { factory: vi.fn().mockResolvedValue(controller) } })
    await flushPromises()
    hidden = true
    document.dispatchEvent(new Event('visibilitychange'))
    expect(callbacks.size).toBe(0)
    hidden = false
    document.dispatchEvent(new Event('visibilitychange'))
    expect(callbacks.size).toBe(1)
    runFrame(20)
    window.dispatchEvent(new Event('focus'))
    expect(callbacks.size).toBe(1)
    wrapper.unmount()
  })

  it('reacts to animate changes without duplicating the loop', async () => {
    const controller = { frame: vi.fn(), resize: vi.fn(), dispose: vi.fn() } satisfies ThreeStageController
    const wrapper = mount(ThreeStage, { props: { factory: vi.fn().mockResolvedValue(controller), animate: false } })
    await flushPromises()
    expect(callbacks.size).toBe(0)
    await wrapper.setProps({ animate: true })
    expect(callbacks.size).toBe(1)
    await wrapper.setProps({ animate: false })
    expect(callbacks.size).toBe(0)
    wrapper.unmount()
  })

  it('reports a frame error once and still disposes cleanly', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    const frame = vi.fn().mockImplementationOnce(() => undefined).mockImplementationOnce(() => { throw new Error('frame exploded') })
    const controller = { frame, resize: vi.fn(), dispose: vi.fn() } satisfies ThreeStageController
    const wrapper = mount(ThreeStage, { props: { factory: vi.fn().mockResolvedValue(controller) } })
    await flushPromises()
    runFrame(0); runFrame(16)
    expect(wrapper.emitted('error')).toEqual([[expect.objectContaining({ stage: 'frame', code: 'THREE_STAGE_FRAME_FAILED' })]])
    expect(callbacks.size).toBe(0)
    wrapper.unmount()
    expect(controller.dispose).toHaveBeenCalledOnce()
    document.dispatchEvent(new Event('visibilitychange'))
    window.dispatchEvent(new Event('focus'))
    expect(callbacks.size).toBe(0)
  })
})
