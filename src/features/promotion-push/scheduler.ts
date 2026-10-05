import type { PromotionKind } from './types'

export const PROMOTION_INTERVAL_MS = 24 * 60 * 60 * 1_000
export const PROMOTION_FIRST_DELAY_MS = 20 * 60 * 1_000

export function duePromotionCount(startedAt: number, now = Date.now()): number {
  if (!Number.isFinite(startedAt) || now <= startedAt) return 0
  if (now - startedAt < PROMOTION_FIRST_DELAY_MS) return 0
  return 1
}

export function choosePromotionKind(random = Math.random()): PromotionKind {
  return random < 0.5 ? 'resource' : 'local'
}

export function nextIndex(current: number, length: number): number {
  if (length <= 0) return 0
  return ((current % length) + length) % length
}

