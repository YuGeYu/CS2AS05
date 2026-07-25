import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(resolve(process.cwd(), path), 'utf8')

describe('application brand', () => {
  it('uses the official app icon in the titlebar', () => {
    const titlebar = read('src/components/AppTitlebar.vue')
    expect(titlebar).toContain("import appIcon from '@/assets/app-icon.png'")
    expect(titlebar).toContain('<img class="titlebar-mark" :src="appIcon" alt="" />')
    expect(titlebar).not.toContain('<span class="titlebar-mark">CS2</span>')
  })

  it('shows only the release version below the sidebar brand', () => {
    const shell = read('src/components/AppShell.vue')
    expect(shell).toContain('<strong>CS2 助手</strong><small>0.5.3</small>')
    expect(shell).not.toContain('Panel 0.5.3')
  })
})
