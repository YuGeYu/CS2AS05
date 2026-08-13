import { describe, expect, it } from 'vitest'
import manifest from '@/features/skin-forge/data/generated/catalog-manifest.json'
import { keychainCatalog, loadStickerCatalog, weaponCatalog, weaponSkinCatalog } from '@/features/skin-forge/data/catalog'
import { fromPlayerSkinModFile, toPlayerSkinModFile } from '@/features/skin-forge/player-skin-mod-adapter'
import golden from './fixtures/skin-forge-v1.8.2-golden.json'

describe('CS2-Skin-Forge v1.8.2 fixed catalog', () => {
  it('contains the complete fixed catalog and v1.8.1 additions', async () => {
    expect(weaponCatalog).toHaveLength(manifest.counts.weapons)
    expect(keychainCatalog).toHaveLength(manifest.counts.keychains)
    expect(manifest.counts.maxWeaponPaintKit).toBeGreaterThanOrEqual(1477)
    const stickers = await loadStickerCatalog()
    expect(stickers).toHaveLength(manifest.counts.stickers)
    expect(stickers.filter((item) => item.numericId >= 11174 && item.numericId <= 11193)).toHaveLength(20)
    expect(weaponCatalog.some((weapon) => weaponSkinCatalog(weapon.numericId).some((item) => item.numericId === 1477))).toBe(true)
  })

  it('keeps picker DOM bounded for the 10k sticker catalog', async () => {
    const stickers = await loadStickerCatalog()
    expect(stickers.length).toBeGreaterThan(10_000)
    expect(stickers.slice(0, 100)).toHaveLength(100)
  })

  it('round-trips the v1.8.2 golden contract without team crossover or unknown-field loss', () => {
    const internal = fromPlayerSkinModFile(golden)
    expect(internal.weapons.ct.weapon_7?.paintKit).toBe(600)
    expect(internal.weapons.t.weapon_7?.paintKit).toBe(675)
    expect(internal.weapons.ct.weapon_7?.stickers).toHaveLength(5)
    const output = toPlayerSkinModFile(internal)['0']!
    expect(output.weaponPaintsCt['7']).toBe(600)
    expect(output.weaponPaintsT['7']).toBe(675)
    expect(output.weaponStickers['7']).toHaveLength(5)
    expect(output.futureField).toEqual({ preserved: true })
  })
})
