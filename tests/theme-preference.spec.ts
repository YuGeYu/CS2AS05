import { beforeEach, describe, expect, it, vi } from 'vitest'

import {
  THEME_STORAGE_KEY,
  getSystemTheme,
  initializeTheme,
  readStoredTheme,
  useThemePreference,
} from '@/composables/useThemePreference'

function setSystemTheme(isDark: boolean) {
  vi.stubGlobal('matchMedia', vi.fn().mockImplementation(() => ({ matches: isDark })))
}

function installStorage() {
  const values = new Map<string, string>()
  const storage = {
    clear: () => values.clear(),
    getItem: (key: string) => values.get(key) ?? null,
    key: (index: number) => [...values.keys()][index] ?? null,
    removeItem: (key: string) => values.delete(key),
    setItem: (key: string, value: string) => values.set(key, value),
    get length() { return values.size },
  }
  Object.defineProperty(window, 'localStorage', { configurable: true, value: storage })
}

describe('theme preference', () => {
  beforeEach(() => {
    installStorage()
    window.localStorage.clear()
    document.documentElement.removeAttribute('data-theme')
    document.documentElement.style.colorScheme = ''
    setSystemTheme(false)
  })

  it('uses the system preference when no saved theme exists', () => {
    setSystemTheme(true)

    expect(initializeTheme()).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(document.documentElement.style.colorScheme).toBe('dark')
  })

  it('uses a valid saved value and rejects invalid storage', () => {
    window.localStorage.setItem(THEME_STORAGE_KEY, 'light')
    expect(readStoredTheme()).toBe('light')
    expect(initializeTheme()).toBe('light')

    window.localStorage.setItem(THEME_STORAGE_KEY, 'system')
    expect(readStoredTheme()).toBeNull()
  })

  it('switches, applies, and persists the selected theme', () => {
    initializeTheme()
    const { theme, toggleTheme } = useThemePreference()

    toggleTheme()

    expect(theme.value).toBe('dark')
    expect(document.documentElement.dataset.theme).toBe('dark')
    expect(window.localStorage.getItem(THEME_STORAGE_KEY)).toBe('dark')
  })

  it('reports a light system preference when media queries are unavailable', () => {
    vi.stubGlobal('matchMedia', undefined)
    expect(getSystemTheme()).toBe('light')
  })
})
