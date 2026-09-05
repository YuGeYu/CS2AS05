export type AnnouncementSeverity = 'success' | 'warning' | 'error'

export interface SiteAnnouncement {
  id: string
  scope: 'idea'
  title: string
  content: string
  severity: AnnouncementSeverity
  publishedAt: string
  createdAt: string
  updatedAt: string
}

export interface AnnouncementFeed {
  latest: SiteAnnouncement | null
  notices: SiteAnnouncement[]
}
