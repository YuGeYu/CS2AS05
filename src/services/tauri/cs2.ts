import { Channel, invoke } from '@tauri-apps/api/core'

import type {
  Cs2EnvironmentStatus,
  Cs2RootCandidate,
  DiagnosticsPayload,
  OperationResult,
  Cs2RootScanEvent,
  Cs2RootScanSummary,
} from '@/types/cs2'

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

function unavailable(message: string): Promise<never> {
  return Promise.reject(new Error(message))
}

export function discoverCs2Roots() {
  return isTauriRuntime()
    ? invoke<Cs2RootCandidate[]>('discover_cs2_roots')
    : unavailable('目录扫描仅可在桌面程序中使用。')
}

export function inspectCs2Root(rootPath: string) {
  return isTauriRuntime()
    ? invoke<Cs2EnvironmentStatus>('inspect_cs2_root', { rootPath })
    : unavailable('目录检查仅可在桌面程序中使用。')
}

export function checkCs2Process() {
  return isTauriRuntime()
    ? invoke<boolean>('check_cs2_process')
    : unavailable('运行状态检查仅可在桌面程序中使用。')
}

export function installBotPackage(rootPath: string) {
  return invoke<OperationResult>('install_bot_package', { rootPath })
}

export function openUpstreamPanel() {
  return invoke<OperationResult>('open_upstream_panel')
}

export function uninstallBotPackage(rootPath: string) {
  return invoke<OperationResult>('uninstall_bot_package', { rootPath })
}

export function getDiagnosticsPayload(rootPath?: string) {
  return invoke<DiagnosticsPayload>('get_diagnostics_payload', { rootPath })
}

export function guessCs2Roots(onEvent: (event: Cs2RootScanEvent) => void) {
  const channel = new Channel<Cs2RootScanEvent>()
  channel.onmessage = onEvent
  return invoke<Cs2RootScanSummary>('guess_cs2_roots', { onEvent: channel })
}

export function stopGuessCs2Roots() {
  return invoke<boolean>('stop_guess_cs2_roots')
}
