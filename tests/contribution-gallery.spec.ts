import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { STATIC_REFERENCE_PROJECTS } from '@/features/intro/static-data'

const read = (path: string) => readFileSync(path, 'utf8')

describe('contribution gallery', () => {
  it('renders a static contribution archive without WebGL', () => {
    const view = read('src/components/easter-egg/EasterEggGame.vue')
    expect(view).toContain('贡献陈列馆')
    expect(view).not.toContain('WASD / 方向键行走')
    expect(view).toContain('<table')
    expect(view).not.toContain('ThreeStage')
    expect(STATIC_REFERENCE_PROJECTS.length).toBeGreaterThan(0)
  })
})
