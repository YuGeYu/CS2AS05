import { invoke } from '@tauri-apps/api/core'

import type { DemoTracerInstallResult, DemoTracerOperationResult, DemoTracerStatus } from '@/types/demotracer'

export function getDemoTracerStatus(rootPath?: string) {
  return invoke<DemoTracerStatus>('demotracer_get_status', { rootPath })
}

export function installDemoTracerPlayback(rootPath: string) {
  return invoke<DemoTracerInstallResult>('demotracer_install_playback', { rootPath })
}

export function uninstallDemoTracerPlayback(rootPath: string) {
  return invoke<DemoTracerOperationResult>('demotracer_uninstall_playback', { rootPath })
}

export function openDemoTracerGui() {
  return invoke<DemoTracerOperationResult>('demotracer_open_gui')
}
