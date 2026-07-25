import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { initializePanelDefaults, setPanelBotItem, setPanelDropKnives, setPanelMode } from '@/services/tauri/panel'

describe('Panel invoke contract', () => {
  beforeEach(() => invoke.mockReset())

  it('uses camelCase Tauri arguments for mode', async () => {
    invoke.mockResolvedValue({})
    await setPanelMode('D:\\Steam\\CS2', 'bots')
    expect(invoke).toHaveBeenCalledWith('set_panel_mode', { rootPath: 'D:\\Steam\\CS2', mode: 'bots' })
  })

  it('initializes disk defaults before the first snapshot', async () => {
    invoke.mockResolvedValue({ status: 'unchanged', initializedFields: [] })
    await initializePanelDefaults('D:\\Steam\\CS2')
    expect(invoke).toHaveBeenCalledWith('initialize_panel_defaults', { rootPath: 'D:\\Steam\\CS2' })
  })

  it('sends independent bot item values', async () => {
    invoke.mockResolvedValue({})
    await setPanelBotItem('root', 'music', true)
    expect(invoke).toHaveBeenCalledWith('set_panel_bot_item', { rootPath: 'root', item: 'music', enabled: true })
  })

  it('sends the captured bind and selected subclasses', async () => {
    invoke.mockResolvedValue({})
    await setPanelDropKnives('root', 'f8', [500, 526])
    expect(invoke).toHaveBeenCalledWith('set_panel_drop_knives', { rootPath: 'root', bindKey: 'f8', selected: [500, 526] })
  })
})
