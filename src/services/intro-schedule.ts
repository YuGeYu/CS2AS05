/** Startup playback policy for the 0.5.14 acknowledgement scene. */
const LAST_FORCED_DAY_KEY = 'cs2as05.intro.last-forced-day.v1'
let memoryDay: string | null = null

function storage(): Storage | null {
  try { return typeof window === 'undefined' ? null : window.localStorage } catch { return null }
}

/** Returns the calendar day in Beijing, independent of the user's system timezone. */
export function beijingDay(date = new Date()): string {
  return new Intl.DateTimeFormat('en-CA', {
    timeZone: 'Asia/Shanghai', year: 'numeric', month: '2-digit', day: '2-digit',
  }).format(date)
}

export function shouldPlayStartupIntro(skipIntro: boolean, date = new Date()): boolean {
  if (!skipIntro) return true
  return (storage()?.getItem(LAST_FORCED_DAY_KEY) ?? memoryDay) !== beijingDay(date)
}

/** Mark only the forced daily playback; user-enabled playback remains preference driven. */
export function markStartupIntroPlayed(date = new Date()): void {
  const day = beijingDay(date); memoryDay = day
  try { storage()?.setItem(LAST_FORCED_DAY_KEY, day) } catch { /* restricted WebViews keep the in-process marker */ }
}
