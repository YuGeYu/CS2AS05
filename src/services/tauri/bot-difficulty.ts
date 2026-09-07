import { invoke } from '@tauri-apps/api/core'

export interface BotProfileSummary { id: string; name: string; source: 'builtin' | 'custom'; baseDifficulty?: string; active: boolean; readOnly: boolean; dbSha256?: string; vpkSha256?: string; warnings: string[] }
export interface BotProfileList { profiles: BotProfileSummary[]; toolVersion?: string; toolSha256?: string }
export interface BotProfileDocument { profile: BotProfileSummary; text?: string; entryPath: string; validation: string; dirty: boolean; toolVersion?: string }
export interface BotToolState { status: 'missing' | 'invalid' | 'ready'; version: string; sha256?: string; pathHint?: string; detail?: string }
export interface BotProfileOperation { profile: BotProfileSummary; backupPath?: string; applied: boolean; message: string }
export interface CreateBotProfileRequest { baseProfileId: string; name: string }
export interface SaveBotProfileRequest { profileId: string; text: string; expectedDbSha256: string }
export interface RenameBotProfileRequest { profileId: string; name: string }
export const listBotProfiles = (rootPath: string) => invoke<BotProfileList>('list_bot_profiles', { rootPath })
export const openBotProfile = (rootPath: string, profileId: string) => invoke<BotProfileDocument>('open_bot_profile', { rootPath, profileId })
export const getBotWorkshopState = () => invoke<BotToolState>('get_bot_workshop_state')
export const createBotProfile = (rootPath: string, request: CreateBotProfileRequest) => invoke<BotProfileOperation>('create_bot_profile', { rootPath, request })
export const saveBotProfile = (rootPath: string, request: SaveBotProfileRequest) => invoke<BotProfileOperation>('save_bot_profile', { rootPath, request })
export const renameBotProfile = (request: RenameBotProfileRequest) => invoke<BotProfileOperation>('rename_bot_profile', { request })
export const deleteBotProfile = (rootPath: string, profileId: string) => invoke<BotProfileOperation>('delete_bot_profile', { rootPath, profileId })
export const applyBotProfile = (rootPath: string, profileId: string) => invoke<BotProfileOperation>('apply_bot_profile', { rootPath, profileId })
