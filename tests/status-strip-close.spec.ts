import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'

const api = vi.hoisted(() => ({
  close: vi.fn(),
  check: vi.fn(),
  snapshot: vi.fn(),
}))

vi.mock('@/services/tauri/cs2', () => ({
  closeCs2: api.close,
  checkCs2Process: api.check,
  getCs2ProcessSnapshot: api.snapshot,
}))

import StatusStrip from '@/components/StatusStrip.vue'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

function mountStrip(state: 'checking' | 'running' | 'stopped' | 'unknown') {
  const pinia = createPinia()
  setActivePinia(pinia)
  const cs2 = useCs2Store()
  const panel = usePanelStore()
  cs2.cs2ProcessState = state
  cs2.selectedRoot = 'C:/CS2'
  panel.snapshot = { ready: true, missingFiles: [], cs2Running: state === 'running' } as never
  return mount(StatusStrip, { global: { plugins: [pinia] } })
}

describe('StatusStrip CS2 close control', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    api.close.mockResolvedValue({ success: true, message: 'CS2 已关闭。' })
    api.snapshot.mockResolvedValue({ processes: [], confidence: 'high', sampleCount: 2, observedAt: Date.now() })
    api.check.mockResolvedValue(false)
  })

  it.each([
    ['checking', true],
    ['stopped', true],
    ['unknown', true],
    ['running', false],
  ] as const)('状态 %s 时按契约设置关闭按钮禁用态', (state, disabled) => {
    const wrapper = mountStrip(state)
    const button = wrapper.get('button.status-strip-close').element as HTMLButtonElement
    expect(button.disabled).toBe(disabled)
    expect(button.getAttribute('aria-label')).toBe('关闭 CS2')
  })

  it('优雅关闭失败时显示应用内强制确认，并只在确认后调用 force=true', async () => {
    api.close.mockResolvedValueOnce({ success: false, message: 'CS2 仍在运行，可能未响应；确认后可强制关闭。' })
    const wrapper = mountStrip('running')
    await wrapper.get('button.status-strip-close').trigger('click')
    await flushPromises()
    expect(api.close).toHaveBeenCalledWith(false)
    expect(wrapper.text()).toContain('强制关闭可能丢失未保存内容')
    expect(wrapper.text()).toContain('确认强制关闭')
    await wrapper.get('.status-strip-confirm button').trigger('click')
    await flushPromises()
    expect(api.close).toHaveBeenNthCalledWith(2, true)
  })

  it('关闭期间锁定按钮，避免重复请求', async () => {
    let resolve!: (value: { success: boolean; message: string }) => void
    api.close.mockReturnValueOnce(new Promise(done => { resolve = done }))
    const wrapper = mountStrip('running')
    const button = wrapper.get('button.status-strip-close')
    await button.trigger('click')
    await flushPromises()
    expect((button.element as HTMLButtonElement).disabled).toBe(true)
    await button.trigger('click')
    expect(api.close).toHaveBeenCalledOnce()
    resolve({ success: true, message: 'CS2 已关闭。' })
    await flushPromises()
  })
})
