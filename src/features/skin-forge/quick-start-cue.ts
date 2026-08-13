export const QUICK_START_CUE_KEY = 'cs2as:skin-forge:quick-start-highlighted:v1'

let sessionConsumed = false

function defaultStorage(): Storage | null {
  try { return globalThis.localStorage } catch { return null }
}

export function shouldShowQuickStartCue(storage: Storage | null = defaultStorage()): boolean {
  if (sessionConsumed) return false
  try { return storage?.getItem(QUICK_START_CUE_KEY) !== '1' } catch { return true }
}

export function consumeQuickStartCue(storage: Storage | null = defaultStorage()) {
  sessionConsumed = true
  try { storage?.setItem(QUICK_START_CUE_KEY, '1') } catch { /* Storage availability must not block the workshop. */ }
}

export function resetQuickStartCueSessionForTests() { sessionConsumed = false }
