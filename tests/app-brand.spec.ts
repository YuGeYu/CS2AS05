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

  it('uses the release version as the accessible easter egg button', () => {
    const shell = read('src/components/AppShell.vue')
    expect(shell).toContain('class="version-easter-egg"')
    expect(shell).toContain('{{ appConfig.appVersion }}</button>')
    expect(shell).not.toContain('<strong>CS2 助手</strong>')
    expect(shell).not.toContain('Panel {{ appConfig.appVersion }}')
  })
})
