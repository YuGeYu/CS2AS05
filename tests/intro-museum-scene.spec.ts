import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

const scene = readFileSync('src/features/intro/scene.ts', 'utf8')
const intro = readFileSync('src/components/intro/StartupIntro.vue', 'utf8')

describe('intro museum scene contract', () => {
  it('uses a fixed camera timeline and a supporter snapshot lock', () => {
    expect(intro).toContain('const SUPPORTER_LOCK_MS = 2_800')
    expect(intro).toContain('lockedCards.value = structuredClone(candidateCards.value)')
    expect(intro).toContain("if (!closed && !locked.value) data.value = value")
    expect(scene).toContain('function cameraAt(state: IntroSceneState)')
    expect(scene).toContain('camera.position.set')
    expect(scene).not.toContain('OrbitControls')
  })

  it('builds a procedural hall and fully releases WebGL resources', () => {
    expect(scene).toContain('new THREE.CircleGeometry')
    expect(scene).toContain('new THREE.CylinderGeometry')
    expect(scene).toContain('renderer.forceContextLoss()')
    expect(scene).not.toContain('GLTFLoader')
    expect(scene).not.toContain('DRACOLoader')
  })
})
