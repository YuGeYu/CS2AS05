import { describe, expect, it } from 'vitest'
import { RADAR_DIMENSION_KEYS, formatRadarRaw, formatRadarScore, normalizeRadarAxis, radarGrid, radarPoint, radarPolygon, radarScaleMax, radarSegments, stableRadarCandidates } from '@/features/demo/radar/performance-radar'
import type { PerformanceRadarDimension, PerformanceRadarPlayer } from '@/types/demo'

const dimension = (key: PerformanceRadarDimension['key'], score: number | null, raw = score): PerformanceRadarDimension => ({
  key, label: key, score, raw, rawLabel: '', unit: 'KPR', benchmark: 1, source: 'test', quality: score == null ? 'unavailable' : 'complete',
})
const player = (stableKey: string, isBot: boolean, teamNumber: number | null): PerformanceRadarPlayer => ({
  stableKey, name: stableKey, isBot, teamNumber, teamName: null, roundsPlayed: 10,
  rawStats: { participatedRounds: 10, kills: 1, deaths: 1, assists: 1, damageHealth: 1, survivedRounds: 1, kastRounds: 1, multiKillRounds: 0, firstKills: 0, firstDeaths: 0, tradeKills: 0 },
  dimensions: [], warnings: [],
})

describe('performance radar geometry', () => {
  it('uses exactly six independent dimensions and no Rating axis', () => {
    expect(RADAR_DIMENSION_KEYS).toEqual(['firepower', 'damage', 'survival', 'participation', 'teamwork', 'opening'])
    expect(RADAR_DIMENSION_KEYS).not.toContain('rating')
  })

  it('keeps high scores distinct while protecting geometry from invalid values', () => {
    expect(radarPoint(0, 150, 222, 320, 460)).not.toEqual(radarPoint(0, 452.5, 222, 320, 460))
    expect(formatRadarScore(452.5)).toBe('453')
    expect(radarScaleMax([452.5, 105.04])).toBe(460)
    expect(radarScaleMax([105])).toBe(140)
    expect(radarPoint(0, 120)).not.toEqual(radarPoint(0, 100))
    expect(radarPoint(0, -30)).toEqual(radarPoint(0, 0))
    expect(radarPoint(0, Number.NaN)).toEqual(radarPoint(0, 0))
    expect(radarGrid(100)).not.toMatch(/NaN|Infinity/)
  })

  it('does not close a polygon by replacing an unavailable axis with zero', () => {
    const values = RADAR_DIMENSION_KEYS.map(key => dimension(key, 50))
    expect(radarPolygon(values)).toBeTruthy()
    values[2] = dimension('survival', null)
    expect(radarPolygon(values)).toBeNull()
    expect(radarSegments(values)).toHaveLength(4)
  })

  it('changes only the requested axis point', () => {
    const baseline = RADAR_DIMENSION_KEYS.map(key => dimension(key, 50))
    const changed = baseline.map(item => item.key === 'damage' ? dimension('damage', 100) : item)
    const before = radarPolygon(baseline)?.split(' ')
    const after = radarPolygon(changed)?.split(' ')
    expect(after?.filter((point, index) => point !== before?.[index])).toHaveLength(1)
  })

  it('keeps precise formatting and deterministic non-Rating selection order', () => {
    expect(formatRadarRaw({ ...dimension('survival', 80, 0.625), unit: '百分比' })).toBe('62.5%')
    expect(formatRadarRaw(dimension('firepower', null, null))).toBe('--')
    expect(stableRadarCandidates([player('z-bot', true, 2), player('b', false, 3), player('a', false, 2)]).map(item => item.stableKey)).toEqual(['a', 'b', 'z-bot'])
  })

  it('normalizes against the full cohort second-highest value', () => {
    expect(normalizeRadarAxis(150, [150, 100, 50]).score).toBe(150)
    expect(normalizeRadarAxis(100, [150, 100, 50]).benchmark).toBe(100)
    expect(normalizeRadarAxis(50, [150, 100, 50]).score).toBe(50)
    expect(normalizeRadarAxis(100, [100, 100, 50]).score).toBe(100)
    expect(normalizeRadarAxis(0, [0, 0]).quality).toBe('partial')
    expect(normalizeRadarAxis(10, [10]).score).toBe(100)
    expect(normalizeRadarAxis(-1, [10, 5]).score).toBeNull()
    expect(normalizeRadarAxis(500, [500, 1], 400).score).toBe(400)
  })
})
