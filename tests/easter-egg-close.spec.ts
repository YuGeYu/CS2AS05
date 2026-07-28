// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import EasterEggGame from '@/components/easter-egg/EasterEggGame.vue'

const interact = vi.fn()
vi.mock('@/components/three/ThreeStage.vue', () => ({
  default: defineComponent({
    name: 'ThreeStage',
    setup(_, { expose }) {
      expose({ interact })
      return () => h('div', { class: 'three-stage-stub' })
    },
  }),
}))

describe('EasterEggGame close control', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    interact.mockReset()
    Object.defineProperties(HTMLElement.prototype, {
      setPointerCapture: { configurable: true, writable: true, value: vi.fn() },
      releasePointerCapture: { configurable: true, writable: true, value: vi.fn() },
      hasPointerCapture: { configurable: true, writable: true, value: vi.fn(() => false) },
    })
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
      width: 800, height: 600, top: 0, left: 0, right: 800, bottom: 600, x: 0, y: 0, toJSON: () => ({}),
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  async function mountGame() {
    const wrapper = mount(EasterEggGame)
    await flushPromises()
    return wrapper
  }

  function dispatchPointer(element: Element, type: string, pointerId: number, clientX = 0, clientY = 0) {
    const event = new Event(type, { bubbles: true, cancelable: true })
    Object.defineProperties(event, {
      pointerId: { value: pointerId },
      clientX: { value: clientX },
      clientY: { value: clientY },
    })
    element.dispatchEvent(event)
  }

  it('closes once from pointerdown during countdown', async () => {
    const wrapper = await mountGame()
    await wrapper.get('[aria-label="关闭青冥试剑"]').trigger('pointerdown', { pointerId: 1 })
    await wrapper.get('[aria-label="关闭青冥试剑"]').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('releases input capture and closes once while playing', async () => {
    const captures = new Set<number>()
    const setPointerCapture = vi.spyOn(HTMLElement.prototype, 'setPointerCapture').mockImplementation((id: number) => captures.add(id))
    const releasePointerCapture = vi.spyOn(HTMLElement.prototype, 'releasePointerCapture').mockImplementation((id: number) => captures.delete(id))
    vi.spyOn(HTMLElement.prototype, 'hasPointerCapture').mockImplementation((id: number) => captures.has(id))
    const wrapper = await mountGame()
    await vi.advanceTimersByTimeAsync(3_000)
    const input = wrapper.get('.game-input-layer')
    dispatchPointer(input.element, 'pointerdown', 7, 100, 100)
    expect(setPointerCapture).toHaveBeenCalledWith(7)
    expect(interact).toHaveBeenCalledWith(expect.objectContaining({ type: 'pointer-down' }))
    await wrapper.get('[aria-label="关闭青冥试剑"]').trigger('pointerdown', { pointerId: 8 })
    await wrapper.get('[aria-label="关闭青冥试剑"]').trigger('click')
    expect(releasePointerCapture).toHaveBeenCalledWith(7)
    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(interact).toHaveBeenCalledTimes(1)
    wrapper.unmount()
  })

  it('closes once with Escape and clears the clock', async () => {
    const wrapper = await mountGame()
    await vi.advanceTimersByTimeAsync(3_000)
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    await vi.advanceTimersByTimeAsync(2_000)
    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(wrapper.text()).toContain('60')
    wrapper.unmount()
  })
})
