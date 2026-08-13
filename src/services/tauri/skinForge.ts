import { invoke } from '@tauri-apps/api/core'
import type { PlayerSkinModFile } from '@/features/skin-forge/player-skin-mod-adapter'

export interface SaveResult { loadoutPath: string; sha256: string; slotCount: number }
export interface PluginCheckResult {
  allPresent: boolean
  missingFiles: string[]
  hashMismatches: string[]
  versionMismatch: boolean
  deployedVersion: string | null
  panelVersion: string
  counterstrikesharpInstalled: boolean
  counterstrikesharpVersion: string | null
  playerSkinModPresent: boolean
  manifestVersion: string | null
  resourceVersion: string
  resourceHashesMatch: boolean
  loadoutReadable: boolean
  canDeploy: boolean
  blockedCode: string | null
  blockedMessage: string | null
  selectedRoot: string
  csgoRoot: string
  targetDir: string
  loadoutPath: string
  hashes: Record<string, string>
}
export interface DeployResult { targetDir: string; files: string[]; pluginVersion: string; hashes: Record<string, string>; loadoutPath: string; counterstrikesharpInstalled: boolean; backupDir: string | null }
export interface CachedImage { path: string; sha256: string; mime: string; bytes: number; cacheHit: boolean }
export const skinForgeGetConfig = () => invoke<{ language: string | null; cs2Path: string | null; draftPath: string }>('skin_forge_get_config')
export const skinForgeLoad = (rootPath: string, slot = 0) => invoke<PlayerSkinModFile | null>('skin_forge_load_loadout', { rootPath, slot })
export const skinForgeSave = (rootPath: string, loadout: PlayerSkinModFile, slot = 0) => invoke<SaveResult>('skin_forge_save_loadout', { rootPath, slot, loadout })
export const skinForgeReset = (rootPath: string, slot = 0) => invoke<SaveResult>('skin_forge_reset_loadout', { rootPath, slot })
export const skinForgeCheckPlugin = (rootPath: string) => invoke<PluginCheckResult>('skin_forge_check_plugin', { rootPath })
export const skinForgeDeploy = (rootPath: string) => invoke<DeployResult>('skin_forge_deploy_plugin', { rootPath })
export const skinForgeCacheImage = (url: string) => invoke<CachedImage>('skin_forge_cache_image', { url })
