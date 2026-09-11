import type { IntroAcknowledgementCard } from '@/features/intro/types'

export interface PlaqueLayoutSlot {
  id: string
  x: number
  z: number
  rotationY: number
  order: number
}

/** Quantity-adaptive plaque placement. It grows the path instead of shrinking cards. */
export function buildPlaqueLayout(cards: IntroAcknowledgementCard[]): PlaqueLayoutSlot[] {
  const records = cards.length ? cards : [{ id: 'fallback', kind: 'supporter' as const, eyebrow: '鸣谢同路人', title: '致每一位同路人', message: '长夜执剑，幸与诸君同路。', detail: 'CS2AS', updatedAt: '' }]
  const gap = 4.35
  const lanes = Math.min(3, Math.max(1, Math.ceil(records.length / 12)))
  const perLane = Math.ceil(records.length / lanes)
  const totalLength = Math.max(13, (perLane - 1) * gap)
  return records.map((card, index) => {
    const lane = Math.min(lanes - 1, Math.floor(index / perLane))
    const laneIndex = index - lane * perLane
    const x = (laneIndex - (Math.min(perLane, records.length - lane * perLane) - 1) / 2) * gap
    const z = (lane - (lanes - 1) / 2) * 5.8 + Math.sin(laneIndex * 0.7) * 0.45
    const tangent = laneIndex === 0 ? 0 : Math.atan2(0.45 * Math.cos(laneIndex * 0.7), gap)
    return { id: card.id, x: x * (totalLength / Math.max(13, (perLane - 1) * gap)), z, rotationY: tangent, order: index }
  })
}
