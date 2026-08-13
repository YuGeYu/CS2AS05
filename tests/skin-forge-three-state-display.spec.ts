import { describe, expect, it } from 'vitest'
import { agentDisplay, gloveDisplay, knifeDisplay, musicDisplay, weaponDisplay } from '@/features/skin-forge/loadout-display'
import { weaponCatalog, weaponSkinCatalog } from '@/features/skin-forge/data/catalog'
import { createWeapon, DEFAULT_LOADOUT } from '@/types/skin-forge'

describe('skin forge three-state display', () => {
  it('shows global random for all five categories', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    expect(knifeDisplay(loadout, 'ct').label).toBe('每次重生随机')
    expect(gloveDisplay(loadout, 'ct').label).toBe('每次重生随机')
    expect(agentDisplay(loadout, 'ct').label).toBe('每次重生随机')
    expect(musicDisplay(loadout).label).toBe('每次重生随机')
    expect(weaponDisplay(loadout, 'ct', weaponCatalog[0]!).label).toBe('每次重生随机')
  })

  it('distinguishes custom values and custom empty values', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.mode = 'custom'
    expect(knifeDisplay(loadout, 'ct').label).toBe('未选择刀具')
    expect(gloveDisplay(loadout, 'ct').label).toBe('未选择手套')
    expect(agentDisplay(loadout, 'ct').label).toBe('未选择角色')
    expect(musicDisplay(loadout).label).toBe('未选择音乐盒')
    expect(weaponDisplay(loadout, 'ct', weaponCatalog[0]!).label).toContain('未选择')

    const weapon = weaponCatalog.find(item => item.numericId === 7)!
    loadout.weapons.ct.weapon_7 = { ...createWeapon(7), paintKit: weaponSkinCatalog(7)[0]!.numericId }
    loadout.musicKit = 7
    expect(weaponDisplay(loadout, 'ct', weapon).state).toBe('custom-value')
    expect(musicDisplay(loadout).state).toBe('custom-value')
  })

  it('keeps CT and T category values independent', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.mode = 'custom'
    loadout.ct.gloves.index = 0
    loadout.ct.gloves.defindex = 5027
    loadout.t.gloves.index = -1
    expect(gloveDisplay(loadout, 'ct').state).toBe('custom-value')
    expect(gloveDisplay(loadout, 't').state).toBe('custom-empty')
  })
})
