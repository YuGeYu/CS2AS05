import { reactive } from 'vue'
import { requestAnnouncementFeed, shouldAutoOpenAnnouncements } from './service'
import type { SiteAnnouncement } from './types'

export const announcementState = reactive({
  open: false,
  loading: false,
  loaded: false,
  notices: [] as SiteAnnouncement[],
  latest: null as SiteAnnouncement | null,
  error: '',
})

let requestGeneration = 0

export async function loadAnnouncements() {
  const generation = ++requestGeneration
  const controller = new AbortController()
  const timer = setTimeout(() => controller.abort(), 8_000)
  announcementState.loading = true
  announcementState.error = ''
  try {
    const feed = await requestAnnouncementFeed(fetch, controller.signal)
    if (generation !== requestGeneration) return
    announcementState.notices = feed.notices
    announcementState.latest = feed.latest
    announcementState.loaded = true
    if (shouldAutoOpenAnnouncements(feed.latest)) announcementState.open = true
  } catch (error) {
    if (generation !== requestGeneration) return
    announcementState.loaded = true
    announcementState.error = error instanceof DOMException && error.name === 'AbortError'
      ? '公告加载超时，请稍后重试。'
      : error instanceof Error ? error.message : '公告加载失败。'
  } finally {
    clearTimeout(timer)
    if (generation === requestGeneration) announcementState.loading = false
  }
}

export function openAnnouncementCenter() {
  announcementState.open = true
  if (!announcementState.loaded && !announcementState.loading) void loadAnnouncements()
}

export function closeAnnouncementCenter() {
  announcementState.open = false
}

export function resetAnnouncementsForTests() {
  requestGeneration++
  announcementState.open = false
  announcementState.loading = false
  announcementState.loaded = false
  announcementState.notices = []
  announcementState.latest = null
  announcementState.error = ''
}
