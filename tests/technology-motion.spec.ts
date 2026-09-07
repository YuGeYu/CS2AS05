import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import type { PanelSnapshot } from '@/features/panel/types'

const tauri = vi.hoisted(() => ({
  check: vi.fn(), discover: vi.fn(), inspect: vi.fn(), initialize: vi.fn(), snapshot: vi.fn(),
  setKnives: vi.fn(), setNades: vi.fn(), setBotItem: vi.fn(), writeText: vi.fn(),
}))

vi.mock('@/services/tauri/cs2', () => ({
  checkCs2Process: tauri.check, discoverCs2Roots: tauri.discover, inspectCs2Root: tauri.inspect,
  getDiagnosticsPayload: vi.fn(), installBotPackage: vi.fn(), openUpstreamPanel: vi.fn(), uninstallBotPackage: vi.fn(),
  guessCs2Roots: vi.fn(), stopGuessCs2Roots: vi.fn(),
}))
vi.mock('@/services/tauri/panel', () => ({
  initializePanelDefaults: tauri.initialize, getPanelSnapshot: tauri.snapshot,
  setPanelDropKnives: tauri.setKnives, setPanelMode: vi.fn(), setPanelDifficulty: vi.fn(),
  setPanelAim: vi.fn(), setPanelNades: tauri.setNades, setPanelBotItem: tauri.setBotItem, launchPanelCs2: vi.fn(),
}))
vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({ writeText: tauri.writeText }))

import AppShell from '@/components/AppShell.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import { parseCommands } from '@/data/panel/commands'
import { KNIVES } from '@/data/panel/knives'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import CommandsView from '@/views/CommandsView.vue'
import BotItemsView from '@/views/BotItemsView.vue'
import KnivesView from '@/views/KnivesView.vue'
import PresetsView from '@/views/PresetsView.vue'

function snapshot(selected: number[] = []): PanelSnapshot {
  return {
    rootPath: 'root', ready: true, missingFiles: [], cs2Running: false,
    mode: { current: 'bots', insecure: true, writable: true },
    difficulty: { current: 'Low', available: ['Low', 'Medium', 'High'] },
    presets: { aim: 'mixed', nades: 'normal', writable: true },
    botItems: { profiles: true, agents: true, music: true, weapons: true, knives: true, gloves: true, stickers: true, charms: true, writable: true },
    dropKnives: { bindKey: '\\', selected, writable: true },
  }
}

