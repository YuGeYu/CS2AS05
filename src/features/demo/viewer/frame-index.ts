import type { PositionPoint } from '@/types/demo'

export const uniqueTicks = (points: readonly PositionPoint[]) => [...new Set(points.map(point => point.tick))].sort((a, b) => a - b)
export const pointsAtTick = (points: readonly PositionPoint[], tick: number) => points.filter(point => point.tick === tick)
