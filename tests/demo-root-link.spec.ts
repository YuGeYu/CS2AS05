import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const mocks = vi.hoisted(() => ({ inspect: vi.fn(), ensure: vi.fn() }))

vi.mock('@/services/tauri/cs2', () => ({
  checkCs2Process: vi.fn(),
  discoverCs2Roots: vi.fn(),
  getDiagnosticsPayload: vi.fn(),
  guessCs2Roots: vi.fn(),
  inspectCs2Root: mocks.inspect,
  installBotPackage: vi.fn(),
  openUpstreamPanel: vi.fn(),
  stopGuessCs2Roots: vi.fn(),
  uninstallBotPackage: vi.fn(),
}))

vi.mock('@/services/tauri/demo', () => ({
  ensureDefaultDemoRoot: mocks.ensure,
}))

import { useCs2Store } from '@/stores/cs2'

describe('selected CS2 root demo linkage', () => {
  beforeEach(() => {
    const values = new Map<string, string>()
    const localStorage = {
      clear: () => values.clear(),
      getItem: (key: string) => values.get(key) ?? null,
      key: (index: number) => [...values.keys()][index] ?? null,
      get length() { return values.size },
      removeItem: (key: string) => values.delete(key),
      setItem: (key: string, value: string) => values.set(key, value),
    } satisfies Storage
    vi.stubGlobal('window', { localStorage })
    setActivePinia(createPinia())
    mocks.inspect.mockReset()
    mocks.ensure.mockReset()
  })

  it('registers the verified canonical CS2 root before persisting the selection', async () => {
    const canonical = 'D:\\SteamLibrary\\steamapps\\common\\Counter-Strike Global Offensive'
    mocks.inspect.mockResolvedValue({ rootPath: canonical })
    mocks.ensure.mockResolvedValue({ path: canonical, scanDepth: 5, origin: 'selected_cs2_root' })

    const store = useCs2Store()
    await store.selectRoot('D:\\SteamLibrary\\steamapps\\common\\Counter-Strike Global Offensive\\game\\csgo')

    expect(mocks.ensure).toHaveBeenCalledExactlyOnceWith(canonical)
    expect(store.selectedRoot).toBe(canonical)
    expect(window.localStorage.getItem('cs2-bot-improver.selected-root.v1')).toBe(canonical)
  })
})
