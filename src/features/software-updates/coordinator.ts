import { computed, reactive } from 'vue'

import { appConfig } from '@/config/app'
import { checkForSoftwareUpdates, dismissRelease, shouldPresentRelease } from '@/features/software-updates/state'
import {
  deferDownloadedSoftwareUpdate,
  discardStaleDownloadedUpdate,
  downloadSoftwareUpdate,
  initializeSoftwareUpdaterState,
  installDownloadedSoftwareUpdate,
  softwareUpdaterState,
} from '@/features/software-updates/updater-state'
import type { SoftwareRelease } from '@/features/software-updates/types'
import { openReleasePage, openUpdateDownload } from '@/services/tauri/support'

const state = reactive({
  checking: false,
  started: false,
  message: '启动后自动检查更新，也可以随时手动检查。',
  activeRelease: null as SoftwareRelease | null,
  latestRelease: null as SoftwareRelease | null,
  downloadError: false,
})

let startupPromise: Promise<void> | null = null

export const softwareUpdateCoordinatorState = state
export const softwareUpdateStatusMessage = computed(() => {
  if (softwareUpdaterState.phase === 'deferred-current-session') return `v${softwareUpdaterState.version} 已下载，等待安装。`
  if (softwareUpdaterState.phase === 'deferred-reminder-only') return `上次选择稍后安装 v${softwareUpdaterState.version}；程序关闭后安装包不会保留，需要重新下载。`
  if (softwareUpdaterState.phase === 'failed' && softwareUpdaterState.error) return softwareUpdaterState.error
  return state.message
})

export function startSoftwareUpdateCoordinator() {
  if (startupPromise) return startupPromise
  state.started = true
  initializeSoftwareUpdaterState()
  startupPromise = checkSoftwareUpdates(false)
  return startupPromise
}

export async function checkSoftwareUpdates(manual = true) {
  state.checking = true
  state.message = '正在检查更新...'
  try {
    const result = await checkForSoftwareUpdates(manual)
    if (result.status === 'disabled') state.message = '自动更新检查当前已关闭。'
    if (result.status === 'current') {
      state.message = `当前 ${appConfig.appVersion} 已是最新版本。`
      if (softwareUpdaterState.phase === 'deferred-reminder-only') await discardStaleDownloadedUpdate()
    }
    if (result.status === 'failed') state.message = result.message
    if (result.status === 'available') {
      if (softwareUpdaterState.version && softwareUpdaterState.version !== result.release.version) await discardStaleDownloadedUpdate()
      state.latestRelease = result.release
      state.message = `发现新版本 ${result.release.version}。`
      if (shouldPresentRelease(result.release, manual)) state.activeRelease = result.release
    }
  } finally {
    state.checking = false
  }
}

export function closeSoftwareUpdate() {
  if (!state.activeRelease || ['checking', 'downloading', 'installing', 'restarting'].includes(softwareUpdaterState.phase)) return
  dismissRelease(state.activeRelease)
  state.activeRelease = null
  state.downloadError = false
}

export async function openSoftwareUpdateDownload() {
  if (!state.activeRelease) return
  state.downloadError = false
  try { await openUpdateDownload(state.activeRelease.download.url) } catch { state.downloadError = true }
}

export async function startSoftwareUpdateDownload() {
  if (!state.activeRelease) return
  try { await downloadSoftwareUpdate(state.activeRelease.version) } catch { /* reactive updater state exposes the error */ }
}

export function deferSoftwareUpdateInstall() {
  deferDownloadedSoftwareUpdate()
  state.activeRelease = null
}

export async function installSoftwareUpdate() {
  try { await installDownloadedSoftwareUpdate() } catch { /* keep modal open with fallback */ }
}

export async function resumeSoftwareUpdate() {
  if (softwareUpdaterState.phase === 'deferred-current-session' && state.latestRelease) { state.activeRelease = state.latestRelease; return }
  await checkSoftwareUpdates(true)
}

export function showPendingSoftwareUpdate() {
  if (state.latestRelease) state.activeRelease = state.latestRelease
}

export async function openSoftwareUpdateReleasePage() {
  try { await openReleasePage() } catch { state.message = '打开官网更新日志失败，请稍后重试。' }
}

export function resetSoftwareUpdateCoordinatorForTests() {
  startupPromise = null
  Object.assign(state, { checking: false, started: false, message: '启动后自动检查更新，也可以随时手动检查。', activeRelease: null, latestRelease: null, downloadError: false })
}
