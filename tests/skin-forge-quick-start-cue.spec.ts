import { beforeEach, describe, expect, it } from 'vitest'
import { consumeQuickStartCue, QUICK_START_CUE_KEY, resetQuickStartCueSessionForTests, shouldShowQuickStartCue } from '@/features/skin-forge/quick-start-cue'

describe('skin forge quick start cue', () => {
  beforeEach(() => resetQuickStartCueSessionForTests())

  it('uses the fixed key and consumes the cue once', () => {
    const values = new Map<string, string>()
    const storage = { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => values.set(key, value) } as Storage
    expect(QUICK_START_CUE_KEY).toBe('cs2as:skin-forge:quick-start-highlighted:v1')
    expect(shouldShowQuickStartCue(storage)).toBe(true)
    consumeQuickStartCue(storage)
    expect(values.get(QUICK_START_CUE_KEY)).toBe('1')
    expect(shouldShowQuickStartCue(storage)).toBe(false)
  })

  it('fails open when storage throws but does not flash repeatedly in the session', () => {
    const storage = { getItem: () => { throw new Error('blocked') }, setItem: () => { throw new Error('blocked') } } as unknown as Storage
    expect(shouldShowQuickStartCue(storage)).toBe(true)
    expect(() => consumeQuickStartCue(storage)).not.toThrow()
    expect(shouldShowQuickStartCue(storage)).toBe(false)
  })

  it('does not block when storage is unavailable', () => {
    expect(shouldShowQuickStartCue(null)).toBe(true)
    expect(() => consumeQuickStartCue(null)).not.toThrow()
    expect(shouldShowQuickStartCue(null)).toBe(false)
  })
})
