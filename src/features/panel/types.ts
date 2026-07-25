export type PanelMode = 'online' | 'bots'
export type Difficulty = 'Low' | 'Medium' | 'High'
export type AimValue = 'head' | 'mixed' | 'body'
export type NadesValue = 'max' | 'more' | 'normal' | 'off'
export type BotItem = 'skins' | 'profiles' | 'agents' | 'music'

export interface PanelSnapshot {
  rootPath: string
  ready: boolean
  missingFiles: string[]
  cs2Running: boolean
  mode: { current: PanelMode | null; insecure: boolean; writable: boolean }
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
}

export interface PanelInitializationResult {
  status: 'initialized' | 'unchanged' | 'deferred'
  initializedFields: string[]
}
