import { describe, expect, it } from 'vitest'
import { cpuPercent, linearSlope, mergeCandidate, percentile } from '../scripts/preflight-metrics.mjs'

describe('preflight metric aggregation', () => {
  it('uses nearest-rank P95 including small samples', () => {
    expect(percentile([], 0.95)).toBeNull()
    expect(percentile([7], 0.95)).toBe(7)
    expect(percentile([1, 2, 3, 4, 5], 0.95)).toBe(5)
    expect(percentile(Array.from({ length: 20 }, (_, index) => index + 1), 0.95)).toBe(19)
  })
  it('calculates normalized CPU deltas and rejects missing samples', () => {
    expect(cpuPercent(1, 1.8, 2, 4)).toBeCloseTo(10)
    expect(cpuPercent(1, 2, 0, 4)).toBeNull()
  })
  it('calculates a linear slope and preserves missing values', () => {
    expect(linearSlope([10, 12, 14, 16])).toBeCloseTo(2)
    expect(linearSlope([1])).toBeNull()
    expect(linearSlope([1, Number.NaN])).toBeNull()
  })
  it('refuses different hashes and session ids', () => {
    expect(() => mergeCandidate('A', [{ exeSha256: 'B', sessionId: 'one' }])).toThrow('hash')
    expect(() => mergeCandidate('A', [{ exeSha256: 'A', sessionId: 'one' }, { exeSha256: 'A', sessionId: 'two' }])).toThrow('session')
    expect(mergeCandidate('A', [{ exeSha256: 'A', sessionId: 'one' }]).sessionId).toBe('one')
  })
})
