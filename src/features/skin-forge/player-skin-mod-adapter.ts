import {
  DEFAULT_LOADOUT,
  createWeapon,
  migrateLoadout,
  synchronizeSharedDetails,
  type Loadout,
  type WeaponLoadout,
} from '@/types/skin-forge'
import { knives } from '@/features/skin-forge/data/generated/knives'

export interface UpstreamPlayerLoadout extends Record<string, unknown> {
  weaponPaints: Record<string, number>
  weaponWears: Record<string, number>
  weaponSeeds: Record<string, number>
  weaponPaintsCt: Record<string, number>
  weaponWearsCt: Record<string, number>
  weaponSeedsCt: Record<string, number>
  weaponPaintsT: Record<string, number>
  weaponWearsT: Record<string, number>
  weaponSeedsT: Record<string, number>
  weaponStickers: Record<string, unknown[]>
  weaponKeychains: Record<string, unknown>
  weaponNametags: Record<string, string>
  weaponStatTrak: Record<string, { enabled: boolean; count: number }>
  knifeIndex: number
  knifePaint: number
  knifeWear: number
  knifeSeed: number
  knifeIndexCt: number
  knifePaintCt: number
  knifeWearCt: number
  knifeSeedCt: number
  knifeIndexT: number
  knifePaintT: number
  knifeWearT: number
  knifeSeedT: number
  gloveIndexCt: number
  glovePaintCt: number
  gloveWearCt: number
  gloveSeedCt: number
  gloveDefIndexCt: number
  gloveIndexT: number
  glovePaintT: number
  gloveWearT: number
  gloveSeedT: number
  gloveDefIndexT: number
  agentModelCt: number
  agentModelT: number
  agentModelPathCt: string
  agentModelPathT: string
  musicKit: number
  useRandom: boolean
}
export type PlayerSkinModFile = Record<string, UpstreamPlayerLoadout>

const KNOWN_FIELDS = new Set([
  'weaponPaints', 'weaponWears', 'weaponSeeds', 'weaponPaintsCt', 'weaponWearsCt', 'weaponSeedsCt',
  'weaponPaintsT', 'weaponWearsT', 'weaponSeedsT', 'weaponStickers', 'weaponKeychains',
  'weaponNametags', 'weaponStatTrak', 'knifeIndex', 'knifePaint', 'knifeWear', 'knifeSeed',
  'knifeIndexCt', 'knifePaintCt', 'knifeWearCt', 'knifeSeedCt', 'knifeIndexT', 'knifePaintT',
  'knifeWearT', 'knifeSeedT', 'gloveIndexCt', 'glovePaintCt', 'gloveWearCt', 'gloveSeedCt',
  'gloveDefIndexCt', 'gloveIndexT', 'glovePaintT', 'gloveWearT', 'gloveSeedT', 'gloveDefIndexT',
  'agentModelCt', 'agentModelT', 'agentModelPathCt', 'agentModelPathT', 'musicKit', 'useRandom',
])

export function toPlayerSkinModFile(loadout: Loadout, slot = 0): PlayerSkinModFile {
  synchronizeSharedDetails(loadout)
  const ct = maps(loadout.weapons.ct)
  const t = maps(loadout.weapons.t)
  const shared = details(loadout)
  const item: UpstreamPlayerLoadout = {
    ...loadout.passthrough,
    weaponPaints: ct.paints,
    weaponWears: ct.wears,
    weaponSeeds: ct.seeds,
    weaponPaintsCt: ct.paints,
    weaponWearsCt: ct.wears,
    weaponSeedsCt: ct.seeds,
    weaponPaintsT: t.paints,
    weaponWearsT: t.wears,
    weaponSeedsT: t.seeds,
    ...shared,
    knifeIndex: knifeIndex(loadout.ct.knife),
    knifePaint: loadout.ct.knife.paintKit,
    knifeWear: clamp(loadout.ct.knife.wear),
    knifeSeed: nonNegative(loadout.ct.knife.seed),
    knifeIndexCt: knifeIndex(loadout.ct.knife),
    knifePaintCt: loadout.ct.knife.paintKit,
    knifeWearCt: clamp(loadout.ct.knife.wear),
    knifeSeedCt: nonNegative(loadout.ct.knife.seed),
    knifeIndexT: knifeIndex(loadout.t.knife),
    knifePaintT: loadout.t.knife.paintKit,
    knifeWearT: clamp(loadout.t.knife.wear),
    knifeSeedT: nonNegative(loadout.t.knife.seed),
    gloveIndexCt: loadout.ct.gloves.index,
    glovePaintCt: loadout.ct.gloves.paintKit,
    gloveWearCt: clamp(loadout.ct.gloves.wear),
    gloveSeedCt: nonNegative(loadout.ct.gloves.seed),
    gloveDefIndexCt: loadout.ct.gloves.defindex,
    gloveIndexT: loadout.t.gloves.index,
    glovePaintT: loadout.t.gloves.paintKit,
    gloveWearT: clamp(loadout.t.gloves.wear),
    gloveSeedT: nonNegative(loadout.t.gloves.seed),
    gloveDefIndexT: loadout.t.gloves.defindex,
    agentModelCt: loadout.ct.agent ?? -1,
    agentModelT: loadout.t.agent ?? -1,
    agentModelPathCt: loadout.ct.agentPath,
    agentModelPathT: loadout.t.agentPath,
    musicKit: loadout.musicKit ?? -1,
    useRandom: loadout.mode === 'random',
  }
  return { [String(slot)]: item }
}

