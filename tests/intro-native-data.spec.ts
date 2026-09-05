// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true, invoke }))

import { FALLBACK_UPSTREAM, loadIntroData } from '@/services/intro-data'

describe('native intro data provider', () => {
  beforeEach(() => {
    invoke.mockReset()
    const values = new Map<string, string>()
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
      clear: () => values.clear(),
    })
  })

  it('uses build-time data without invoking native sponsor loading', async () => {
    const result = await loadIntroData()
    expect(invoke).not.toHaveBeenCalled()
    expect(result.sources).toEqual({ supporters: 'fallback', upstream: 'fallback' })
    expect(result.supporters).toHaveLength(6)
  })

  it('falls back to cache when the native command rejects', async () => {
    localStorage.setItem('cs2as:intro:supporters:v1', JSON.stringify({ savedAt: Date.now(), value: { supporters: [{ id: 'cached', nickname: '缓存', amountCents: 1 }] } }))
    const result = await loadIntroData()
    expect(result.sources.supporters).toBe('fallback')
    expect(result.supporters).toHaveLength(6)
    expect(result.sources.upstream).toBe('fallback')
  })

  it('does not trust malformed native records', async () => {
    const result = await loadIntroData()
    expect(result.supporters).toHaveLength(6)
    expect(result.sources.supporters).toBe('fallback')
  })
})