describe('technology motion interaction boundaries', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    window.localStorage?.clear()
    setActivePinia(createPinia())
    Object.values(tauri).forEach(mock => mock.mockReset())
    tauri.check.mockResolvedValue(false)
    tauri.discover.mockResolvedValue([])
    tauri.initialize.mockResolvedValue({ status: 'unchanged', initializedFields: [] })
    tauri.snapshot.mockResolvedValue(null)
  })

  afterEach(() => {
    vi.clearAllTimers()
    vi.useRealTimers()
  })

  it('keeps exactly one authoritative current page with the active style', async () => {
    const wrapper = mount(AppShell, {
      global: { plugins: [createPinia()], stubs: { OverviewView: true, PresetsView: true, BotItemsView: true, KnivesView: true, CommandsView: true, InstallView: true } },
    })
    await flushPromises()
    const overviewActive = wrapper.get('[aria-current="page"]')
    expect(overviewActive.attributes('aria-label')).toBe('概览')
    expect(overviewActive.classes()).toContain('is-active')
    expect(wrapper.findAll('[aria-current="page"]')).toHaveLength(1)
    expect(wrapper.get('main').attributes('data-current-view')).toBe('overview')

    await wrapper.get('button[aria-label="刀具"]').trigger('click')
    await vi.advanceTimersByTimeAsync(220)
    expect(wrapper.get('[aria-current="page"]').attributes('aria-label')).toBe('刀具')
    expect(wrapper.findAll('[aria-current="page"]')).toHaveLength(1)
    expect(wrapper.get('main').attributes('data-current-view')).toBe('knives')
    expect(wrapper.get('.view-swap-frame').element.childElementCount).toBeGreaterThan(0)
    await wrapper.get('button[aria-label="刀具"]').trigger('click')
    expect(wrapper.get('[aria-current="page"]').attributes('aria-label')).toBe('刀具')
    wrapper.unmount()
  })

  it('keeps the segmented indicator on the confirmed prop and blocks pending input', async () => {
    const options = [{ value: 'a', label: 'A' }, { value: 'b', label: 'B' }, { value: 'c', label: 'C' }] as const
    const wrapper = mount(SegmentedControl, { props: { modelValue: null, options, label: 'test' } })
    expect(wrapper.find('.segmented__indicator').exists()).toBe(false)
    expect(wrapper.get('.segmented').attributes('style')).toContain('--segment-count: 3')

    await wrapper.setProps({ modelValue: 'a' })
    await wrapper.findAll('button')[1]?.trigger('click')
    expect(wrapper.emitted('update:modelValue')?.at(-1)).toEqual(['b'])
    expect(wrapper.get('.segmented').attributes('style')).toContain('--segment-index: 0')
    await wrapper.setProps({ modelValue: 'b' })
    expect(wrapper.get('.segmented').attributes('style')).toContain('--segment-index: 1')

    await wrapper.setProps({ pending: true })
    expect(wrapper.get('.segmented').attributes('aria-busy')).toBe('true')
    expect(wrapper.findAll('button').every(button => button.attributes('disabled') !== undefined)).toBe(true)
    await wrapper.setProps({ pending: false, options: options.slice(0, 2), modelValue: 'b' })
    expect(wrapper.get('.segmented').attributes('style')).toContain('--segment-count: 2')
    await wrapper.setProps({ options: [...options, { value: 'd', label: 'D' }] })
    expect(wrapper.get('.segmented').attributes('style')).toContain('--segment-count: 4')
    wrapper.unmount()
  })

  it('animates a toggle only after false changes to true and clears its timer', async () => {
    const wrapper = mount(ToggleSwitch, { props: { modelValue: true, label: '皮肤' } })
    expect(wrapper.classes()).not.toContain('is-just-enabled')
    await wrapper.setProps({ modelValue: false })
    expect(wrapper.classes()).not.toContain('is-just-enabled')
    await wrapper.setProps({ modelValue: true })
    expect(wrapper.classes()).toContain('is-just-enabled')
    await vi.advanceTimersByTimeAsync(420)
    expect(wrapper.classes()).not.toContain('is-just-enabled')
    await wrapper.setProps({ modelValue: false })
    await wrapper.setProps({ modelValue: true })
    wrapper.unmount()
    expect(vi.getTimerCount()).toBe(0)
  })

  it('shows single-knife feedback only after a successful authoritative mutation', async () => {
    const pinia = createPinia(); setActivePinia(pinia)
    useCs2Store().selectedRoot = 'root'
    const panel = usePanelStore(); panel.snapshot = snapshot()
    tauri.setKnives.mockResolvedValue(snapshot([500]))
    const wrapper = mount(KnivesView, { global: { plugins: [pinia] } })
    await wrapper.get('[aria-label="刀具多选"] button').trigger('click')
    await flushPromises()
    expect(wrapper.get('[aria-label="刀具多选"] button').classes()).toContain('is-just-selected')
    expect(wrapper.get('[aria-label="刀具多选"] button').attributes('aria-pressed')).toBe('true')
    await vi.advanceTimersByTimeAsync(450)
    expect(wrapper.get('[aria-label="刀具多选"] button').classes()).not.toContain('is-just-selected')

    tauri.setKnives.mockResolvedValueOnce(snapshot())
    await wrapper.get('[aria-label="刀具多选"] button').trigger('click')
    await flushPromises()
    expect(wrapper.findAll('.is-just-selected')).toHaveLength(0)

    tauri.setKnives.mockRejectedValueOnce(new Error('write failed'))
    await wrapper.findAll('[aria-label="刀具多选"] button')[1]?.trigger('click')
    await flushPromises()
    expect(wrapper.findAll('.is-just-selected')).toHaveLength(0)
    expect(wrapper.get('[role="alert"]').text()).toContain('write failed')
    wrapper.unmount()
  })

  it('summarizes bulk selection without animating all knife cards', async () => {
    const pinia = createPinia(); setActivePinia(pinia)
    useCs2Store().selectedRoot = 'root'
    const panel = usePanelStore(); panel.snapshot = snapshot()
    tauri.setKnives.mockResolvedValue(snapshot(KNIVES.map(knife => knife.id)))
    const wrapper = mount(KnivesView, { global: { plugins: [pinia] } })
    const selectAll = wrapper.findAll('.selection-actions button').find(button => button.text() === '全选')
    await selectAll?.trigger('click'); await flushPromises()
    expect(wrapper.findAll('.knife-grid .is-just-selected')).toHaveLength(0)
    expect(wrapper.get('[aria-live="polite"]').text()).toBe('已选择全部 20 款刀具')
    expect(wrapper.get('.selection-actions').classes()).toContain('is-confirmed')
    wrapper.unmount()
  })

  it('shows copy confirmation only after Tauri succeeds and preserves trailing spaces', async () => {
    tauri.writeText.mockResolvedValue(undefined)
    const wrapper = mount(CommandsView)
    const trailing = parseCommands().find(entry => entry.copyable && entry.copy.endsWith(' '))
    expect(trailing).toBeDefined()
    const button = wrapper.get(`[data-command-id="${trailing?.id}"]`)
    await button.trigger('click'); await flushPromises()
    expect(tauri.writeText).toHaveBeenCalledWith(trailing?.copy)
    expect(button.attributes('data-copied')).toBe('true')
    expect(wrapper.get('.command-result').text()).toContain('保留尾随空格')
    expect(button.get('svg').attributes('class')).toContain('check')

    tauri.writeText.mockRejectedValueOnce(new Error('clipboard unavailable'))
    const other = wrapper.findAll('.command-list button').find(candidate => candidate.attributes('data-command-id') !== String(trailing?.id))
    await other?.trigger('click'); await flushPromises()
    expect(wrapper.find('[data-copied="true"]').exists()).toBe(false)
    expect(wrapper.get('[role="alert"]').text()).toBe('复制失败，请重试。')
    wrapper.unmount()
    expect(vi.getTimerCount()).toBe(0)
  })

  it('searches and copies the single exact br_reroll command through Tauri', async () => {
    tauri.writeText.mockResolvedValue(undefined)
    const wrapper = mount(CommandsView)
    await wrapper.get('input').setValue('br_reroll')
    const matches = wrapper.findAll('.command-list button.is-match')
    expect(matches).toHaveLength(1)
    expect(matches[0]?.text()).toBe('br_reroll')
    await matches[0]?.trigger('click')
    await flushPromises()
    expect(tauri.writeText).toHaveBeenCalledWith('br_reroll')
    expect(matches[0]?.attributes('data-copied')).toBe('true')
    wrapper.unmount()
  })

  it('keeps all five Nades choices and commits less only after backend success', async () => {
    const pinia = createPinia(); setActivePinia(pinia)
    useCs2Store().selectedRoot = 'root'
    const panel = usePanelStore(); panel.snapshot = snapshot()
    tauri.setNades.mockResolvedValue({ ...snapshot(), presets: { aim: 'mixed', nades: 'less', writable: true } })
    const wrapper = mount(PresetsView, { global: { plugins: [pinia] } })
    const nades = wrapper.findAll('.control-group')[1]!
    expect(nades.findAll('.segmented button').map(button => button.text())).toEqual(['最多', '较多', '正常', '较少', '关闭'])
    await nades.findAll('.segmented button')[3]?.trigger('click')
    await flushPromises()
    expect(tauri.setNades).toHaveBeenCalledWith('root', 'less')
    expect(panel.snapshot?.presets.nades).toBe('less')

    tauri.setNades.mockRejectedValueOnce(new Error('nades write failed'))
    await nades.findAll('.segmented button')[4]?.trigger('click')
    await flushPromises()
    expect(panel.snapshot?.presets.nades).toBe('less')
    expect(panel.lastError).toBe('nades write failed')
    wrapper.unmount()
  })

  it('renders all eight Bot Items and rolls visible state back on write failure', async () => {
    const pinia = createPinia(); setActivePinia(pinia)
    useCs2Store().selectedRoot = 'root'
    const panel = usePanelStore(); panel.snapshot = snapshot()
    tauri.setBotItem.mockRejectedValueOnce(new Error('item write failed'))
    const wrapper = mount(BotItemsView, { global: { plugins: [pinia] } })
    expect(wrapper.findAll('.toggle-row')).toHaveLength(8)
    const first = wrapper.findAll('input[type="checkbox"]')[0]!
    expect((first.element as HTMLInputElement).checked).toBe(true)
    await first.setValue(false)
    await flushPromises()
    expect(panel.snapshot?.botItems.profiles).toBe(true)
    expect((first.element as HTMLInputElement).checked).toBe(true)
    expect(wrapper.get('[role="alert"]').text()).toContain('item write failed')
    wrapper.unmount()
  })
})
