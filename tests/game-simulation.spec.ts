import { describe, expect, it, vi } from 'vitest'
import { distanceToSegment, GameSimulation, TARGET_SPAWN_Z, TARGET_SPEED } from '@/features/easter-egg/game-scene'

function randomSequence(values: number[]) {
  let index = 0
  return () => values[index++] ?? 0.5
}

describe('easter egg simulation', () => {
  it('does not spawn or advance targets while inactive', () => {
    const simulation = new GameSimulation(() => 0.5)
    simulation.step(0.05)
    expect(simulation.targets).toHaveLength(0)
    simulation.setActive(true)
    expect(simulation.targets[0]?.z).toBe(TARGET_SPAWN_Z)
    simulation.setActive(false)
    simulation.step(0.05)
    expect(simulation.targets[0]?.z).toBe(TARGET_SPAWN_Z)
  })

  it('advances deterministically and leaves at least three seconds before a miss', () => {
    const simulation = new GameSimulation(() => 0.5)
    simulation.setActive(true)
    simulation.step(0.05)
    expect(simulation.targets[0]?.z).toBeCloseTo(TARGET_SPAWN_Z + TARGET_SPEED * 0.05)
    for (let index = 1; index < 60; index += 1) simulation.step(0.05)
    expect(simulation.lives).toBe(3)
  })

  it('does not exhaust all lives during a ten second idle animation check', () => {
    const simulation = new GameSimulation(() => 0.5)
    simulation.setActive(true)
    for (let index = 0; index < 200; index += 1) simulation.step(0.05)
    expect(simulation.lives).toBeGreaterThan(0)
  })

  it('scores a jade line hit once and rejects repeat moves', () => {
    const onChange = vi.fn()
    const simulation = new GameSimulation(randomSequence([0.5, 0.5, 0.5]), onChange)
    simulation.setActive(true)
    const id = simulation.targets[0]!.id
    expect(distanceToSegment({ x: 0.5, y: 0.5 }, { x: 0.3, y: 0.5 }, { x: 0.7, y: 0.5 })).toBe(0)
    expect(simulation.slash({ x: 0.3, y: 0.5 }, { x: 0.7, y: 0.5 }, [{ id, x: 0.5, y: 0.5, radius: 0.08 }])).toHaveLength(1)
    expect(simulation.score).toBe(110)
    expect(simulation.slash({ x: 0.3, y: 0.5 }, { x: 0.7, y: 0.5 }, [{ id, x: 0.5, y: 0.5, radius: 0.08 }])).toHaveLength(0)
  })

  it('uses a bronze seal to clear combo without subtracting score', () => {
    const simulation = new GameSimulation(randomSequence([0.5, 0.5, 0.5, 0.1, 0.5, 0.5]))
    simulation.setActive(true)
    let id = simulation.targets[0]!.id
    simulation.slash({ x: 0.3, y: 0.5 }, { x: 0.7, y: 0.5 }, [{ id, x: 0.5, y: 0.5, radius: 0.08 }])
    for (let index = 0; index < 19; index += 1) simulation.step(0.05)
    id = simulation.targets[0]!.id
    simulation.slash({ x: 0.3, y: 0.5 }, { x: 0.7, y: 0.5 }, [{ id, x: 0.5, y: 0.5, radius: 0.08 }])
    expect(simulation.score).toBe(110)
    expect(simulation.combo).toBe(0)
  })

  it('reset restores the initial state', () => {
    const simulation = new GameSimulation(() => 0.5)
    simulation.setActive(true)
    simulation.reset()
    expect(simulation.targets).toEqual([])
    expect(simulation).toMatchObject({ score: 0, combo: 0, bestCombo: 0, lives: 3, active: false })
  })
})
