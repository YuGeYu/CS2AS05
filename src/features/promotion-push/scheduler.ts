import type { PromotionKind } from './types'

export const PROMOTION_INTERVAL_MS = 10 * 60 * 1_000

export function duePromotionCount(startedAt: number, now = Date.now()): number {
  if (!Number.isFinite(startedAt) || now <= startedAt) return 0
  return Math.floor((now - startedAt) / PROMOTION_INTERVAL_MS)
}

export function choosePromotionKind(random = Math.random()): PromotionKind {
  return random < 0.5 ? 'resource' : 'local'
}

export function nextIndex(current: number, length: number): number {
  if (length <= 0) return 0
  return ((current % length) + length) % length
}

