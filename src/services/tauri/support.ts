import { invoke } from '@tauri-apps/api/core'

export function openOfficialSite() {
  return invoke<void>('open_official_site')
}

export function openIdeaPage() {
  return invoke<void>('open_idea_page')
}

export function openApiPurchase() {
  return invoke<void>('open_api_purchase')
}

export interface AiConnectionSummary {
  url: string
  model: string
  hasKey: boolean
  keyHint: string | null
  usingDefault: boolean
}

export interface AiChatMessage {
  role: 'user' | 'assistant'
  content: string
}

export interface AiChatSession {
  id: string
  title: string
  messages: AiChatMessage[]
  updatedAt: number
}

export function getAiConnection() {
  return invoke<AiConnectionSummary>('get_ai_connection')
}

export function saveAiConnection(url: string, key: string, model: string) {
  return invoke<AiConnectionSummary>('save_ai_connection', { url, key, model })
}

export function getAiChatSessions() {
  return invoke<AiChatSession[]>('get_ai_chat_sessions')
}

export function saveAiChatSessions(sessions: AiChatSession[]) {
  return invoke<void>('save_ai_chat_sessions', { sessions })
}

export function getAiModels(url: string, key = '') {
  return invoke<string[]>('get_ai_models', { url, key })
}

export function runAiPowerShell(command: string) {
  return invoke<string>('run_ai_powershell', { command })
}

export function chatAi(messages: AiChatMessage[], context?: string) {
  return invoke<string>('chat_ai', { messages, context })
}

export function openFaultIdeaPage(ticketId: string) {
  return invoke<void>('open_fault_idea_page', { ticketId })
}

export function openReleasePage() {
  return invoke<void>('open_release_page')
}

export function openUpstreamProject() {
  return invoke<void>('open_upstream_project')
}

export type ReferenceProjectId = 'bot-improver' | 'botvision' | 'demotracer' | 'demoparser' | 'cs-demo-manager' | 'inventory-simulator' | 'threejs' | 'gametracking-cs2' | 'insight-agent'

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
  ideaSectionId: string
  accountUsername: string
  autoRegistered: boolean
}

export interface AssistantAccount {
  username: string | null
  loggedIn: boolean
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

export function getAssistantAccount() {
  return invoke<AssistantAccount>('get_assistant_account')
}

export function loginAssistant(username: string, password: string) {
  return invoke<AssistantAccount>('login_assistant', { username, password })
}

export function logoutAssistant() {
  return invoke<AssistantAccount>('logout_assistant')
}
