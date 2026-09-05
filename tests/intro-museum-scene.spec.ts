import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'
import { STATIC_REFERENCE_PROJECTS } from '@/features/intro/static-data'

const intro = readFileSync('src/components/intro/StartupIntro.vue', 'utf8')
const gallery = readFileSync('src/components/easter-egg/EasterEggGame.vue', 'utf8')

describe('intro museum scene contract', () => {
  it('uses a static acknowledgement archive without remote loading', () => {
    expect(intro).toContain('STATIC_REFERENCE_PROJECTS')
    expect(intro).not.toContain('ThreeStage')
    expect(gallery).toContain('STATIC_REFERENCE_PROJECTS')
    expect(gallery).not.toContain('ThreeStage')
    expect(intro).not.toContain('loadIntroData')
    expect(gallery).not.toContain('loadIntroData')
  })

  it('keeps the fixed project archive populated', () => {
    expect(STATIC_REFERENCE_PROJECTS.length).toBeGreaterThan(0)
    expect(STATIC_REFERENCE_PROJECTS.map(item => item.repository)).toContain('ed0ard/CS2-Bot-Improver')
  })

  it('renders table semantics for both surfaces', () => {
    expect(intro).toContain('<table')
    expect(gallery).toContain('<table')
  })
})
