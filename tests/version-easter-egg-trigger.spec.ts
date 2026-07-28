import { describe, expect, it } from 'vitest'
import { easterEggThreshold, registerVersionClick, type VersionTriggerState } from '@/features/easter-egg/version-trigger'

describe('version easter egg trigger', () => {
  it.each([['0.5.5', 5], ['0.6.0', 6], ['0.10.2', 10], ['0.5.5-beta.1', 5], ['broken', 5]])('parses %s as %i clicks', (version, expected) => {
    expect(easterEggThreshold(version)).toBe(expected)
  })

  it('unlocks on the fifth relaxed click and resets', () => {
    let state: VersionTriggerState = { count: 0, firstClickAt: null, lastClickAt: null }
    for (let index = 0; index < 4; index += 1) {
      const result = registerVersionClick(state, '0.5.5', index * 1_000)
      expect(result.unlocked).toBe(false)
      state = result
    }
    const result = registerVersionClick(state, '0.5.5', 4_000)
    expect(result).toMatchObject({ unlocked: true, count: 0, firstClickAt: null, lastClickAt: null })
  })

  it('counts the click after a long pause as the first click', () => {
    const result = registerVersionClick({ count: 3, firstClickAt: 0, lastClickAt: 2_000 }, '0.5.5', 6_001)
    expect(result).toMatchObject({ count: 1, firstClickAt: 6_001, lastClickAt: 6_001, unlocked: false })
  })
})