export function fromPlayerSkinModFile(value: unknown, slot = 0): Loadout {
  const root = object(value)
  const item = object(root?.[String(slot)])
  if (!item) return structuredClone(DEFAULT_LOADOUT)
  const result = structuredClone(DEFAULT_LOADOUT)
  result.mode = item.useRandom === true ? 'random' : 'custom'
  result.ct.knife = knife(item, 'Ct')
  result.t.knife = knife(item, 'T')
  result.ct.gloves = glove(item, 'Ct')
  result.t.gloves = glove(item, 'T')
  result.ct.agent = nullablePositive(item.agentModelCt)
  result.t.agent = nullablePositive(item.agentModelT)
  result.ct.agentPath = string(item.agentModelPathCt)
  result.t.agentPath = string(item.agentModelPathT)
  result.musicKit = nullablePositive(item.musicKit)
  result.weapons.ct = readWeapons(item, 'Ct')
  result.weapons.t = readWeapons(item, 'T')
  applyDetails(result.weapons.ct, item)
  applyDetails(result.weapons.t, item)
  result.passthrough = Object.fromEntries(Object.entries(item).filter(([key]) => !KNOWN_FIELDS.has(key)))
  return migrateLoadout(result)
}

function maps(weapons: Record<string, WeaponLoadout>) {
  const paints: Record<string, number> = {}
  const wears: Record<string, number> = {}
  const seeds: Record<string, number> = {}
  for (const weapon of Object.values(weapons)) {
    // Upstream represents random/unselected weapon skins by omitting the
    // defindex from the paint map. Writing an explicit -1 makes the plugin
    // treat it as a selected paint and bypass its random fallback.
    if (integer(weapon.paintKit, -1) < 0) continue
    const key = String(validDefindex(weapon.defindex))
    paints[key] = integer(weapon.paintKit, -1)
    wears[key] = clamp(weapon.wear)
    seeds[key] = nonNegative(weapon.seed)
  }
  return { paints, wears, seeds }
}

function details(loadout: Loadout) {
  const weaponStickers: Record<string, unknown[]> = {}
  const weaponKeychains: Record<string, unknown> = {}
  const weaponNametags: Record<string, string> = {}
  const weaponStatTrak: Record<string, { enabled: boolean; count: number }> = {}
  const seen = new Set<number>()
  for (const team of ['ct', 't'] as const) {
    for (const weapon of Object.values(loadout.weapons[team])) {
      if (seen.has(weapon.defindex)) continue
      seen.add(weapon.defindex)
      const key = String(validDefindex(weapon.defindex))
      if (weapon.stickers.length) weaponStickers[key] = weapon.stickers.slice(0, 5).map((sticker) => ({
        id: nonNegative(sticker.id), schema: nonNegative(sticker.schema), offsetX: finite(sticker.offsetX),
        offsetY: finite(sticker.offsetY), wear: clamp(sticker.wear), scale: Math.min(5, Math.max(0.1, finite(sticker.scale, 1))),
        rotation: Math.min(180, Math.max(-180, finite(sticker.rotation))),
      }))
      if (weapon.keychain?.id) weaponKeychains[key] = { ...weapon.keychain, id: nonNegative(weapon.keychain.id), seed: nonNegative(weapon.keychain.seed) }
      if (weapon.nameTag) weaponNametags[key] = weapon.nameTag.slice(0, 64)
      if (weapon.statTrak !== null) weaponStatTrak[key] = { enabled: true, count: nonNegative(weapon.statTrak) }
    }
  }
  return { weaponStickers, weaponKeychains, weaponNametags, weaponStatTrak }
}

