import type { PerformanceRadarDimension, PerformanceRadarPlayer } from '@/types/demo'

export const PERFORMANCE_RADAR_MODEL_VERSION = 'performance-radar-v1'
export const RADAR_DIMENSION_KEYS = ['firepower', 'damage', 'survival', 'participation', 'teamwork', 'opening'] as const

export type RadarDimensionKey = (typeof RADAR_DIMENSION_KEYS)[number]

export interface RadarPoint { x: number; y: number }

export interface RadarNormalization { score: number | null; benchmark: number | null; quality: 'complete' | 'partial' | 'unavailable'; warning?: string }

/** Normalize one axis against the full roster's second-highest finite value. */
export function normalizeRadarAxis(raw: number | null, cohort: Array<number | null>, safeMax = RADAR_SAFE_SCORE_MAX): RadarNormalization {
  const valid = cohort.filter((value): value is number => value != null && Number.isFinite(value) && value >= 0).sort((a, b) => b - a)
  if (raw == null || !Number.isFinite(raw) || raw < 0 || !valid.length) return { score: null, benchmark: null, quality: 'unavailable' }
  const benchmark = valid.length > 1 ? valid[1]! : valid[0]!
  if (benchmark <= 0) return { score: 0, benchmark, quality: 'partial', warning: '本轴全场有效值均为 0，按 flat 处理。' }
  const score = raw / benchmark * 100
  if (score > safeMax) return { score: safeMax, benchmark, quality: 'partial', warning: `本轴比例超过安全上限 ${safeMax}，已保护性截断。` }
  return { score, benchmark, quality: 'complete' }
}

const TAU = Math.PI * 2
export const RADAR_MIN_DISPLAY_MAX = 140
export const RADAR_SAFE_SCORE_MAX = 500

export function radarScaleMax(values: number[]): number {
  const maximum = values.filter(Number.isFinite).reduce((max, value) => Math.max(max, value), 0)
  return Math.min(RADAR_SAFE_SCORE_MAX, Math.max(RADAR_MIN_DISPLAY_MAX, Math.ceil(maximum / 20) * 20))
}

export function radarPoint(index: number, value: number, radius = 222, center = 320, scaleMax = RADAR_MIN_DISPLAY_MAX): RadarPoint {
  const angle = -Math.PI / 2 + (index * TAU) / RADAR_DIMENSION_KEYS.length
  const bounded = Number.isFinite(value) ? Math.min(RADAR_SAFE_SCORE_MAX, Math.max(0, value)) : 0
  const safeScaleMax = Math.max(1, Math.min(RADAR_SAFE_SCORE_MAX, scaleMax))
  const distance = radius * bounded / safeScaleMax
  return {
    x: center + Math.cos(angle) * distance,
    y: center + Math.sin(angle) * distance,
  }
}

export function radarGrid(level: number, radius = 222, center = 320, scaleMax = RADAR_MIN_DISPLAY_MAX): string {
  return RADAR_DIMENSION_KEYS.map((_, index) => radarPoint(index, level, radius, center, scaleMax))
    .map(point => `${point.x.toFixed(2)},${point.y.toFixed(2)}`)
    .join(' ')
}

export function radarPolygon(dimensions: PerformanceRadarDimension[], radius = 222, center = 320, scaleMax = RADAR_MIN_DISPLAY_MAX): string | null {
  const byKey = new Map(dimensions.map(dimension => [dimension.key, dimension]))
  const values = RADAR_DIMENSION_KEYS.map(key => byKey.get(key)?.score ?? null)
  if (values.some(value => value == null || !Number.isFinite(value))) return null
  return values.map((value, index) => radarPoint(index, value as number, radius, center, scaleMax))
    .map(point => `${point.x.toFixed(2)},${point.y.toFixed(2)}`)
    .join(' ')
}

export function radarSegments(dimensions: PerformanceRadarDimension[], radius = 222, center = 320, scaleMax = RADAR_MIN_DISPLAY_MAX): string[] {
  const byKey = new Map(dimensions.map(dimension => [dimension.key, dimension]))
  return RADAR_DIMENSION_KEYS.flatMap((key, index) => {
    const nextIndex = (index + 1) % RADAR_DIMENSION_KEYS.length
    const value = byKey.get(key)?.score
    const nextValue = byKey.get(RADAR_DIMENSION_KEYS[nextIndex]!)?.score
    if (value == null || nextValue == null || !Number.isFinite(value) || !Number.isFinite(nextValue)) return []
    const first = radarPoint(index, value, radius, center, scaleMax)
    const second = radarPoint(nextIndex, nextValue, radius, center, scaleMax)
    return [`${first.x.toFixed(2)},${first.y.toFixed(2)} ${second.x.toFixed(2)},${second.y.toFixed(2)}`]
  })
}

export function radarLabelPoint(index: number): RadarPoint {
  return radarPoint(index, 100, 274, 320, 140)
}

export function formatRadarRaw(dimension: PerformanceRadarDimension): string {
  if (dimension.raw == null || !Number.isFinite(dimension.raw)) return '--'
  if (dimension.unit === '百分比') return `${(dimension.raw * 100).toFixed(1)}%`
  return dimension.raw.toFixed(2)
}

export function formatRadarScore(score: number | null): string {
  return score == null || !Number.isFinite(score) ? '--' : Math.round(Math.min(RADAR_SAFE_SCORE_MAX, Math.max(0, score))).toString()
}

export function stableRadarCandidates(players: PerformanceRadarPlayer[]): PerformanceRadarPlayer[] {
  return [...players].sort((left, right) =>
    Number(left.isBot) - Number(right.isBot)
    || (left.teamNumber ?? 99) - (right.teamNumber ?? 99)
    || left.stableKey.localeCompare(right.stableKey),
  )
}

export const radarStroke = (index: number) => ['#42b8d7', '#d3a95b', '#e47771'][index % 3]
export const radarDash = (index: number) => ['', '12 7', '3 6'][index % 3]
