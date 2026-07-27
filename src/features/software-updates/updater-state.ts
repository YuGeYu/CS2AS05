import { reactive, readonly } from 'vue'

import { requestSoftwareUpdate } from '@/services/software-updates'
import { checkTauriUpdater, relaunchAfterUpdate, type UpdaterDownloadEvent, type UpdaterResource } from '@/services/tauri/software-updater'

export type UpdaterPhase =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'downloaded'
  | 'deferred-current-session'
  | 'deferred-reminder-only'
  | 'installing'
  | 'restarting'
  | 'failed'

const DEFERRED_KEY = 'cs2-bot-improver.deferred-update.v1'
const state = reactive({
  phase: 'idle' as UpdaterPhase,
  version: '',
  downloadedBytes: 0,
  totalBytes: null as number | null,
  error: '',
  reminderNeedsDownload: false,
})

let updateResource: UpdaterResource | null = null
let inFlight: Promise<void> | null = null

export const softwareUpdaterState = readonly(state)

export function initializeSoftwareUpdaterState() {
  if (updateResource || ['downloading', 'downloaded', 'deferred-current-session', 'installing', 'restarting'].includes(state.phase)) return
  const deferred = readDeferredReminder()
  if (!deferred) return
  state.phase = 'deferred-reminder-only'
  state.version = deferred.version
  state.reminderNeedsDownload = true
}

export async function downloadSoftwareUpdate(expectedVersion: string) {
  if (inFlight) return inFlight
  inFlight = downloadSoftwareUpdateInner(expectedVersion).finally(() => { inFlight = null })
  return inFlight
}

async function downloadSoftwareUpdateInner(expectedVersion: string) {
  if (updateResource && state.version === expectedVersion && ['downloaded', 'deferred-current-session'].includes(state.phase)) return
  await releaseUpdaterResource()
  state.phase = 'checking'
  state.version = expectedVersion
  state.downloadedBytes = 0
  state.totalBytes = null
  state.error = ''
  state.reminderNeedsDownload = false
  try {
    const update = await checkTauriUpdater()
    if (!update || update.version !== expectedVersion) throw new Error('官网直连没有返回当前提示的版本，请使用夸克更新。')
    updateResource = update
    state.phase = 'available'
    state.phase = 'downloading'
    await update.download(handleDownloadEvent)
    state.phase = 'downloaded'
  } catch (error) {
    await releaseUpdaterResource()
    state.phase = 'failed'
    state.error = normalizeUpdaterError(error, '下载或签名检查失败，请使用夸克更新。')
    throw error
  }
}

function handleDownloadEvent(event: UpdaterDownloadEvent) {
  if (event.event === 'Started') {
    state.downloadedBytes = 0
    state.totalBytes = event.data.contentLength ?? null
  } else if (event.event === 'Progress') {
    state.downloadedBytes += Math.max(0, event.data.chunkLength)
    if (state.totalBytes !== null) state.downloadedBytes = Math.min(state.downloadedBytes, state.totalBytes)
  } else if (event.event === 'Finished' && state.totalBytes !== null) {
    state.downloadedBytes = state.totalBytes
  }
}

export function deferDownloadedSoftwareUpdate() {
  if (!updateResource || state.phase !== 'downloaded') return
  state.phase = 'deferred-current-session'
  writeDeferredReminder(state.version)
}

export async function installDownloadedSoftwareUpdate() {
  if (inFlight) return inFlight
  inFlight = installDownloadedSoftwareUpdateInner().finally(() => { inFlight = null })
  return inFlight
}

async function installDownloadedSoftwareUpdateInner() {
  if (!updateResource || !['downloaded', 'deferred-current-session'].includes(state.phase)) throw new Error('已下载的更新已不在当前程序中，需要重新下载。')
  const version = state.version
  try {
    const result = await requestSoftwareUpdate({ manual: true })
    if (result.status !== 'available' || result.release.version !== version || !result.release.isActive) {
      await releaseUpdaterResource()
      clearDeferredReminder()
      state.phase = 'failed'
      state.error = '该版本已撤回或不再是最新版，请重新检查更新。'
      throw new Error(state.error)
    }
    state.phase = 'installing'
    await updateResource.install()
    clearDeferredReminder()
    state.phase = 'restarting'
    await relaunchAfterUpdate()
  } catch (error) {
    if (state.phase !== 'failed') {
      await releaseUpdaterResource()
      clearDeferredReminder()
      state.phase = 'failed'
      state.error = normalizeUpdaterError(error, '安装或重启未完成。Windows 安装器可能已接管，请留意程序是否自动退出。')
    }
    throw error
  }
}

export async function prepareDeferredUpdateForExit() {
  if (updateResource && ['downloaded', 'deferred-current-session'].includes(state.phase)) writeDeferredReminder(state.version)
  await releaseUpdaterResource()
}

export function hasPendingDownloadedUpdate() {
  return Boolean(updateResource && ['downloaded', 'deferred-current-session'].includes(state.phase))
}

export async function discardStaleDownloadedUpdate() {
  await releaseUpdaterResource()
  clearDeferredReminder()
  state.phase = 'idle'
  state.version = ''
  state.downloadedBytes = 0
  state.totalBytes = null
  state.error = ''
  state.reminderNeedsDownload = false
}

export async function resetSoftwareUpdaterStateForTests() {
  await releaseUpdaterResource()
  clearDeferredReminder()
  state.phase = 'idle'
  state.version = ''
  state.downloadedBytes = 0
  state.totalBytes = null
  state.error = ''
  state.reminderNeedsDownload = false
  inFlight = null
}

async function releaseUpdaterResource() {
  const resource = updateResource
  updateResource = null
  if (resource) await resource.close().catch(() => undefined)
}

function writeDeferredReminder(version: string) {
  getStorage()?.setItem(DEFERRED_KEY, JSON.stringify({ version, recordedAt: new Date().toISOString(), needsDownload: true }))
}

function readDeferredReminder(): { version: string } | null {
  try {
    const parsed = JSON.parse(getStorage()?.getItem(DEFERRED_KEY) || '') as Record<string, unknown>
    return typeof parsed.version === 'string' && parsed.version ? { version: parsed.version } : null
  } catch {
    return null
  }
}

function clearDeferredReminder() {
  getStorage()?.removeItem(DEFERRED_KEY)
}

function getStorage() {
  try {
    return typeof window === 'undefined' ? undefined : window.localStorage
  } catch {
    return undefined
  }
}

function normalizeUpdaterError(error: unknown, fallback: string) {
  return error instanceof Error && error.message ? error.message : fallback
}
