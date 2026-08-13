// @vitest-environment jsdom
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ discover: vi.fn(), process: vi.fn(), panel: vi.fn(), save: vi.fn(), load: vi.fn(), checkPlugin: vi.fn() }))
vi.mock('@/services/tauri/cs2', () => ({ checkCs2Process: mocks.process, discoverCs2Roots: mocks.discover, inspectCs2Root: vi.fn(), getDiagnosticsPayload: vi.fn(), installBotPackage: vi.fn(), openUpstreamPanel: vi.fn(), uninstallBotPackage: vi.fn() }))
vi.mock('@/services/tauri/panel', () => ({ initializePanelDefaults: vi.fn().mockResolvedValue({ status: 'unchanged', initializedFields: [] }), getPanelSnapshot: mocks.panel, setPanelMode: vi.fn(), setPanelDifficulty: vi.fn(), setPanelAim: vi.fn(), setPanelNades: vi.fn(), setPanelBotItem: vi.fn(), setPanelDropKnives: vi.fn(), launchPanelCs2: vi.fn() }))
vi.mock('@/services/tauri/skinForge', () => ({
  skinForgeLoad: mocks.load,
  skinForgeSave: mocks.save,
  skinForgeCheckPlugin: mocks.checkPlugin,
  skinForgeDeploy: vi.fn(),
  skinForgeReset: vi.fn(),
}))

import AppShell from '@/components/AppShell.vue'
import { useCs2Store } from '@/stores/cs2'
import { useSkinForgeStore } from '@/stores/skinForge'

describe('skin forge navigation guard', () => {
  let pinia: ReturnType<typeof createPinia>
  beforeEach(() => {
    pinia = createPinia()
    setActivePinia(pinia)
    mocks.discover.mockResolvedValue([]); mocks.process.mockResolvedValue(false); mocks.panel.mockResolvedValue(null)
    mocks.load.mockResolvedValue(null); mocks.save.mockResolvedValue({ loadoutPath: 'player_loadout.json', sha256: 'abc', slotCount: 1 })
    mocks.checkPlugin.mockResolvedValue({ allPresent: true, hashMismatches: [] })
  })

  it('requires an explicit decision, then discards by reloading before leaving', async () => {
    const wrapper = mount(AppShell, { attachTo: document.body, global: { plugins: [pinia] } })
    await flushPromises()
    useCs2Store().selectedRoot = 'C:\\CS2'
    await wrapper.get('button[aria-label="皮肤工坊"]').trigger('click')
    useSkinForgeStore().markDirty()
    await wrapper.get('button[aria-label="概览"]').trigger('click')
    expect(document.body.textContent).toContain('还有搭配没有应用')
    expect(wrapper.get('main').attributes('data-current-view')).toBe('skinForge')
    const dialog = document.body.querySelector('.forge-leave-dialog')!
    const discard = [...dialog.querySelectorAll('button')].find(button => button.textContent?.trim() === '不应用并离开') as HTMLButtonElement
    discard.click(); await flushPromises()
    expect(mocks.load).toHaveBeenCalled()
    expect(wrapper.get('main').attributes('data-current-view')).toBe('overview')
    wrapper.unmount()
  })

  it('applies successfully before leaving', async () => {
    const wrapper = mount(AppShell, { attachTo: document.body, global: { plugins: [pinia] } })
    await flushPromises()
    useCs2Store().selectedRoot = 'C:\\CS2'
    await wrapper.get('button[aria-label="皮肤工坊"]').trigger('click')
    useSkinForgeStore().markDirty()
    await wrapper.get('button[aria-label="安装与诊断"]').trigger('click')
    const dialog = document.body.querySelector('.forge-leave-dialog')!
    const apply = [...dialog.querySelectorAll('button')].find(button => button.textContent?.trim() === '应用并离开') as HTMLButtonElement
    apply.click(); await flushPromises()
    expect(mocks.save).toHaveBeenCalledOnce()
    expect(wrapper.get('main').attributes('data-current-view')).toBe('install')
    wrapper.unmount()
  })
})
