import { beforeEach, describe, expect, it, vi } from 'vitest'
import { FALLBACK_UPSTREAM, loadCachedIntroData, loadIntroData, parseSupporters, parseUpstream } from '@/services/intro-data'

describe('intro data', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    const values = new Map<string, string>()
    vi.stubGlobal('localStorage', {
      getItem: (key: string) => values.get(key) ?? null,
      setItem: (key: string, value: string) => values.set(key, value),
      removeItem: (key: string) => values.delete(key),
      clear: () => values.clear(),
    })
  })

  it('validates and truncates supporter data', () => {
    const rows = Array.from({ length: 55 }, (_, index) => ({ id: String(index), nickname: '旅人', message: '同行', amountCents: 2000, sortOrder: index, isVisible: true, createdAt: '', updatedAt: '' }))
    expect(parseSupporters({ supporters: rows })).toHaveLength(50)
    expect(parseSupporters({ supporters: [{ id: 'bad', amountCents: -1 }] })).toEqual([])
  })

  it('normalizes optional upstream fields without fake statistics', () => {
    expect(parseUpstream({ full_name: 'ed0ard/CS2-Bot-Improver', description: null, html_url: FALLBACK_UPSTREAM.url, license: null })).toMatchObject({ stars: null, forks: null, license: 'AGPL-3.0' })
  })

  it('falls back when both public endpoints fail', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')))
    const result = await loadIntroData()
    expect(result.supporters).toEqual([])
    expect(result.upstream).toEqual(FALLBACK_UPSTREAM)
    expect(result.sources).toEqual({ supporters: 'fallback', upstream: 'fallback' })
  })

  it('treats an empty network list as authoritative and replaces cached names', async () => {
    localStorage.setItem('cs2as:intro:supporters:v1', JSON.stringify({ savedAt: Date.now(), value: { supporters: [{ id: 'old', nickname: '旧记录', amountCents: 100 }] } }))
    vi.stubGlobal('fetch', vi.fn()
      .mockResolvedValueOnce(new Response(JSON.stringify({ supporters: [] }), { status: 200 }))
      .mockResolvedValueOnce(new Response(JSON.stringify({ full_name: 'ed0ard/CS2-Bot-Improver', html_url: FALLBACK_UPSTREAM.url }), { status: 200 })))
    const result = await loadIntroData()
    expect(result.supporters).toEqual([])
    expect(result.sources.supporters).toBe('network')
    expect(localStorage.getItem('cs2as:intro:supporters:v1')).not.toContain('旧记录')
  })

  it('returns cached supporters synchronously before network refresh', () => {
    localStorage.setItem('cs2as:intro:supporters:v1', JSON.stringify({ savedAt: Date.now(), value: { supporters: [{ id: 'cached', nickname: '缓存同路人', message: '先亮馆，再刷新', amountCents: 600, updatedAt: 'now' }] } }))
    const result = loadCachedIntroData()
    expect(result.supporters).toHaveLength(1)
    expect(result.supporters[0]?.nickname).toBe('缓存同路人')
    expect(result.sources.supporters).toBe('cache')
  })
})
