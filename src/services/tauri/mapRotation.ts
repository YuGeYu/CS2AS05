import { invoke } from '@tauri-apps/api/core'
import type { MapRotationDefault } from '@/types/mapRotation'

export const getMapRotationDefault = (rootPath: string) => invoke<MapRotationDefault>('get_map_rotation_default', { rootPath })
export const setMapRotationDefault = (rootPath: string, enabled: boolean) => invoke<MapRotationDefault>('set_map_rotation_default', { rootPath, enabled })
export const resetMapRotationDefault = (rootPath: string) => invoke<MapRotationDefault>('reset_map_rotation_default', { rootPath })
