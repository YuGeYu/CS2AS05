import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'

const tauri = vi.hoisted(() => ({
  check: vi.fn(), discover: vi.fn(), inspect: vi.fn(), diagnostics: vi.fn(), install: vi.fn(), panel: vi.fn(), uninstall: vi.fn(),
}))
vi.mock('@/services/tauri/cs2', () => ({
  checkCs2Process: tauri.check,
  discoverCs2Roots: tauri.discover,
  inspectCs2Root: tauri.inspect,
  getDiagnosticsPayload: tauri.diagnostics,
  installBotPackage: tauri.install,
  openUpstreamPanel: tauri.panel,
  uninstallBotPackage: tauri.uninstall,
}))

import InstallView from '@/views/InstallView.vue'

function mountView() {
  return mount(InstallView, {
    global: {
      plugins: [createPinia()],
      stubs: { SupportActions: true, Teleport: true },
    },
  })
}

describe('CS2 process polling', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    Object.values(tauri).forEach((mock) => mock.mockReset())
    tauri.discover.mockResolvedValue([])
    tauri.check.mockResolvedValue(false)
  })

  it('checks immediately, waits 10 seconds, and tracks process transitions', async () => {
    tauri.check.mockResolvedValueOnce(false).mockResolvedValueOnce(true).mockResolvedValueOnce(false)
    const wrapper = mountView()
    await flushPromises()
    expect(tauri.check).toHaveBeenCalledOnce()
    expect(wrapper.text()).toContain('未运行')
    await vi.advanceTimersByTimeAsync(9_999)
    expect(tauri.check).toHaveBeenCalledOnce()
    await vi.advanceTimersByTimeAsync(1)
    await flushPromises()
    expect(tauri.check).toHaveBeenCalledTimes(2)
    expect(wrapper.text()).toContain('运行中')
    await vi.advanceTimersByTimeAsync(10_000)
    await flushPromises()
    expect(tauri.check).toHaveBeenCalledTimes(3)
    expect(wrapper.text()).toContain('未运行')
    wrapper.unmount()
  })

  it('does not overlap slow checks and stops after unmount', async () => {
    let resolve!: (value: boolean) => void
    tauri.check.mockReturnValue(new Promise<boolean>((done) => { resolve = done }))
    const wrapper = mountView()
    await flushPromises()
    await vi.advanceTimersByTimeAsync(30_000)
    expect(tauri.check).toHaveBeenCalledOnce()
    resolve(false)
    await flushPromises()
    wrapper.unmount()
    await vi.advanceTimersByTimeAsync(10_000)
    expect(tauri.check).toHaveBeenCalledOnce()
  })

  it('checks immediately on focus and visibility without rescanning', async () => {
    const wrapper = mountView()
    await flushPromises()
    expect(tauri.check).toHaveBeenCalledOnce()
    window.dispatchEvent(new Event('focus'))
    await flushPromises()
    expect(tauri.check).toHaveBeenCalledTimes(2)
    document.dispatchEvent(new Event('visibilitychange'))
    await flushPromises()
    expect(tauri.check).toHaveBeenCalledTimes(3)
    expect(tauri.discover).toHaveBeenCalledOnce()
    expect(tauri.inspect).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('shows detection failures instead of reporting CS2 as stopped', async () => {
    tauri.check.mockRejectedValueOnce(new Error('failed'))
    const wrapper = mountView()
    await flushPromises()
    expect(wrapper.text()).toContain('检测失败')
    expect(wrapper.text()).not.toContain('未运行')
    wrapper.unmount()
  })
})
