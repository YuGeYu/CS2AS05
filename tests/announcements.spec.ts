import { describe, expect, it, vi } from 'vitest'

import { announcementSeverityLabel, requestAnnouncementFeed, shouldAutoOpenAnnouncements } from '@/features/announcements/service'
import type { SiteAnnouncement } from '@/features/announcements/types'

const notice = (severity: SiteAnnouncement['severity'], publishedAt = '2026-08-16T08:00:00.000Z'): SiteAnnouncement => ({
  id: `${severity}-${publishedAt}`,
  scope: 'idea',
  title: `${announcementSeverityLabel(severity)}公告`,
  content: '公告内容',
  severity,
  publishedAt,
  createdAt: publishedAt,
  updatedAt: publishedAt,
})

describe('assistant announcements', () => {
  it('opens automatically only when the latest severity is warning or error', () => {
    expect(shouldAutoOpenAnnouncements(notice('success'))).toBe(false)
    expect(shouldAutoOpenAnnouncements(notice('warning'))).toBe(true)
    expect(shouldAutoOpenAnnouncements(notice('error'))).toBe(true)
    expect(shouldAutoOpenAnnouncements(null)).toBe(false)
  })

  it('uses the three user-facing Chinese severity names', () => {
    expect(['success', 'warning', 'error'].map(announcementSeverityLabel)).toEqual(['成功', '警告', '错误'])
  })

  it('sorts valid public announcements newest first and rejects malformed entries', async () => {
    const older = notice('error', '2026-08-15T08:00:00.000Z')
    const latest = notice('success', '2026-08-16T08:00:00.000Z')
    const fetcher = vi.fn(async () => new Response(JSON.stringify({ notices: [older, { title: '缺少字段' }, latest] }), { status: 200 }))
    const feed = await requestAnnouncementFeed(fetcher as typeof fetch)
    expect(feed.notices).toEqual([latest, older])
    expect(feed.latest).toEqual(latest)
  })

  it('surfaces an actionable HTTP error', async () => {
    const fetcher = vi.fn(async () => new Response('{}', { status: 503 }))
    await expect(requestAnnouncementFeed(fetcher as typeof fetch)).rejects.toThrow('HTTP 503')
  })
})
