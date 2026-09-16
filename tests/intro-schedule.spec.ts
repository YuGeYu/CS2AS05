import { describe, expect, it } from 'vitest'
import { beijingDay, markStartupIntroPlayed, shouldPlayStartupIntro } from '@/services/intro-schedule'

describe('0.5.14 startup intro schedule', () => {
  it('uses Beijing calendar day regardless of host offset', () => {
    expect(beijingDay(new Date('2026-09-11T16:30:00.000Z'))).toBe('2026-09-12')
  })
  it('forces the first skipped-intro launch once per Beijing day', () => {
    const date = new Date('2026-09-12T01:00:00.000Z')
    expect(shouldPlayStartupIntro(true, date)).toBe(true)
    markStartupIntroPlayed(date)
    expect(shouldPlayStartupIntro(true, date)).toBe(false)
    expect(shouldPlayStartupIntro(true, new Date('2026-09-12T15:59:00.000Z'))).toBe(false)
    expect(shouldPlayStartupIntro(true, new Date('2026-09-12T16:01:00.000Z'))).toBe(true)
  })
  it('always plays when the user did not choose skip', () => {
    expect(shouldPlayStartupIntro(false)).toBe(true)
  })
})
