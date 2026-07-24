import { invoke } from '@tauri-apps/api/core'

export function openOfficialSite() {
  return invoke<void>('open_official_site')
}

export function openIdeaPage() {
  return invoke<void>('open_idea_page')
}

export function openReleasePage() {
  return invoke<void>('open_release_page')
}

export function openUpstreamProject() {
  return invoke<void>('open_upstream_project')
}

export function openUpdateDownload(url: string) {
  return invoke<void>('open_update_download', { url })
}
