import { readonly, ref } from 'vue'

export type AppTheme = 'light' | 'dark'

export const THEME_STORAGE_KEY = 'cs2-bot-improver.theme.v1'

const theme = ref<AppTheme>('light')

export function initializeTheme(): AppTheme {
  const nextTheme = readStoredTheme() ?? getSystemTheme()
  applyTheme(nextTheme)
  return nextTheme
}

export function useThemePreference() {
  function setTheme(nextTheme: AppTheme) {
    applyTheme(nextTheme)
    window.localStorage.setItem(THEME_STORAGE_KEY, nextTheme)
  }

  function toggleTheme() {
    setTheme(theme.value === 'light' ? 'dark' : 'light')
  }

  return { theme: readonly(theme), setTheme, toggleTheme }
}

export function readStoredTheme(): AppTheme | null {
  if (typeof window === 'undefined') return null
  const stored = window.localStorage.getItem(THEME_STORAGE_KEY)
  return stored === 'light' || stored === 'dark' ? stored : null
}

export function getSystemTheme(): AppTheme {
  if (typeof window === 'undefined') return 'light'
  return window.matchMedia?.('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function applyTheme(nextTheme: AppTheme) {
  theme.value = nextTheme
  if (typeof document === 'undefined') return
  document.documentElement.dataset.theme = nextTheme
  document.documentElement.style.colorScheme = nextTheme
}
