import { reactive, readonly } from 'vue'
import { applyTheme, readStoredTheme, type AppTheme } from '@/composables/useThemePreference'
import { NAV_ITEM_KEYS, type ViewKey } from '@/config/navigation'

export type AccentPalette = 'default' | 'rose' | 'tide' | 'sunset' | 'forest' | 'sea' | 'dream'
export type RadiusPreference = 'auto' | '0' | '0.25' | '0.5' | '0.75' | '1.0'
export type DensityPreference = 'compact' | 'default' | 'loose'
export type SidebarMode = 'default' | 'embedded' | 'floating'

export interface AppearancePreferences {
  theme: AppTheme
  palette: AccentPalette
  radius: RadiusPreference
  density: DensityPreference
  sidebarMode: SidebarMode
  skipIntro: boolean
  sidebarOrder: ViewKey[]
  hiddenSidebarItems: ViewKey[]
}

const STORAGE_KEY = 'cs2-bot-improver.appearance.v1'
const defaults: AppearancePreferences = {
  theme: 'light', palette: 'default', radius: 'auto', density: 'default', sidebarMode: 'default', skipIntro: true,
  sidebarOrder: [...NAV_ITEM_KEYS], hiddenSidebarItems: [],
}
const state = reactive<AppearancePreferences>({ ...defaults, sidebarOrder: [...defaults.sidebarOrder], hiddenSidebarItems: [] })
let initialized = false

function validList(value: unknown): ViewKey[] {
  if (!Array.isArray(value)) return []
  return value.filter((key): key is ViewKey => NAV_ITEM_KEYS.includes(key as ViewKey))
}

function normalizeOrder(order: unknown): ViewKey[] {
  const result = validList(order)
  return [...result, ...NAV_ITEM_KEYS.filter(key => !result.includes(key))]
}

function readStored(): Partial<AppearancePreferences> | null {
  if (typeof window === 'undefined') return null
  try { return JSON.parse(window.localStorage.getItem(STORAGE_KEY) || 'null') } catch { return null }
}

function persist() {
  if (typeof window === 'undefined') return
  try { window.localStorage?.setItem(STORAGE_KEY, JSON.stringify(state)) } catch { /* storage can be unavailable in restricted WebViews */ }
}

function applyAppearance() {
  if (typeof document === 'undefined') return
  const root = document.documentElement
  root.dataset.palette = state.palette
  root.dataset.radius = state.radius
  root.dataset.density = state.density
  root.dataset.sidebarMode = state.sidebarMode
  applyTheme(state.theme)
}

export function initializeAppearancePreferences(): AppearancePreferences {
  if (initialized) return state
  initialized = true
  const stored = readStored()
  if (stored) {
    if (stored.theme === 'light' || stored.theme === 'dark') state.theme = stored.theme
    if (['default', 'rose', 'tide', 'sunset', 'forest', 'sea', 'dream'].includes(stored.palette || '')) state.palette = stored.palette as AccentPalette
    if (['auto', '0', '0.25', '0.5', '0.75', '1.0'].includes(stored.radius || '')) state.radius = stored.radius as RadiusPreference
    if (['compact', 'default', 'loose'].includes(stored.density || '')) state.density = stored.density as DensityPreference
    if (['default', 'embedded', 'floating'].includes(stored.sidebarMode || '')) state.sidebarMode = stored.sidebarMode as SidebarMode
    if (typeof stored.skipIntro === 'boolean') state.skipIntro = stored.skipIntro
    state.sidebarOrder = normalizeOrder(stored.sidebarOrder)
    state.hiddenSidebarItems = validList(stored.hiddenSidebarItems).filter(key => key !== 'install')
  } else {
    state.theme = readStoredTheme() ?? state.theme
  }
  applyAppearance()
  if (typeof window !== 'undefined') {
    window.addEventListener('cs2as:theme-changed', event => {
      const nextTheme = (event as CustomEvent<AppTheme>).detail
      if (nextTheme === 'light' || nextTheme === 'dark') {
        state.theme = nextTheme
        persist()
      }
    })
  }
  return state
}

export function useAppearancePreferences() {
  initializeAppearancePreferences()
  function update(patch: Partial<AppearancePreferences>) {
    if (patch.theme === 'light' || patch.theme === 'dark') state.theme = patch.theme
    if (patch.palette) state.palette = patch.palette
    if (patch.radius) state.radius = patch.radius
    if (patch.density) state.density = patch.density
    if (patch.sidebarMode) state.sidebarMode = patch.sidebarMode
    if (typeof patch.skipIntro === 'boolean') state.skipIntro = patch.skipIntro
    if (patch.sidebarOrder) state.sidebarOrder = normalizeOrder(patch.sidebarOrder)
    if (patch.hiddenSidebarItems) state.hiddenSidebarItems = validList(patch.hiddenSidebarItems).filter(key => key !== 'install')
    applyAppearance(); persist()
  }
  function reset() {
    Object.assign(state, { ...defaults, sidebarOrder: [...defaults.sidebarOrder], hiddenSidebarItems: [] })
    applyAppearance(); persist()
  }
  return { preferences: readonly(state), update, reset }
}
