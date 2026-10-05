import { describe, expect, it } from 'vitest'
import { choosePromotionKind, duePromotionCount, nextIndex, PROMOTION_INTERVAL_MS, PROMOTION_FIRST_DELAY_MS } from '@/features/promotion-push/scheduler'
import { readFileSync } from 'node:fs'

describe('promotion push schedule', () => {
  it('waits twenty minutes and shows at most one push per run', () => {
    const startedAt = 100_000
    expect(duePromotionCount(startedAt, startedAt + PROMOTION_FIRST_DELAY_MS - 1)).toBe(0)
    expect(duePromotionCount(startedAt, startedAt + PROMOTION_FIRST_DELAY_MS - 1)).toBe(0)
    expect(duePromotionCount(startedAt, startedAt + PROMOTION_FIRST_DELAY_MS)).toBe(1)
    expect(duePromotionCount(startedAt, startedAt + PROMOTION_INTERVAL_MS * 3 + 1)).toBe(1)
  })

  it('uses a 50/50 random split and cycles each library in order', () => {
    expect(choosePromotionKind(0.49)).toBe('resource')
    expect(choosePromotionKind(0.5)).toBe('local')
    expect(nextIndex(0, 2)).toBe(0)
    expect(nextIndex(2, 2)).toBe(0)
    expect(nextIndex(5, 10)).toBe(5)
  })
})

describe('promotion push UI contract', () => {
  it('keeps the push modal closable and contains both local push entries', () => {
    const component = readFileSync('src/components/PromotionPushModal.vue', 'utf8')
    const support = readFileSync('src-tauri/src/services/support.rs', 'utf8')
    expect(component).toContain('aria-modal="true"')
    expect(component).toContain('打开“凉拌娱乐”QQ群')
    expect(component).toContain('微信赞赏码')
    expect(support).toContain('https://qm.qq.com/q/DXLtk0KFby')
    expect(component).toContain('openCommunityGroupLink')
    expect(component).toContain('以后不再主动提醒')
    expect(component).not.toContain("openLink(groupUrl)")
  })
})
