import { appConfig } from '@/config/app'
import type { AnnouncementFeed, AnnouncementSeverity, SiteAnnouncement } from './types'

const SEVERITIES = new Set<AnnouncementSeverity>(['success', 'warning', 'error'])

export function shouldAutoOpenAnnouncements(latest: SiteAnnouncement | null) {
  return latest?.severity === 'warning' || latest?.severity === 'error'
}

export function announcementSeverityLabel(severity: AnnouncementSeverity) {
  return ({ success: '成功', warning: '警告', error: '错误' } as const)[severity]
}

function isAnnouncement(value: unknown): value is SiteAnnouncement {
  if (!value || typeof value !== 'object') return false
  const item = value as Record<string, unknown>
  return typeof item.id === 'string'
    && typeof item.title === 'string'
    && typeof item.content === 'string'
    && typeof item.publishedAt === 'string'
    && SEVERITIES.has(item.severity as AnnouncementSeverity)
}

export async function requestAnnouncementFeed(
  fetcher: typeof fetch = fetch,
  signal?: AbortSignal,
): Promise<AnnouncementFeed> {
  const response = await fetcher(appConfig.announcementFeedUrl, {
    signal,
    headers: { Accept: 'application/json' },
  })
  if (!response.ok) throw new Error(`公告服务暂时不可用（HTTP ${response.status}）`)
  const payload = await response.json() as Record<string, unknown>
  const notices = Array.isArray(payload.notices)
    ? payload.notices.filter(isAnnouncement).sort((a, b) => Date.parse(b.publishedAt) - Date.parse(a.publishedAt))
    : []
  return { latest: notices[0] ?? null, notices }
}
