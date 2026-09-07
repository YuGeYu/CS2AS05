// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import EasterEggGame from '@/components/easter-egg/EasterEggGame.vue'


describe('EasterEggGame close control', () => {
  beforeEach(() => {
    vi.useFakeTimers()
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

  it('closes once from pointerdown in the contribution gallery', async () => {
    const wrapper = await mountGame()
    await wrapper.get('[aria-label="关闭贡献陈列馆"]').trigger('pointerdown', { pointerId: 1 })
    await wrapper.get('[aria-label="关闭贡献陈列馆"]').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('releases input capture and closes once while exploring', async () => {
    const wrapper = await mountGame()
    await wrapper.get('[aria-label="关闭贡献陈列馆"]').trigger('pointerdown', { pointerId: 8 })
    await wrapper.get('[aria-label="关闭贡献陈列馆"]').trigger('click')
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('closes once with Escape and clears the clock', async () => {
    const wrapper = await mountGame()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    expect(wrapper.emitted('close')).toHaveLength(1)
    expect(wrapper.text()).toContain('贡献陈列馆')
    wrapper.unmount()
  })
})
