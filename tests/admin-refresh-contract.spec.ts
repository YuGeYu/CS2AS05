import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('official admin refresh contract', () => {
  it('refreshes on return and explicit actions without fixed polling', () => {
    const admin = readFileSync('E:/cs2as/src/pages/AdminPage.vue', 'utf8')
    expect(admin).not.toContain('setInterval')
    expect(admin).toContain("document.addEventListener('visibilitychange', refreshWhenReturning)")
    expect(admin).toContain("window.addEventListener('focus', refreshWhenReturning)")
    expect(admin).toContain('Date.now() - lastRefreshEpoch >= 60_000')
    expect(admin).toContain('立即同步')
  })
})
