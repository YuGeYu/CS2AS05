import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'
import { clampGalleryPosition, GALLERY_RADIUS } from '@/features/easter-egg/game-scene'

const read = (path: string) => readFileSync(path, 'utf8')

describe('contribution gallery', () => {
  it('keeps first-person movement inside the gallery floor', () => {
    expect(clampGalleryPosition(3, 4)).toEqual({ x: 3, z: 4 })
    const edge = clampGalleryPosition(99, 0)
    expect(edge.x).toBeCloseTo(GALLERY_RADIUS)
    expect(edge.z).toBe(0)
  })

  it('replaces Qingming sword game with the contribution gallery', () => {
    const view = read('src/components/easter-egg/EasterEggGame.vue')
    const scene = read('src/features/easter-egg/game-scene.ts')
    expect(view).toContain('贡献陈列馆')
    expect(view).toContain('WASD / 方向键行走')
    expect(view).not.toContain('青冥试剑')
    expect(scene).toContain('tang-painted-court-lady.glb')
    expect(scene).toContain('KTX2Loader')
    expect(scene).toContain("setTranscoderPath('/museum/contribution-gallery/basis/')")
    expect(scene).toContain('fittedBox.min.y')
    expect(scene).not.toContain('tang-mingguang-armor.glb')
    expect(scene).not.toContain('yue-wang-jian.glb')
    expect(scene).toContain('artifactKeyLight')
    expect(scene).toContain('artifactRimLight')
    expect(scene).toContain('artifactRotation += delta * 0.12')
    expect(scene).not.toContain('artifactRoot.rotation.y = time')
    expect(view).toContain('唐代彩绘仕女俑')
    expect(view).not.toContain('唐代明光铠')
    expect(view).not.toContain('越王勾践剑')
    expect(scene).toContain('back.rotation.y = Math.PI')
  })
})
