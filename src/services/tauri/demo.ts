import { invoke } from '@tauri-apps/api/core'
import type { DemoListPage, DemoRecordingSettings, DemoReport, DemoRoot } from '@/types/demo'

export const listDemoRoots = () => invoke<DemoRoot[]>('list_demo_roots')
export const addDemoRoot = (path: string, scanDepth = 5) => invoke<DemoRoot>('add_demo_root', { path, scanDepth })
export const ensureDefaultDemoRoot = (rootPath: string) => invoke<DemoRoot>('ensure_default_demo_root', { rootPath })
export const updateDemoRoot = (id: number, enabled: boolean, scanDepth: number) => invoke<void>('update_demo_root', { id, enabled, scanDepth })
export const removeDemoRoot = (id: number) => invoke<void>('remove_demo_root', { id })
export const scanDemoRoots = () => invoke<{ discovered: number; parsed: number; failed: number }>('scan_demo_roots')
export const importDemoFile = (path: string) => invoke<{ demoFileId: number; status: string }>('import_demo_file', { path })
export const listDemos = (query: string, status: string, page: number, pageSize: number) => invoke<DemoListPage>('list_demos', { query, status, page, pageSize })
export const getDemoReport = (id: number) => invoke<DemoReport>('get_demo_report', { id })
export const retryDemoParse = (id: number) => invoke<{ demoFileId: number; status: string }>('retry_demo_parse', { id })
export const getDemoSettings = (rootPath: string) => invoke<DemoRecordingSettings>('get_demo_settings', { rootPath })
export const setDemoRecordingEnabled = (rootPath: string, enabled: boolean) => invoke<DemoRecordingSettings>('set_demo_recording_enabled', { rootPath, enabled })
