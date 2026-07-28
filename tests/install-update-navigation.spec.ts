import { defineComponent, onMounted } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  check: vi.fn(), discover: vi.fn(), process: vi.fn(), panel: vi.fn(),
}))

vi.mock('@/components/SupportActions.vue', () => ({
  default: defineComponent({
    name: 'SupportActions',
    setup() {
      onMounted(() => mocks.check(false))
      return () => null
    },
  }),
}))
vi.mock('@/services/tauri/cs2', () => ({
  checkCs2Process: mocks.process,
  discoverCs2Roots: mocks.discover,
  inspectCs2Root: vi.fn(),
  getDiagnosticsPayload: vi.fn(),
  installBotPackage: vi.fn(),
  openUpstreamPanel: vi.fn(),
  uninstallBotPackage: vi.fn(),
}))
vi.mock('@/services/tauri/panel', () => ({
  initializePanelDefaults: vi.fn().mockResolvedValue({ status: 'unchanged', initializedFields: [] }),
  getPanelSnapshot: mocks.panel,
  setPanelMode: vi.fn(), setPanelDifficulty: vi.fn(), setPanelAim: vi.fn(), setPanelNades: vi.fn(),
  setPanelBotItem: vi.fn(), setPanelDropKnives: vi.fn(), launchPanelCs2: vi.fn(),
}))

import AppShell from '@/components/AppShell.vue'

describe('installation update check navigation', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    Object.values(mocks).forEach(mock => mock.mockReset())
    mocks.discover.mockResolvedValue([])
    mocks.process.mockResolvedValue(false)
    mocks.panel.mockResolvedValue(null)
  })

  afterEach(() => {
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  it('checks on every installation-page entry but not on overview', async () => {
    const wrapper = mount(AppShell, { global: { plugins: [createPinia()] } })
    await flushPromises()
    expect(mocks.check).not.toHaveBeenCalled()

    await wrapper.get('button[aria-label="安装与诊断"]').trigger('click')
    await vi.advanceTimersByTimeAsync(220)
    expect(mocks.check).toHaveBeenCalledTimes(1)
    expect(mocks.check).toHaveBeenLastCalledWith(false)

    await wrapper.get('button[aria-label="概览"]').trigger('click')
    await vi.advanceTimersByTimeAsync(220)
    await wrapper.get('button[aria-label="安装与诊断"]').trigger('click')
    await vi.advanceTimersByTimeAsync(220)
    expect(mocks.check).toHaveBeenCalledTimes(2)
    expect(mocks.check).toHaveBeenLastCalledWith(false)
    wrapper.unmount()
  })
})
