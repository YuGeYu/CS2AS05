import { describe, expect, it } from 'vitest'
import { fromPlayerSkinModFile, toPlayerSkinModFile } from '@/features/skin-forge/player-skin-mod-adapter'
import { createWeapon, DEFAULT_LOADOUT } from '@/types/skin-forge'

describe('skin forge selection mode contract', () => {
  it('preserves random versus custom music state, including unselected custom', () => {
    const random = toPlayerSkinModFile(structuredClone(DEFAULT_LOADOUT))['0']!
    expect(random).toMatchObject({ useRandom: true, musicKit: -1 })
    expect(fromPlayerSkinModFile({ 0: random })).toMatchObject({ mode: 'random', musicKit: null })

    const custom = structuredClone(DEFAULT_LOADOUT)
    custom.mode = 'custom'
    custom.musicKit = 42
    const selected = toPlayerSkinModFile(custom)['0']!
    expect(selected).toMatchObject({ useRandom: false, musicKit: 42 })
    expect(fromPlayerSkinModFile({ 0: selected })).toMatchObject({ mode: 'custom', musicKit: 42 })

    custom.musicKit = null
    const unselected = toPlayerSkinModFile(custom)['0']!
    expect(unselected).toMatchObject({ useRandom: false, musicKit: -1 })
    expect(fromPlayerSkinModFile({ 0: unselected })).toMatchObject({ mode: 'custom', musicKit: null })
  })

  it('serializes cancellation sentinels for knife, gloves, and agents', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    const item = toPlayerSkinModFile(loadout)['0']!
    expect(item).toMatchObject({ knifeIndexCt: -1, knifePaintCt: -1, gloveIndexCt: -1, gloveDefIndexCt: 0, agentModelCt: -1 })
  })

  it('omits unselected weapon paints so upstream random fallback remains active', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.mode = 'random'
    loadout.weapons.ct.weapon_7 = createWeapon(7)
    const item = toPlayerSkinModFile(loadout)['0']!
    expect(item.weaponPaintsCt).not.toHaveProperty('7')
    expect(item.weaponWearsCt).not.toHaveProperty('7')
  })
})
