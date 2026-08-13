import { allKeychains } from './generated/keychains'
import { knifeSkinsByType } from './generated/knifeSkins'
import { knives } from './generated/knives'
import { agentNameMap, getGloveLocalizedName, getGlovePaintLocalizedName, getLocalizedName, musicKitNameMap } from './generated/localNames'
import { agentModels, gloves, musicKits, weaponPaints } from './generated/skins'
import { skinNamesEn } from './generated/skinNamesEn'
import { getWeaponDefaultImage } from './generated/weaponImages'
import { weapons } from './generated/weapons'

export interface CatalogItem {
  id: string
  numericId: number
  name: string
  secondary?: string
  image: string
  search: string
  meta?: Record<string, string | number>
}

export const weaponCatalog: CatalogItem[] = weapons.map((weapon) => ({
  id: `weapon_${weapon.defindex}`,
  numericId: weapon.defindex,
  name: weapon.nameZh,
  secondary: weapon.name,
  image: getWeaponDefaultImage(weapon.defindex),
  search: searchable(weapon.nameZh, weapon.name, weapon.defindex, weapon.category),
  meta: { category: weapon.category },
}))

export function weaponSkinCatalog(defindex: number): CatalogItem[] {
  return (weaponPaints[defindex] ?? []).map((paint) => ({
    id: `${defindex}_${paint.id}`,
    numericId: paint.id,
    name: paint.name,
    secondary: skinNamesEn[defindex]?.[paint.id],
    image: paint.image ?? getWeaponDefaultImage(defindex),
    search: searchable(paint.name, skinNamesEn[defindex]?.[paint.id], paint.id),
  }))
}

export const knifeCatalog: CatalogItem[] = knives.map((knife, index) => ({
  id: `knife_${knife.defindex}`,
  numericId: knife.defindex,
  name: knife.nameZh,
  secondary: knife.name,
  image: getWeaponDefaultImage(knife.defindex),
  search: searchable(knife.nameZh, knife.name, knife.defindex),
  meta: { index },
}))

export function knifeSkinCatalog(defindex: number): CatalogItem[] {
  return (knifeSkinsByType[defindex] ?? []).map((paint) => ({
    id: `${defindex}_${paint.id}`,
    numericId: paint.id,
    name: paint.name,
    image: paint.image,
    search: searchable(paint.name, paint.id),
  }))
}

export const gloveCatalog: CatalogItem[] = gloves.map((glove, index) => ({
  id: `glove_${glove.defindex}`,
  numericId: glove.defindex,
  name: getGloveLocalizedName(glove.defindex, glove.name, 'schinese'),
  secondary: glove.codename,
  image: glove.paints[0]?.image ?? '',
  search: searchable(glove.name, glove.codename, glove.defindex),
  meta: { index },
}))

export function glovePaintCatalog(defindex: number): CatalogItem[] {
  const glove = gloves.find((entry) => entry.defindex === defindex)
  return (glove?.paints ?? []).map((paint) => ({
    id: `${defindex}_${paint.id}`,
    numericId: paint.id,
    name: getGlovePaintLocalizedName(defindex, paint.id, paint.name, 'schinese'),
    image: paint.image ?? '',
    search: searchable(paint.name, paint.id),
  }))
}

export const agentCatalog: Record<'ct' | 't', CatalogItem[]> = {
  ct: mapAgents('ct'),
  t: mapAgents('t'),
}

export const musicCatalog: CatalogItem[] = musicKits.map((kit) => ({
  id: `music_${kit.id}`,
  numericId: kit.id,
  name: getLocalizedName(`music_kit-${kit.id}`, musicKitNameMap, 'schinese', kit.name),
  image: kit.image,
  search: searchable(kit.name, musicKitNameMap[`music_kit-${kit.id}`]?.en, kit.id),
}))

export const keychainCatalog: CatalogItem[] = allKeychains.map((item) => ({
  id: `keychain_${item.id}`,
  numericId: item.id,
  name: item.nameZh || item.name,
  secondary: item.name,
  image: item.image,
  search: searchable(item.nameZh, item.name, item.id),
}))

export async function loadStickerCatalog(): Promise<CatalogItem[]> {
  const { allStickers, getStickerImageUrl } = await import('./generated/stickers')
  return allStickers.map((item) => ({
    id: `sticker_${item.id}`,
    numericId: item.id,
    name: translateStickerName(item.name),
    secondary: item.name,
    image: getStickerImageUrl(item.image),
    search: searchable(translateStickerName(item.name), item.name, item.id),
  }))
}

export function filterCatalog(items: CatalogItem[], query: string): CatalogItem[] {
  const terms = normalize(query).split(/\s+/).filter(Boolean)
  if (!terms.length) return items
  return items.filter((item) => terms.every((term) => item.search.includes(term)))
}

function mapAgents(team: 'ct' | 't'): CatalogItem[] {
  return agentModels[team].map((agent, index) => ({
    id: agent.id,
    numericId: index,
    name: getLocalizedName(agent.id, agentNameMap, 'schinese', agent.name),
    secondary: agentNameMap[agent.id]?.en,
    image: agent.image,
    search: searchable(agent.name, agentNameMap[agent.id]?.zh, agentNameMap[agent.id]?.en, agent.id),
    meta: { model: agent.model, index },
  }))
}

function translateStickerName(name: string): string {
  return name.replace(/^Sticker \| /, '印花 | ').replace(/^Patch \| /, '布章 | ')
}
function searchable(...values: unknown[]): string { return normalize(values.filter(Boolean).join(' ')) }
function normalize(value: string): string { return value.normalize('NFKC').toLocaleLowerCase('zh-CN').replace(/[|_-]+/g, ' ') }
