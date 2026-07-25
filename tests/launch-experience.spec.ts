import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, ref } from 'vue'

const cs2Tauri = vi.hoisted(() => ({ check: vi.fn() }))
const panelTauri = vi.hoisted(() => ({ launch: vi.fn() }))

vi.mock('@/services/tauri/cs2', () => ({
  checkCs2Process: cs2Tauri.check,
  discoverCs2Roots: vi.fn(), inspectCs2Root: vi.fn(), getDiagnosticsPayload: vi.fn(),
  installBotPackage: vi.fn(), openUpstreamPanel: vi.fn(), uninstallBotPackage: vi.fn(),
  guessCs2Roots: vi.fn(), stopGuessCs2Roots: vi.fn(),
}))
vi.mock('@/services/tauri/panel', () => ({
  initializePanelDefaults: vi.fn().mockResolvedValue({ status: 'unchanged', initializedFields: [] }),
  getPanelSnapshot: vi.fn(), launchPanelCs2: panelTauri.launch,
  setPanelMode: vi.fn(), setPanelDifficulty: vi.fn(), setPanelAim: vi.fn(), setPanelNades: vi.fn(),
  setPanelBotItem: vi.fn(), setPanelDropKnives: vi.fn(),
}))

import { useCs2LaunchExperience } from '@/composables/useCs2LaunchExperience'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const Host = defineComponent({
  setup() {
    const root = ref('D:\\Steam\\Counter-Strike Global Offensive')
    return { launch: useCs2LaunchExperience(root) }
  },
  template: '<button @click="launch.start(\'bots\')">launch</button><span>{{ launch.phase }}</span>',
})

describe('CS2 launch experience', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    cs2Tauri.check.mockReset()
    panelTauri.launch.mockReset()
  })

  it('probes serially and closes as soon as cs2.exe is detected', async () => {
    panelTauri.launch.mockResolvedValue({})
    let resolveCheck!: (running: boolean) => void
    cs2Tauri.check.mockReturnValue(new Promise<boolean>((resolve) => { resolveCheck = resolve }))
    const wrapper = mount(Host, { global: { plugins: [createPinia()] } })
    await wrapper.get('button').trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('waitingForProcess')
    expect(cs2Tauri.check).toHaveBeenCalledOnce()
    await vi.advanceTimersByTimeAsync(5_000)
    expect(cs2Tauri.check).toHaveBeenCalledOnce()
    resolveCheck(true)
    await flushPromises()
    expect(useCs2Store().cs2Running).toBe(true)
    expect(wrapper.text()).toContain('idle')
    wrapper.unmount()
  })

  it('closes immediately when the launch command fails and records the Panel error', async () => {
    panelTauri.launch.mockRejectedValue(new Error('Steam not found'))
    const wrapper = mount(Host, { global: { plugins: [createPinia()] } })
    await wrapper.get('button').trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('idle')
    expect(usePanelStore().lastError).toBe('Steam not found')
    expect(cs2Tauri.check).not.toHaveBeenCalled()
    wrapper.unmount()
  })
})
