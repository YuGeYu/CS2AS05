export type PanelMode = 'online' | 'bots'
export type Difficulty = 'Low' | 'Medium' | 'High'
export type AimValue = 'head' | 'mixed' | 'body'
export type NadesValue = 'max' | 'more' | 'normal' | 'less' | 'off'
export type BotItem = 'profiles' | 'agents' | 'music' | 'weapons' | 'knives' | 'gloves' | 'stickers' | 'charms'

export interface PanelSnapshot {
  rootPath: string
  ready: boolean
  missingFiles: string[]
  cs2Running: boolean
  mode: { current: PanelMode | null; insecure: boolean; writable: boolean }
  gameinfo?: { status: 'official' | 'bots' | 'unknown' | 'recoveryRequired'; activeSha256?: string | null; officialSha256?: string | null; resourceVersion?: string | null; writable: boolean }
  difficulty: { current: Difficulty | null; available: Difficulty[] }
  presets: { aim: AimValue | null; nades: NadesValue | null; writable: boolean }
  botItems: Record<BotItem, boolean> & { writable: boolean }
  dropKnives: { bindKey: string; selected: number[]; writable: boolean }
}

export interface LaunchResult {
  options: string
  insecure: boolean
  pluginAction: 'unchanged' | 'installed'
  pluginVersion: string
  finalMode?: PanelMode
  gameinfoSha256?: string
}

export interface PanelInitializationResult {
  status: 'initialized' | 'unchanged' | 'deferred'
  initializedFields: string[]
}
