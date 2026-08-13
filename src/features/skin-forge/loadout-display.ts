import type { Loadout, Team, WeaponLoadout } from '@/types/skin-forge'
import { agentCatalog, gloveCatalog, knifeCatalog, musicCatalog, weaponCatalog, weaponSkinCatalog, type CatalogItem } from '@/features/skin-forge/data/catalog'

export type DisplayState = 'random' | 'custom-value' | 'custom-empty'
export interface DisplayValue { state: DisplayState; label: string; valueId?: string }

function value(mode: Loadout['mode'], concrete: CatalogItem | undefined, empty: string): DisplayValue {
  if (mode === 'random') return { state: 'random', label: '每次重生随机' }
  return concrete ? { state: 'custom-value', label: concrete.name, valueId: concrete.id } : { state: 'custom-empty', label: empty }
}

export function knifeDisplay(loadout: Loadout, team: Team): DisplayValue {
  const knife = loadout[team].knife
  const item = knife.index >= 0
    ? knifeCatalog.find(entry => Number(entry.meta?.index) === knife.index)
    : knifeCatalog.find(entry => entry.numericId === knife.defindex && knife.defindex > 42)
  return value(loadout.mode, item, '未选择刀具')
}

export function gloveDisplay(loadout: Loadout, team: Team): DisplayValue {
  const glove = loadout[team].gloves
  const item = glove.index >= 0 ? gloveCatalog.find(entry => Number(entry.meta?.index) === glove.index) : undefined
  return value(loadout.mode, item, '未选择手套')
}

export function agentDisplay(loadout: Loadout, team: Team): DisplayValue {
  const agent = loadout[team].agent
  const item = agent === null ? undefined : agentCatalog[team].find(entry => Number(entry.meta?.index) === agent)
  return value(loadout.mode, item, '未选择角色')
}

export function musicDisplay(loadout: Loadout): DisplayValue {
  const item = loadout.musicKit === null ? undefined : musicCatalog.find(entry => entry.numericId === loadout.musicKit)
  return value(loadout.mode, item, '未选择音乐盒')
}

export function weaponDisplay(loadout: Loadout, team: Team, item: CatalogItem): DisplayValue {
  const weapon: WeaponLoadout | undefined = loadout.weapons[team][`weapon_${item.numericId}`]
  const skin = weapon && weapon.paintKit >= 0
    ? weaponSkinCatalog(item.numericId).find(entry => entry.numericId === weapon.paintKit)
    : undefined
  return value(loadout.mode, skin, `未选择${item.name}皮肤`)
}

export function weaponCatalogDisplay(loadout: Loadout, team: Team): Array<DisplayValue & { item: CatalogItem }> {
  return weaponCatalog.map(item => ({ ...weaponDisplay(loadout, team, item), item }))
}
