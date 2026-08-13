import type { IntroData, SupporterAcknowledgement, UpstreamProjectSummary } from '@/features/intro/types'
import { isTauri } from '@tauri-apps/api/core'
import { getIntroPublicData } from '@/services/tauri/intro'

const SUPPORTERS_URL = 'https://cs2as.600318.xyz/api/supporters'
const UPSTREAM_URL = 'https://api.github.com/repos/ed0ard/CS2-Bot-Improver'
const SUPPORTERS_CACHE = 'cs2as:intro:supporters:v1'
const UPSTREAM_CACHE = 'cs2as:intro:upstream:v1'
const FALLBACK_UPSTREAM: UpstreamProjectSummary = {
  fullName: 'ed0ard/CS2-Bot-Improver',
  description: 'CS2 Bot 行为增强项目，本助手基于其能力构建。',
  url: 'https://github.com/ed0ard/CS2-Bot-Improver',
  license: 'AGPL-3.0',
  stars: null,
  forks: null,
  pushedAt: null,
}

interface CacheRecord<T> { savedAt: number; value: T }

function text(value: unknown, max: number, nullable = false): string | null {
  if (value === null && nullable) return null
  if (typeof value !== 'string') return nullable ? null : ''
  return value.trim().slice(0, max) || (nullable ? null : '')
}

function integer(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? Math.floor(value) : null
}

function parseSupporters(value: unknown): SupporterAcknowledgement[] {
  if (!value || typeof value !== 'object' || !Array.isArray((value as { supporters?: unknown }).supporters)) throw new Error('Invalid supporters response')
  return (value as { supporters: unknown[] }).supporters.slice(0, 50).flatMap((item) => {
    if (!item || typeof item !== 'object') return []
    const row = item as Record<string, unknown>
    const id = text(row.id, 80)
    const amountCents = integer(row.amountCents)
    if (!id || amountCents === null) return []
    return [{
      id,
      nickname: text(row.nickname, 80, true),
      message: text(row.message, 180, true),
      amountCents,
      sortOrder: integer(row.sortOrder) ?? 0,
      isVisible: row.isVisible !== false,
      createdAt: text(row.createdAt, 40) ?? '',
      updatedAt: text(row.updatedAt, 40) ?? '',
    }]
  })
}

function parseUpstream(value: unknown): UpstreamProjectSummary {
  if (!value || typeof value !== 'object') throw new Error('Invalid upstream response')
  const row = value as Record<string, unknown>
  const fullName = text(row.full_name, 100)
  if (!fullName) throw new Error('Missing upstream name')
  const license = row.license && typeof row.license === 'object' ? text((row.license as Record<string, unknown>).spdx_id, 40) : null
  return {
    fullName,
    description: text(row.description, 240, true) ?? FALLBACK_UPSTREAM.description,
    url: text(row.html_url, 300) || FALLBACK_UPSTREAM.url,
    license: license || 'AGPL-3.0',
    stars: integer(row.stargazers_count),
    forks: integer(row.forks_count),
    pushedAt: text(row.pushed_at, 40, true),
  }
}

function readCache<T>(key: string, maxAge: number, parser: (value: unknown) => T): T | null {
  try {
    const record = JSON.parse(localStorage.getItem(key) ?? '') as CacheRecord<unknown>
    if (!Number.isFinite(record.savedAt) || Date.now() - record.savedAt > maxAge) return null
    return parser(record.value)
  } catch { return null }
}

function writeCache(key: string, value: unknown) {
  try { localStorage.setItem(key, JSON.stringify({ savedAt: Date.now(), value })) } catch { /* visual cache is optional */ }
}

async function fetchJson(url: string, timeoutMs = 1_800): Promise<unknown> {
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), timeoutMs)
  try {
    const response = await fetch(url, { signal: controller.signal, headers: { Accept: 'application/json' } })
    if (!response.ok) throw new Error(`HTTP ${response.status}`)
    return await response.json()
  } finally { clearTimeout(timer) }
}

