import type { IntroAcknowledgementCard } from '@/features/intro/types'

export interface PlaqueLayoutSlot {
  id: string
  x: number
  z: number
  rotationY: number
  order: number
}

/** Quantity-adaptive plaque placement. Every card gets its own longitudinal bay. */
export function buildPlaqueLayout(cards: IntroAcknowledgementCard[]): PlaqueLayoutSlot[] {
  const records = cards.length ? cards : [{ id: 'fallback', kind: 'supporter' as const, eyebrow: '鸣谢同路人', title: '致每一位同路人', message: '长夜执剑，幸与诸君同路。', detail: 'CS2AS', updatedAt: '' }]
  const gap = 5.2
  const center = (records.length - 1) / 2
  return records.map((card, index) => ({
    id: card.id,
    x: (index - center) * gap,
    z: Math.sin(index * 0.72) * 1.15,
    rotationY: Math.sin(index * 0.72) * 0.045,
    order: index,
  }))
}
