export const SAFETY_CONFIRMATION_KEY = 'cs2as:skin-forge:safety-confirmed-date:v1'

export function localDateKey(date = new Date()): string {
  const year = date.getFullYear()
  const month = String(date.getMonth() + 1).padStart(2, '0')
  const day = String(date.getDate()).padStart(2, '0')
  return `${year}-${month}-${day}`
}

function safeStorage(): Storage | undefined {
  try { return window.localStorage } catch { return undefined }
}

export function hasConfirmedSafetyToday(storage: Storage | undefined = safeStorage(), date = new Date()): boolean {
  if (!storage) return false
  try { return storage.getItem(SAFETY_CONFIRMATION_KEY) === localDateKey(date) } catch { return false }
}

export function confirmSafetyToday(storage: Storage | undefined = safeStorage(), date = new Date()): void {
  if (!storage) return
  try { storage.setItem(SAFETY_CONFIRMATION_KEY, localDateKey(date)) } catch { /* storage is optional */ }
}
