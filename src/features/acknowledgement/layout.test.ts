import { describe, expect, it } from 'vitest'
import { buildPlaqueLayout } from './layout'
import type { IntroAcknowledgementCard } from '@/features/intro/types'

function cards(count: number): IntroAcknowledgementCard[] {
  return Array.from({ length: count }, (_, index) => ({ id: `card-${index}`, kind: index % 2 ? 'supporter' : 'upstream', eyebrow: '测试', title: `牌子 ${index}`, message: '测试', detail: '测试', updatedAt: '' }))
}

describe('buildPlaqueLayout', () => {
  it('keeps a stable slot for an empty dataset', () => {
    expect(buildPlaqueLayout([])).toHaveLength(1)
  })

  it('returns one slot per record without duplicate ids', () => {
    const result = buildPlaqueLayout(cards(120))
    expect(result).toHaveLength(120)
    expect(new Set(result.map(item => item.id)).size).toBe(120)
  })

  it('spreads large datasets across bounded lanes', () => {
    const result = buildPlaqueLayout(cards(120))
    expect(new Set(result.map(item => item.z)).size).toBeGreaterThan(1)
    expect(result.every(item => Number.isFinite(item.x) && Number.isFinite(item.z))).toBe(true)
  })
})
