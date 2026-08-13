export type Team = 'ct' | 't'
export type ForgeMode = 'custom' | 'random'

export interface StickerSlot {
  id: number
  schema?: number
  offsetX: number
  offsetY: number
  wear: number
  scale: number
  rotation: number
}

export interface KeychainSlot {
  id: number
  offsetX: number
  offsetY: number
  offsetZ: number
  seed: number
}

export interface WeaponLoadout {
  defindex: number
  paintKit: number
  wear: number
  seed: number
  nameTag: string
  statTrak: number | null
  stickers: StickerSlot[]
  keychain: KeychainSlot | null
}

export interface TeamLoadout {
  knife: { index: number; defindex: number; paintKit: number; wear: number; seed: number }
  gloves: { defindex: number; index: number; paintKit: number; wear: number; seed: number }
  agent: number | null
  agentPath: string
}

export interface Loadout {
  schema: 3
  mode: ForgeMode
  activeTeam: Team
  ct: TeamLoadout
  t: TeamLoadout
  weapons: Record<Team, Record<string, WeaponLoadout>>
  musicKit: number | null
  passthrough?: Record<string, unknown>
}

const emptyTeam = (): TeamLoadout => ({
  knife: { index: -1, defindex: 42, paintKit: -1, wear: 0.01, seed: 0 },
  gloves: { defindex: 0, index: -1, paintKit: -1, wear: 0.01, seed: 0 },
  agent: null,
  agentPath: '',
})

export const DEFAULT_LOADOUT: Loadout = {
  schema: 3,
  mode: 'random',
  activeTeam: 'ct',
  musicKit: null,
  ct: emptyTeam(),
  t: emptyTeam(),
  weapons: {
    ct: {},
    t: {},
  },
}

export function createWeapon(defindex: number): WeaponLoadout {
  return {
    defindex,
    paintKit: -1,
    wear: 0.01,
    seed: 0,
    nameTag: '',
    statTrak: null,
    stickers: [],
    keychain: null,
  }
}

export function migrateLoadout(input: unknown): Loadout {
  const source = asObject(input)
  const result = structuredClone(DEFAULT_LOADOUT)
  result.mode = source.mode === 'custom' ? 'custom' : source.mode === 'random' ? 'random' : result.mode
  result.activeTeam = source.activeTeam === 't' ? 't' : 'ct'
  result.musicKit = nullableInteger(source.musicKit)
  result.ct = migrateTeam(source.ct, source)
  result.t = migrateTeam(source.t, source)

  const weaponSource = asObject(source.weapons)
  const hasTeams = Object.hasOwn(weaponSource, 'ct') || Object.hasOwn(weaponSource, 't')
  result.weapons.ct = migrateWeapons(hasTeams ? weaponSource.ct : weaponSource)
  result.weapons.t = migrateWeapons(hasTeams ? weaponSource.t : weaponSource)
  if (!hasTeams) result.weapons.t = structuredClone(result.weapons.ct)
  synchronizeSharedDetails(result)
  result.passthrough = asObject(source.passthrough)
  return result
}

export function synchronizeSharedDetails(loadout: Loadout, preferred: Team = 'ct'): void {
  const other: Team = preferred === 'ct' ? 't' : 'ct'
  const byDefindex = new Map<number, Pick<WeaponLoadout, 'stickers' | 'keychain' | 'nameTag' | 'statTrak'>>()
  for (const team of [preferred, other]) {
    for (const weapon of Object.values(loadout.weapons[team])) {
      if (!byDefindex.has(weapon.defindex)) {
        byDefindex.set(weapon.defindex, {
          stickers: normalizeStickers(weapon.stickers),
          keychain: normalizeKeychain(weapon.keychain),
          nameTag: typeof weapon.nameTag === 'string' ? weapon.nameTag.slice(0, 64) : '',
          statTrak: weapon.statTrak === null ? null : nonNegative(weapon.statTrak),
        })
      }
    }
  }
  for (const team of ['ct', 't'] as const) {
    for (const weapon of Object.values(loadout.weapons[team])) {
      const details = byDefindex.get(weapon.defindex)
      if (details) Object.assign(weapon, structuredClone(details))
    }
  }
}

