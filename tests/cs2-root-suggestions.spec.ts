import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'

const cs2Tauri = vi.hoisted(() => ({ guess: vi.fn(), stop: vi.fn(), inspect: vi.fn(), discover: vi.fn() }))
const panelTauri = vi.hoisted(() => ({ snapshot: vi.fn() }))
const demoTauri = vi.hoisted(() => ({ ensureRoot: vi.fn() }))

vi.mock('@/services/tauri/cs2', () => ({
  checkCs2Process: vi.fn(), discoverCs2Roots: cs2Tauri.discover, inspectCs2Root: cs2Tauri.inspect,
  getDiagnosticsPayload: vi.fn(), installBotPackage: vi.fn(), openUpstreamPanel: vi.fn(), uninstallBotPackage: vi.fn(),
  guessCs2Roots: cs2Tauri.guess, stopGuessCs2Roots: cs2Tauri.stop,
}))
vi.mock('@/services/tauri/panel', () => ({
  initializePanelDefaults: vi.fn().mockResolvedValue({ status: 'unchanged', initializedFields: [] }),
  getPanelSnapshot: panelTauri.snapshot, launchPanelCs2: vi.fn(),
  setPanelMode: vi.fn(), setPanelDifficulty: vi.fn(), setPanelAim: vi.fn(), setPanelNades: vi.fn(),
  setPanelBotItem: vi.fn(), setPanelDropKnives: vi.fn(),
}))
vi.mock('@/services/tauri/demo', () => ({
  ensureDefaultDemoRoot: demoTauri.ensureRoot,
}))

import Cs2RootSuggestionsDialog from '@/components/Cs2RootSuggestionsDialog.vue'

const candidate = { path: 'D:\\Steam\\Counter-Strike Global Offensive', source: 'Steam App 730 清单', confidence: 'verified' as const, evidence: ['gameinfo.gi'] }

describe('CS2 root suggestions', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    cs2Tauri.guess.mockReset()
    cs2Tauri.stop.mockReset()
    cs2Tauri.inspect.mockReset()
    cs2Tauri.discover.mockReset()
    panelTauri.snapshot.mockReset()
    demoTauri.ensureRoot.mockReset()
    cs2Tauri.inspect.mockResolvedValue({ rootPath: candidate.path })
    cs2Tauri.discover.mockResolvedValue([])
    panelTauri.snapshot.mockResolvedValue(null)
    demoTauri.ensureRoot.mockResolvedValue({ path: candidate.path, scanDepth: 5, origin: 'selected_cs2_root' })
  })

  it('stops the active scan before applying the selected candidate', async () => {
    let complete!: (value: unknown) => void
    cs2Tauri.guess.mockImplementation((onEvent: (event: unknown) => void) => {
      onEvent({ kind: 'candidate', elapsedMs: 10, checkedLocations: 1, candidate })
      return new Promise((resolve) => { complete = resolve })
    })
    cs2Tauri.stop.mockImplementation(async () => {
      complete({ candidates: [candidate], elapsedMs: 11, checkedLocations: 1, stopReason: 'userStopped', warnings: [] })
      return true
    })
    const wrapper = mount(Cs2RootSuggestionsDialog, { props: { open: false }, global: { plugins: [createPinia()], stubs: { Teleport: true } } })
    await wrapper.setProps({ open: true })
    await flushPromises()
    await wrapper.get('.root-suggestions__list .secondary-button').trigger('click')
    await flushPromises()
    expect(cs2Tauri.stop).toHaveBeenCalledOnce()
    expect(cs2Tauri.inspect).toHaveBeenCalledWith(candidate.path)
    expect(wrapper.emitted('close')).toHaveLength(1)
    wrapper.unmount()
  })

  it('keeps a valid saved root from producing a false not-found warning', async () => {
    const { useCs2Store } = await import('@/stores/cs2')
    const store = useCs2Store()
    cs2Tauri.inspect.mockResolvedValue({ rootPath: candidate.path, gameDirExists: true, csgoDirExists: true })
    await store.selectRoot(candidate.path)
    await store.scanRoots()
    expect(store.message).toEqual({ tone: 'ready', title: '目录已确认', message: '已确认当前保存的 CS2 游戏目录可用。' })
  })
})