function cachedSupporters(): { value: SupporterAcknowledgement[]; source: IntroData['sources']['supporters'] } {
  const value = readCache(SUPPORTERS_CACHE, 7 * 86_400_000, parseSupporters) ?? []
  return { value, source: value.length ? 'cache' : 'fallback' }
}

function cachedUpstream(): { value: UpstreamProjectSummary; source: IntroData['sources']['upstream'] } {
  const value = readCache(UPSTREAM_CACHE, 7 * 86_400_000, parseUpstream) ?? FALLBACK_UPSTREAM
  return { value, source: value === FALLBACK_UPSTREAM ? 'fallback' : 'cache' }
}

export function loadCachedIntroData(): IntroData {
  const supporters = cachedSupporters()
  const upstream = cachedUpstream()
  return {
    supporters: supporters.value,
    upstream: upstream.value,
    fetchedAt: null,
    sources: { supporters: supporters.source, upstream: upstream.source },
  }
}

async function loadBrowserIntroData(): Promise<IntroData> {
  const [supportersResult, upstreamResult] = await Promise.allSettled([
    fetchJson(SUPPORTERS_URL).then(parseSupporters),
    fetchJson(UPSTREAM_URL).then(parseUpstream),
  ])
  let supportersSource: IntroData['sources']['supporters'] = 'network'
  let upstreamSource: IntroData['sources']['upstream'] = 'network'
  let supporters: SupporterAcknowledgement[]
  let upstream: UpstreamProjectSummary
  if (supportersResult.status === 'fulfilled') {
    supporters = supportersResult.value
    writeCache(SUPPORTERS_CACHE, { supporters })
  } else {
    const cached = cachedSupporters()
    supporters = cached.value
    supportersSource = cached.source
  }
  if (upstreamResult.status === 'fulfilled') {
    upstream = upstreamResult.value
    writeCache(UPSTREAM_CACHE, upstream)
  } else {
    const cached = cachedUpstream()
    upstream = cached.value
    upstreamSource = cached.source
  }
  return {
    supporters,
    upstream,
    fetchedAt: supportersSource === 'network' || upstreamSource === 'network' ? new Date().toISOString() : null,
    sources: { supporters: supportersSource, upstream: upstreamSource },
  }
}

async function loadNativeIntroData(): Promise<IntroData> {
  try {
    const payload = await getIntroPublicData()
    let supporters: SupporterAcknowledgement[]
    let supportersSource: IntroData['sources']['supporters']
    if (payload.sources.supporters === 'network') {
      supporters = parseSupporters({ supporters: payload.supporters })
      supportersSource = 'network'
      writeCache(SUPPORTERS_CACHE, { supporters })
    } else {
      const cached = cachedSupporters()
      supporters = cached.value
      supportersSource = cached.source
    }
    let upstream: UpstreamProjectSummary
    let upstreamSource: IntroData['sources']['upstream']
    if (payload.sources.upstream === 'network') {
      const native = payload.upstream
      upstream = parseUpstream({
        full_name: native.fullName,
        description: native.description,
        html_url: native.url,
        license: { spdx_id: native.license },
        stargazers_count: native.stars,
        forks: native.forks,
        pushed_at: native.pushedAt,
      })
      upstreamSource = 'network'
      writeCache(UPSTREAM_CACHE, upstream)
    } else {
      const cached = cachedUpstream()
      upstream = cached.value
      upstreamSource = cached.source
    }
    return {
      supporters,
      upstream,
      fetchedAt: supportersSource === 'network' || upstreamSource === 'network' ? new Date().toISOString() : null,
      sources: { supporters: supportersSource, upstream: upstreamSource },
    }
  } catch (error) {
    console.warn('INTRO_NATIVE_INVOKE_FAILED', error instanceof Error ? error.message : 'unknown')
    const supporters = cachedSupporters()
    const upstream = cachedUpstream()
    return {
      supporters: supporters.value,
      upstream: upstream.value,
      fetchedAt: null,
      sources: { supporters: supporters.source, upstream: upstream.source },
    }
  }
}

export async function loadIntroData(): Promise<IntroData> {
  return isTauri() ? loadNativeIntroData() : loadBrowserIntroData()
}

export { FALLBACK_UPSTREAM, parseSupporters, parseUpstream }
