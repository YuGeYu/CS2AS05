import { describe, expect, it } from 'vitest'
import { fromPlayerSkinModFile, toPlayerSkinModFile } from '@/features/skin-forge/player-skin-mod-adapter'
import { createWeapon, DEFAULT_LOADOUT, migrateLoadout } from '@/types/skin-forge'

describe('PlayerSkinMod loadout adapter', () => {
  it('writes slot 0 with independent CT and T weapon maps', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.weapons.ct.weapon_7 = createWeapon(7); loadout.weapons.t.weapon_7 = createWeapon(7)
    loadout.weapons.ct.weapon_7.paintKit = 600; loadout.weapons.ct.weapon_7.seed = 3; loadout.weapons.t.weapon_7.paintKit = 44
    loadout.weapons.ct.weapon_7.stickers = Array.from({ length: 7 }, (_, id) => ({ id: id + 1, wear: 0.1, offsetX: 0, offsetY: 0, scale: 1, rotation: 0 }))
    loadout.weapons.ct.weapon_7.keychain = { id: 12, offsetX: 0, offsetY: 0, offsetZ: 0, seed: 2 }; loadout.weapons.ct.weapon_7.nameTag = 'Local'; loadout.weapons.ct.weapon_7.statTrak = 12
    const file = toPlayerSkinModFile(loadout)
    expect(Object.keys(file)).toEqual(['0'])
    expect(file['0']!.weaponPaintsCt['7']).toBe(600)
    expect(file['0']!.weaponPaintsT['7']).toBe(44)
    expect(file['0']!.weaponStickers['7']).toHaveLength(5)
    expect(file['0']!.weaponKeychains['7']).toMatchObject({ id: 12, seed: 2 })
    expect(file['0']!.weaponNametags['7']).toBe('Local')
    expect(file['0']!.weaponStatTrak['7']).toEqual({ enabled: true, count: 12 })
  })

  it('maps knife, glove, agent, music and random fields', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.ct.knife = { defindex: 507, paintKit: 44, wear: 0.12, seed: 1 }
    loadout.t.knife = { defindex: 508, paintKit: 45, wear: 0.22, seed: 2 }
    loadout.ct.agent = 3; loadout.t.agent = 4; loadout.musicKit = 7; loadout.mode = 'random'
    const item = toPlayerSkinModFile(loadout)['0']!
    expect(item).toMatchObject({ knifeIndexCt: 507, knifeIndexT: 508, agentModelCt: 3, agentModelT: 4, musicKit: 7, useRandom: true })
  })

  it('round-trips shared stickers, keychain, nametag and StatTrak details', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.weapons.ct.weapon_7 = createWeapon(7)
    Object.assign(loadout.weapons.ct.weapon_7, { stickers: [{ id: 123, wear: 0.1, offsetX: 0, offsetY: 0, scale: 1.2, rotation: 4 }], keychain: { id: 9, offsetX: 0, offsetY: 0, offsetZ: 0, seed: 2 }, nameTag: '回读', statTrak: 18 })
    const result = fromPlayerSkinModFile(toPlayerSkinModFile(loadout))
    expect(result.weapons.ct.weapon_7).toMatchObject({ nameTag: '回读', statTrak: 18, keychain: { id: 9, seed: 2 } })
    expect(result.weapons.ct.weapon_7!.stickers[0]).toMatchObject({ id: 123, wear: 0.1, scale: 1.2, rotation: 4 })
  })

  it('reads legacy shared maps and clamps invalid editable values', () => {
    const result = fromPlayerSkinModFile({ 0: { weaponPaints: { 7: 600 }, weaponWears: { 7: 2 }, weaponSeeds: { 7: -4 }, knifeIndex: 507, knifePaint: 44, knifeWear: 0.2 } })
    expect(result.weapons.ct.weapon_7).toMatchObject({ paintKit: 600, wear: 1, seed: 0 })
    expect(result.weapons.t.weapon_7).toMatchObject({ paintKit: 600, wear: 1, seed: 0 })
    expect(result.ct.knife.defindex).toBe(507)
  })

  it('migrates first-round shared weapons without linking team objects', () => {
    const result = migrateLoadout({ schema: 2, weapons: { ak47: { ...createWeapon(7), paintKit: 0 } } })
    result.weapons.ct.weapon_7!.paintKit = 600
    expect(result.weapons.t.weapon_7!.paintKit).toBe(0)
  })

  it('rejects invalid weapon defindex', () => {
    const loadout = structuredClone(DEFAULT_LOADOUT)
    loadout.weapons.ct.weapon_7 = createWeapon(7); loadout.weapons.ct.weapon_7.defindex = 0
    expect(() => toPlayerSkinModFile(loadout)).toThrow('非法武器 defindex')
  })
})
