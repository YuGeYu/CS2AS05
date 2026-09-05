import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const panelApi = vi.hoisted(() => ({
  initialize: vi.fn(),
  snapshot: vi.fn(),
}))

vi.mock('@/services/tauri/panel', () => ({
  initializePanelDefaults: panelApi.initialize,
  getPanelSnapshot: panelApi.snapshot,
  launchPanelCs2: vi.fn(),
  setPanelAim: vi.fn(),
  setPanelBotItem: vi.fn(),
  setPanelDifficulty: vi.fn(),
  setPanelDropKnives: vi.fn(),
  setPanelMode: vi.fn(),
  setPanelNades: vi.fn(),
}))

import { usePanelStore } from '@/stores/panel'

const read = (path: string) => readFileSync(resolve(process.cwd(), path), 'utf8')

describe('0.5.5 release blocker recovery', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    panelApi.initialize.mockReset().mockResolvedValue({ status: 'unchanged', initializedFields: [] })
    panelApi.snapshot.mockReset().mockResolvedValue({ ready: true })
  })

  it('does not touch managed panel files before the plugin environment exists', async () => {
    const panel = usePanelStore()
    await panel.refresh('D:\\SteamLibrary\\Counter-Strike Global Offensive', false, false)
    expect(panelApi.initialize).not.toHaveBeenCalled()
    expect(panelApi.snapshot).not.toHaveBeenCalled()
    expect(panel.snapshot).toBeNull()
    expect(panel.lastError).toBe('')
  })

  it('preloads a hidden scoreboard and forbids synchronous dynamic window construction', () => {
    const command = read('src-tauri/src/commands/scoreboard.rs')
    const config = JSON.parse(read('src-tauri/tauri.conf.json'))
    const scoreboard = config.app.windows.find((window: { label: string }) => window.label === 'scoreboard')
    expect(scoreboard).toMatchObject({ url: 'scoreboard.html', visible: false, decorations: true })
    expect(command).not.toContain('WebviewWindowBuilder')
    expect(command).not.toContain('WebviewUrl::App')
    expect(command).toContain('scoreboard_frontend_ready')
    expect(command).toContain('scoreboard_present')
    expect(command).toContain('demo::presentable_report(&app, report_id)')
    expect(read('src/ScoreboardApp.vue')).toContain("next.metricsVersion !== 'lb-rating-2.0'")
    expect(command).toContain('emit_to("scoreboard"')
  })

  it('keeps a non-white boot fallback and actionable main-window failure reporting', () => {
    expect(read('scoreboard.html')).toContain('boot-fallback')
    expect(read('scoreboard.html')).toContain('正在载入本局战报')
    expect(read('src/scoreboard.ts')).toContain('unhandledrejection')
    expect(read('src/ScoreboardApp.vue')).toContain("listen<PendingReport>('scoreboard://load-report'")
    expect(read('src/components/AppShell.vue')).toContain("listen<string>('scoreboard://boot-error'")
  })

  it('stores the selected CS2 demo root at game/csgo with bounded depth', () => {
    const demo = read('src-tauri/src/services/demo.rs')
    expect(demo).toContain('root.join("game").join("csgo")')
    expect(demo).toContain("VALUES(?1,?2,1,1,?3,'selected_cs2_root')")
    expect(read('src/views/DemoReviewView.vue')).toContain('通常无需手动添加')
  })
})