function migrateTeam(value: unknown, legacy: Record<string, unknown>): TeamLoadout {
  const source = asObject(value)
  const result = emptyTeam()
  const knife = asObject(source.knife ?? asObject(legacy.weapon).knife)
  const gloves = asObject(source.gloves ?? legacy.gloves)
  result.knife = {
    index: integer(knife.index, -1),
    defindex: integer(knife.defindex, result.knife.defindex),
    paintKit: integer(knife.paintKit, -1),
    wear: clamp(knife.wear ?? result.knife.wear),
    seed: nonNegative(knife.seed),
  }
  result.gloves = {
    defindex: nonNegative(gloves.defindex),
    index: integer(gloves.index, -1),
    paintKit: integer(gloves.paintKit, -1),
    wear: clamp(gloves.wear ?? result.gloves.wear),
    seed: nonNegative(gloves.seed),
  }
  result.agent = nullableInteger(source.agent)
  result.agentPath = typeof source.agentPath === 'string' ? source.agentPath : ''
  return result
}

function migrateWeapons(value: unknown): Record<string, WeaponLoadout> {
  const result: Record<string, WeaponLoadout> = {}
  for (const raw of Object.values(asObject(value))) {
    const source = asObject(raw)
    const defindex = integer(source.defindex)
    if (defindex < 1 || defindex > 65_535) continue
    result[`weapon_${defindex}`] = {
      defindex,
      paintKit: integer(source.paintKit, -1),
      wear: clamp(source.wear ?? 0.01),
      seed: nonNegative(source.seed),
      nameTag: typeof source.nameTag === 'string' ? source.nameTag.slice(0, 64) : '',
      statTrak: nullableInteger(source.statTrak),
      stickers: normalizeStickers(source.stickers),
      keychain: normalizeKeychain(source.keychain),
    }
  }
  return result
}

function normalizeStickers(value: unknown): StickerSlot[] {
  if (!Array.isArray(value)) return []
  return value.slice(0, 5).map((raw) => {
    const item = asObject(raw)
    return {
      id: nonNegative(item.id),
      schema: nonNegative(item.schema),
      offsetX: finite(item.offsetX),
      offsetY: finite(item.offsetY),
      wear: clamp(item.wear),
      scale: Math.min(5, Math.max(0.1, finite(item.scale, 1))),
      rotation: Math.min(180, Math.max(-180, finite(item.rotation))),
    }
  }).filter((item) => item.id > 0)
}

function normalizeKeychain(value: unknown): KeychainSlot | null {
  const source = asObject(value)
  const id = nonNegative(source.id)
  return id > 0 ? {
    id,
    offsetX: Math.min(5, Math.max(-5, finite(source.offsetX))),
    offsetY: Math.min(5, Math.max(-5, finite(source.offsetY))),
    offsetZ: Math.min(5, Math.max(-5, finite(source.offsetZ))),
    seed: Math.min(9999, nonNegative(source.seed)),
  } : null
}

function asObject(value: unknown): Record<string, unknown> {
  return value && typeof value === 'object' && !Array.isArray(value) ? value as Record<string, unknown> : {}
}
function finite(value: unknown, fallback = 0): number { const parsed = Number(value); return Number.isFinite(parsed) ? parsed : fallback }
function integer(value: unknown, fallback = 0): number { return Math.trunc(finite(value, fallback)) }
function nonNegative(value: unknown): number { return Math.max(0, integer(value)) }
function nullableInteger(value: unknown): number | null { return value === null || value === undefined || value === '' ? null : integer(value) }
function clamp(value: unknown): number { return Math.min(1, Math.max(0, finite(value))) }
