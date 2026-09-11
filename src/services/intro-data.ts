import type { IntroData, SupporterAcknowledgement, UpstreamProjectSummary } from '@/features/intro/types'
import { STATIC_INTRO_DATA } from '@/features/intro/static-data'
import { getIntroPublicData } from '@/services/tauri/intro'

function text(value: unknown, max: number, nullable = false): string | null {
  if (value === null && nullable) return null
  if (typeof value !== 'string') return nullable ? null : ''
  return value.trim().slice(0, max) || (nullable ? null : '')
}
function integer(value: unknown): number | null { return typeof value === 'number' && Number.isFinite(value) && value >= 0 ? Math.floor(value) : null }
export function parseSupporters(value: unknown): SupporterAcknowledgement[] {
  if (!value || typeof value !== 'object' || !Array.isArray((value as { supporters?: unknown }).supporters)) throw new Error('Invalid supporters response')
  return (value as { supporters: unknown[] }).supporters.slice(0, 50).flatMap(item => {
    if (!item || typeof item !== 'object') return []
    const row = item as Record<string, unknown>; const id = text(row.id, 80)
    const amountCents = integer(row.amountCents)
    const visibleAmount = typeof row.visibleAmount === 'number' && Number.isFinite(row.visibleAmount) && row.visibleAmount >= 0 ? row.visibleAmount : null
    if (!id || (amountCents === null && visibleAmount === null)) return []
    return [{ id, nickname: text(row.nickname, 80, true), message: text(row.message, 180, true), amountCents, platform: row.platform === 'bilibili' || row.platform === 'wechat' || row.platform === 'other' ? row.platform : undefined, unit: row.unit === 'beike' || row.unit === 'cny' || row.unit === 'unknown' ? row.unit : undefined, visibleAmount, exchangeRateCny: typeof row.exchangeRateCny === 'number' ? row.exchangeRateCny : null, amountScope: row.amountScope === 'visible_record' || row.amountScope === 'reported_total' || row.amountScope === 'unknown' ? row.amountScope : undefined, sourceLabel: text(row.sourceLabel, 100, true), occurredAt: text(row.occurredAt, 40, true), sortOrder: integer(row.sortOrder) ?? 0, isVisible: row.isVisible !== false, createdAt: text(row.createdAt, 40) ?? '', updatedAt: text(row.updatedAt, 40) ?? '' }]
  })
}
export function parseUpstream(value: unknown): UpstreamProjectSummary {
  if (!value || typeof value !== 'object') throw new Error('Invalid upstream response')
  const row = value as Record<string, unknown>; const fullName = text(row.full_name, 100)
  if (!fullName) throw new Error('Missing upstream name')
  const license = row.license && typeof row.license === 'object' ? text((row.license as Record<string, unknown>).spdx_id, 40) : null
  return { fullName, description: text(row.description, 240, true) ?? STATIC_INTRO_DATA.upstream.description, url: text(row.html_url, 300) || STATIC_INTRO_DATA.upstream.url, license: license || 'AGPL-3.0', stars: integer(row.stargazers_count), forks: integer(row.forks_count), pushedAt: text(row.pushed_at, 40, true) }
}
export function loadCachedIntroData(): IntroData { return structuredClone(STATIC_INTRO_DATA) }
let inFlight: Promise<IntroData> | null = null
export async function loadIntroData(): Promise<IntroData> {
  if (inFlight) return inFlight
  inFlight = getIntroPublicData().then(payload => ({ supporters: parseSupporters({ supporters: payload.supporters }), upstream: parseUpstream({ full_name: payload.upstream.fullName, description: payload.upstream.description, html_url: payload.upstream.url, license: { spdx_id: payload.upstream.license }, stargazers_count: payload.upstream.stars, forks_count: payload.upstream.forks, pushed_at: payload.upstream.pushedAt }), fetchedAt: new Date().toISOString(), sources: payload.sources })).catch(() => loadCachedIntroData()).finally(() => { inFlight = null })
  return inFlight
}
export const FALLBACK_UPSTREAM = STATIC_INTRO_DATA.upstream