function readWeapons(item: Record<string, unknown>, suffix: 'Ct' | 'T') {
  const paints = object(item[`weaponPaints${suffix}`]) ?? object(item.weaponPaints) ?? {}
  const wears = object(item[`weaponWears${suffix}`]) ?? object(item.weaponWears) ?? {}
  const seeds = object(item[`weaponSeeds${suffix}`]) ?? object(item.weaponSeeds) ?? {}
  const result: Record<string, WeaponLoadout> = {}
  for (const [key, paint] of Object.entries(paints)) {
    const defindex = validDefindex(key)
    const weapon = createWeapon(defindex)
    weapon.paintKit = integer(paint, -1)
    weapon.wear = clamp(wears[key] ?? 0.01)
    weapon.seed = nonNegative(seeds[key])
    result[`weapon_${defindex}`] = weapon
  }
  return result
}

function applyDetails(weapons: Record<string, WeaponLoadout>, item: Record<string, unknown>) {
  const stickers = object(item.weaponStickers) ?? {}
  const keychains = object(item.weaponKeychains) ?? {}
  const names = object(item.weaponNametags) ?? {}
  const stats = object(item.weaponStatTrak) ?? {}
  const keys = new Set([...Object.keys(stickers), ...Object.keys(keychains), ...Object.keys(names), ...Object.keys(stats)])
  for (const key of keys) {
    const defindex = validDefindex(key)
    const weapon = Object.values(weapons).find((entry) => entry.defindex === defindex) ?? createWeapon(defindex)
    weapons[`weapon_${defindex}`] = weapon
    weapon.stickers = (Array.isArray(stickers[key]) ? stickers[key] : []).slice(0, 5).flatMap((raw) => {
      const value = object(raw)
      return value && nonNegative(value.id) > 0 ? [{ id: nonNegative(value.id), schema: nonNegative(value.schema), offsetX: finite(value.offsetX), offsetY: finite(value.offsetY), wear: clamp(value.wear), scale: finite(value.scale, 1), rotation: finite(value.rotation) }] : []
    })
    const chain = object(keychains[key])
    weapon.keychain = chain && nonNegative(chain.id) > 0 ? { id: nonNegative(chain.id), offsetX: finite(chain.offsetX), offsetY: finite(chain.offsetY), offsetZ: finite(chain.offsetZ), seed: nonNegative(chain.seed) } : null
    weapon.nameTag = string(names[key])
    const stat = object(stats[key])
    weapon.statTrak = stat?.enabled === true ? nonNegative(stat.count) : null
  }
}

function knife(item: Record<string, unknown>, suffix: 'Ct' | 'T') {
  const rawIndex = integer(item[`knifeIndex${suffix}`] ?? item.knifeIndex, -1)
  const isArrayIndex = rawIndex >= 0 && rawIndex < knives.length
  return { index: isArrayIndex ? rawIndex : -1, defindex: isArrayIndex ? knives[rawIndex]!.defindex : rawIndex >= 0 ? rawIndex : 42, paintKit: integer(item[`knifePaint${suffix}`] ?? item.knifePaint, -1), wear: clamp(item[`knifeWear${suffix}`] ?? item.knifeWear ?? 0.01), seed: nonNegative(item[`knifeSeed${suffix}`] ?? item.knifeSeed) }
}
function glove(item: Record<string, unknown>, suffix: 'Ct' | 'T') {
  return { defindex: nonNegative(item[`gloveDefIndex${suffix}`] ?? item.gloveDefIndex), index: integer(item[`gloveIndex${suffix}`] ?? item.gloveIndex, -1), paintKit: integer(item[`glovePaint${suffix}`] ?? item.glovePaint, -1), wear: clamp(item[`gloveWear${suffix}`] ?? item.gloveWear ?? 0.01), seed: nonNegative(item[`gloveSeed${suffix}`] ?? item.gloveSeed) }
}
function object(value: unknown): Record<string, unknown> | null { return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : null }
function string(value: unknown): string { return typeof value === 'string' ? value : '' }
function finite(value: unknown, fallback = 0): number { const parsed = Number(value); return Number.isFinite(parsed) ? parsed : fallback }
function integer(value: unknown, fallback = 0): number { return Math.trunc(finite(value, fallback)) }
function nonNegative(value: unknown): number { return Math.max(0, integer(value)) }
function nullablePositive(value: unknown): number | null { const parsed = integer(value, -1); return parsed < 0 ? null : parsed }
function clamp(value: unknown): number { return Math.min(1, Math.max(0, finite(value))) }
function validDefindex(value: unknown): number { const parsed = integer(value); if (parsed < 1 || parsed > 65_535) throw new Error(`非法武器 defindex：${String(value)}`); return parsed }
function knifeIndex(knife: Loadout['ct']['knife']): number {
  if (knife.index >= 0) return knife.index
  return knife.defindex > 42 ? knife.defindex : -1
}
