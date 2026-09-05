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

  it('uses build-time data without public endpoint requests', async () => {
    const fetch = vi.fn()
    vi.stubGlobal('fetch', fetch)
    const result = await loadIntroData()
    expect(result.supporters).toHaveLength(6)
    expect(result.upstream.fullName).toBe(FALLBACK_UPSTREAM.fullName)
    expect(result.sources).toEqual({ supporters: 'fallback', upstream: 'fallback' })
    expect(fetch).not.toHaveBeenCalled()
  })

  it('ignores legacy cached supporter records', async () => {
    localStorage.setItem('cs2as:intro:supporters:v1', JSON.stringify({ savedAt: Date.now(), value: { supporters: [{ id: 'old', nickname: '旧记录', amountCents: 100 }] } }))
    const result = await loadIntroData()
    expect(result.supporters).toHaveLength(6)
    expect(result.sources.supporters).toBe('fallback')
  })

  it('ignores browser cache in favor of the build-time archive', () => {
    localStorage.setItem('cs2as:intro:supporters:v1', JSON.stringify({ savedAt: Date.now(), value: { supporters: [{ id: 'cached', nickname: '缓存同路人', message: '先亮馆，再刷新', amountCents: 600, updatedAt: 'now' }] } }))
    const result = loadCachedIntroData()
    expect(result.supporters).toHaveLength(6)
    expect(result.sources.supporters).toBe('fallback')
  })
})
