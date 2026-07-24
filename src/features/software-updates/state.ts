import type { SoftwareRelease, UpdateCheckResult } from '@/features/software-updates/types'
import { requestSoftwareUpdate } from '@/services/software-updates'

const DISMISSED_NORMAL_KEY = 'cs2-bot-improver.dismissed-update.v1'
const dismissedRecommended = new Set<string>()
let inFlight: Promise<UpdateCheckResult> | null = null

export function checkForSoftwareUpdates(manual = false) {
  if (inFlight) return inFlight
  inFlight = requestSoftwareUpdate({ manual }).finally(() => { inFlight = null })
  return inFlight
}

export function shouldPresentRelease(release: SoftwareRelease, manual: boolean) {
  if (manual || release.isCritical || release.severity === 'critical') return true
  if (release.severity === 'recommended') return !dismissedRecommended.has(release.version)
  return getStorage()?.getItem(DISMISSED_NORMAL_KEY) !== release.version
}

export function dismissRelease(release: SoftwareRelease) {
  if (release.severity === 'recommended') dismissedRecommended.add(release.version)
  if (release.severity === 'normal') getStorage()?.setItem(DISMISSED_NORMAL_KEY, release.version)
}

export function resetSoftwareUpdateStateForTests() {
  inFlight = null
  dismissedRecommended.clear()
  getStorage()?.removeItem(DISMISSED_NORMAL_KEY)
}

function getStorage() {
  try {
    return typeof window === 'undefined' ? undefined : window.localStorage
  } catch {
    return undefined
  }
}
