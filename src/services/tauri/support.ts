import { invoke } from '@tauri-apps/api/core'
import type { PanelSnapshot } from '@/features/panel/types'

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

export function openResourceLink(url: string) {
  return invoke<void>('open_resource_link', { url })
}

export function openCommunityDownload(url: string) {
  return invoke<void>('open_community_download', { url })
}
export function openAccountRegister() { return invoke<void>('open_account_register') }

export function launchCommunityConnect(rootPath: string, connection: string) {
  return invoke<PanelSnapshot>('launch_community_connect', { rootPath, connection })
}
export function isBotProfileGuideSeen() { return invoke<boolean>('is_bot_profile_guide_seen') }
export function dismissBotProfileGuide() { return invoke<void>('dismiss_bot_profile_guide') }

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
export function quickRegisterAssistant() { return invoke<AssistantAccount>('quick_register_assistant') }

export interface IdeaUser {
  id: string
  username: string
  displayName: string
  role: 'user' | 'admin' | 'owner'
  isDisabled?: number
  createdAt?: string
}

export interface IdeaReply {
  id: string
  content: string
  createdAt: string
  updatedAt: string
  admin: IdeaUser
}

export interface IdeaEntry {
  id: string
  content: string
  createdAt: string
  updatedAt: string
  user: IdeaUser
  reply?: IdeaReply | null
}

export interface IdeaSection {
  id: string
  title: string
  sortOrder: number
  kind?: 'idea' | 'fault'
  locked?: boolean
  faultId?: string | null
  faultTicketId?: string | null
  faultStatus?: 'pending' | 'investigating' | 'resolved' | 'closed' | null
  ideas: IdeaEntry[]
}

export interface IdeaNotice {
  id: string
  scope: 'idea'
  title: string
  content: string
  severity: 'success' | 'warning' | 'error'
  isActive: boolean
  publishedAt: string
  createdAt: string
  updatedAt: string
}

export interface IdeasPayload {
  notice: IdeaNotice | null
  notices: IdeaNotice[]
  sections: IdeaSection[]
  user: IdeaUser | null
}

export function loadIdeas() { return invoke<IdeasPayload>('load_ideas') }
export function createIdea(sectionId: string, content: string) { return invoke<{ idea: IdeaEntry }>('create_idea', { sectionId, content }) }
export function updateIdea(ideaId: string, content: string) { return invoke<{ idea: IdeaEntry }>('update_idea', { ideaId, content }) }
export function deleteIdea(ideaId: string) { return invoke<{ ok: true }>('delete_idea', { ideaId }) }
export function createIdeaSection(title: string) { return invoke<{ section: IdeaSection }>('create_idea_section', { title }) }
export function updateIdeaSection(sectionId: string, title: string) { return invoke<{ section: IdeaSection }>('update_idea_section', { sectionId, title }) }
export function deleteIdeaSection(sectionId: string) { return invoke<{ ok: true; deletedIdeas: number }>('delete_idea_section', { sectionId }) }
export function replyIdea(ideaId: string, content: string) { return invoke<{ reply: IdeaReply }>('reply_idea', { ideaId, content }) }

export function logoutAssistant() {
  return invoke<AssistantAccount>('logout_assistant')
}

export interface CommunityAuth {
  token: string
  serviceUrl: string
  expiresAt: number
}

export function getCommunityAuth() {
  return invoke<CommunityAuth>('get_community_auth')
}

export function downloadCommunityFile(url: string, filename: string, token: string) {
  return invoke<string>('download_community_file', { url, filename, token })
}
