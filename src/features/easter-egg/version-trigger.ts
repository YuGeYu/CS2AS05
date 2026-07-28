export function easterEggThreshold(version: string): number {
  const core = version.split('-')[0] ?? ''
  const minor = Number.parseInt(core.split('.')[1] ?? '', 10)
  return Number.isFinite(minor) && minor > 0 ? minor : 5
}

export interface VersionTriggerState {
  count: number
  firstClickAt: number | null
  lastClickAt: number | null
}

export interface VersionTriggerResult extends VersionTriggerState {
  unlocked: boolean
}

export function registerVersionClick(
  state: VersionTriggerState,
  version: string,
  now: number,
): VersionTriggerResult {
  const continued = state.lastClickAt !== null
    && now - state.lastClickAt <= 4_000
    && state.firstClickAt !== null
    && now - state.firstClickAt <= 12_000
  const count = continued ? state.count + 1 : 1
  if (count >= easterEggThreshold(version)) {
    return { count: 0, firstClickAt: null, lastClickAt: null, unlocked: true }
  }
  return {
    count,
    firstClickAt: continued ? state.firstClickAt : now,
    lastClickAt: now,
    unlocked: false,
  }
}
