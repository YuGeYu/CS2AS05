import { relaunch } from '@tauri-apps/plugin-process'
import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater'

export type UpdaterResource = Update
export type UpdaterDownloadEvent = DownloadEvent

export async function checkTauriUpdater() {
  return check({ timeout: 10_000 })
}

export async function relaunchAfterUpdate() {
  await relaunch()
}
