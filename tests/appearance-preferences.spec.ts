import { beforeEach, describe, expect, it } from 'vitest'
import { useAppearancePreferences } from '@/composables/useAppearancePreferences'

describe('appearance preferences', () => {
  beforeEach(() => {
    document.documentElement.removeAttribute('data-palette')
    document.documentElement.removeAttribute('data-radius')
    document.documentElement.removeAttribute('data-density')
    document.documentElement.removeAttribute('data-sidebar-mode')
  })

  it('applies palette, radius, density and sidebar mode to the document and persists them', () => {
    const { preferences, update } = useAppearancePreferences()
    update({ palette: 'rose', radius: '0.25', density: 'compact', sidebarMode: 'floating' })
    expect(preferences.palette).toBe('rose')
    expect(document.documentElement.dataset.palette).toBe('rose')
    expect(document.documentElement.dataset.radius).toBe('0.25')
    expect(document.documentElement.dataset.density).toBe('compact')
    expect(document.documentElement.dataset.sidebarMode).toBe('floating')
  })

  it('never hides installation diagnostics and keeps every navigation key in the order', () => {
    const { preferences, update } = useAppearancePreferences()
    update({ hiddenSidebarItems: ['install', 'overview'], sidebarOrder: ['install', 'overview'] })
    expect(preferences.hiddenSidebarItems).toEqual(['overview'])
    expect(preferences.sidebarOrder[0]).toBe('install')
    expect(preferences.sidebarOrder).toHaveLength(9)
  })

  it('persists the startup animation preference', () => {
    const { preferences, update } = useAppearancePreferences()
    update({ skipIntro: true })
    expect(preferences.skipIntro).toBe(true)
    expect(preferences.skipIntro).toBe(true)
  })
})
