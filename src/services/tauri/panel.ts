import { invoke } from '@tauri-apps/api/core'

import type { AimValue, BotItem, Difficulty, LaunchResult, NadesValue, PanelInitializationResult, PanelMode, PanelSnapshot } from '@/features/panel/types'

export const initializePanelDefaults = (rootPath: string) => invoke<PanelInitializationResult>('initialize_panel_defaults', { rootPath })
export const getPanelSnapshot = (rootPath: string) => invoke<PanelSnapshot>('get_panel_snapshot', { rootPath })
export const setPanelMode = (rootPath: string, mode: PanelMode) => invoke<PanelSnapshot>('set_panel_mode', { rootPath, mode })
export const setPanelDifficulty = (rootPath: string, level: Difficulty) => invoke<PanelSnapshot>('set_panel_difficulty', { rootPath, level })
export const setPanelAim = (rootPath: string, value: AimValue) => invoke<PanelSnapshot>('set_panel_aim', { rootPath, value })
export const setPanelNades = (rootPath: string, value: NadesValue) => invoke<PanelSnapshot>('set_panel_nades', { rootPath, value })
export const setPanelBotItem = (rootPath: string, item: BotItem, enabled: boolean) => invoke<PanelSnapshot>('set_panel_bot_item', { rootPath, item, enabled })
export const setPanelDropKnives = (rootPath: string, bindKey: string, selected: number[]) => invoke<PanelSnapshot>('set_panel_drop_knives', { rootPath, bindKey, selected })
export const launchPanelCs2 = (rootPath: string, mode: PanelMode) => invoke<LaunchResult>('launch_panel_cs2', { rootPath, mode })
