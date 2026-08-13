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

export type ReferenceProjectId = 'bot-improver' | 'demotracer' | 'demoparser' | 'cs-demo-manager' | 'skin-forge'

export function openReferenceProject(project: ReferenceProjectId) {
  return invoke<void>('open_reference_project', { project })
}

export function openUpdateDownload(url: string) {
  return invoke<void>('open_update_download', { url })
}

export interface AssistantPreferences {
  autostartEnabled: boolean
}

export interface FaultSubmissionResult {
  success: boolean
  ticketId: string
  message: string
}

export function getAssistantPreferences() {
  return invoke<AssistantPreferences>('get_assistant_preferences')
}

export function setAssistantAutostart(enabled: boolean) {
  return invoke<AssistantPreferences>('set_assistant_autostart', { enabled })
}

export function clearAssistantData() {
  return invoke<{ success: boolean; message: string }>('clear_assistant_data')
}

export function submitFaultReport(details: string, rootPath?: string) {
  return invoke<FaultSubmissionResult>('submit_fault_report', { details, rootPath })
}
