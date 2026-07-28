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

  it('uses invoke data, validates it, and caches independent endpoint successes', async () => {
    invoke.mockResolvedValue({
      supporters: [{ id: 's1', nickname: '同路人', message: '同行', amountCents: 2000, sortOrder: 0, isVisible: true, createdAt: '', updatedAt: '' }],
      upstream: FALLBACK_UPSTREAM,
      sources: { supporters: 'network', upstream: 'network' },
      diagnostics: { supporters: 'ok', upstream: 'ok' },
    })
    const result = await loadIntroData()
    expect(invoke).toHaveBeenCalledWith('get_intro_public_data')
    expect(result.sources).toEqual({ supporters: 'network', upstream: 'network' })
    expect(result.supporters[0]?.nickname).toBe('同路人')
    expect(localStorage.getItem('cs2as:intro:supporters:v1')).toContain('同路人')
  })

  it('falls back to cache when the native command rejects', async () => {
    localStorage.setItem('cs2as:intro:supporters:v1', JSON.stringify({ savedAt: Date.now(), value: { supporters: [{ id: 'cached', nickname: '缓存', amountCents: 1 }] } }))
    invoke.mockRejectedValue(new Error('offline'))
    const result = await loadIntroData()
    expect(result.sources.supporters).toBe('cache')
    expect(result.supporters[0]?.nickname).toBe('缓存')
    expect(result.sources.upstream).toBe('fallback')
  })

  it('does not trust malformed native records', async () => {
    invoke.mockResolvedValue({
      supporters: [{ id: 'bad', nickname: '无效', amountCents: -1 }],
      upstream: FALLBACK_UPSTREAM,
      sources: { supporters: 'network', upstream: 'fallback' },
      diagnostics: { supporters: 'ok', upstream: 'network' },
    })
    const result = await loadIntroData()
    expect(result.supporters).toEqual([])
    expect(result.sources.supporters).toBe('network')
  })
})
