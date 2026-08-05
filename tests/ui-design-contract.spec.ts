import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(resolve(process.cwd(), path), 'utf8')
const styles = read('src/styles/main.css')

describe('desktop design contract', () => {
  it('defines the blue data-dense semantic system for both themes', () => {
    for (const token of [
      '--color-primary: #1e40af', '--color-secondary: #3b82f6', '--color-accent: #d97706',
      '--color-background: #f8fafc', '--color-text: #0f172a', '--color-border: #dbeafe',
      '--app-bg:', '--surface:', '--surface-muted:', '--surface-raised:', '--text:', '--text-muted:',
      '--border:', '--primary:', '--primary-hover:', '--bronze:', '--cinnabar:', '--ink:',
      '--engraving-line:', '--panel-shadow:', '--focus-ring:', '--motion-standard:',
    ]) expect(styles).toContain(token)

    expect(styles).toContain(":root[data-theme='dark']")
    expect(styles).toContain('--app-bg: #0b1120')
    expect(styles).toContain('--primary: #60a5fa')
    expect(styles).not.toContain('font-family: Inter')
    expect(styles).toContain('"Microsoft YaHei UI", "Segoe UI", sans-serif')
    expect(styles).toContain('Consolas, "Cascadia Mono", monospace')
  })

  it('keeps the application shell semantic and exposes navigation state', () => {
    const shell = read('src/components/AppShell.vue')
    expect(shell).toContain('<aside class="sidebar">')
    expect(shell).toContain('<nav aria-label="主导航"')
    expect(shell).toContain(':aria-current="current === item.key ? \'page\' : undefined"')
    expect(shell).toContain('<main class="view-container"')
    expect(shell).not.toContain('玄铁机括')
    expect(shell).not.toContain('<strong>人机增强</strong>')
    expect(styles).not.toContain("content: '青玉灵脉'")
  })

  it('keeps all principal workspaces on native semantic controls', () => {
    const paths = [
      'src/views/OverviewView.vue', 'src/views/PresetsView.vue', 'src/views/BotItemsView.vue',
      'src/views/KnivesView.vue', 'src/views/CommandsView.vue', 'src/views/DemoReviewView.vue',
      'src/views/InstallView.vue',
    ]
    for (const path of paths) {
      const view = read(path)
      expect(view).toContain('<section')
      expect(view).toMatch(/<button|<ToggleSwitch/)
      expect(view).not.toMatch(/<input[^>]+type=["']file["']/i)
      expect(view).not.toMatch(/showOpenFilePicker|webkitdirectory/i)
    }
    const demo = read('src/views/DemoReviewView.vue')
    expect(demo).toContain("import('@tauri-apps/plugin-dialog')")
    expect(demo).toContain('role="tablist"')
    expect(demo).toContain(':aria-selected="tab === \'library\'"')
    expect(demo).toContain(':data-team="group.key"')
  })

  it('gives icon buttons accessible names and preserves async state semantics', () => {
    const sources = [
      read('src/components/AppTitlebar.vue'), read('src/components/Cs2RootSuggestionsDialog.vue'),
      read('src/components/SoftwareUpdateModal.vue'), read('src/views/OverviewView.vue'),
      read('src/views/CommandsView.vue'), read('src/views/DemoReviewView.vue'),
    ].join('\n')
    const iconButtons = sources.match(/<button\b[^>]*class="[^"]*icon-button[^"]*"[^>]*>/g) ?? []
    expect(iconButtons.length).toBeGreaterThan(5)
    for (const button of iconButtons) expect(button).toMatch(/(?:aria-label|title)=/)
    expect(read('src/components/ui/SegmentedControl.vue')).toContain(':aria-busy="pending || undefined"')
    expect(read('src/components/ui/ToggleSwitch.vue')).toContain(':disabled="disabled"')
  })

  it('contains responsive, internal-table-scroll, sticky-header, and reduced-motion guards', () => {
    expect(styles).toContain('@media (max-width: 860px)')
    expect(styles).toContain('@media (max-width: 700px)')
    expect(styles).toMatch(/\.demo-table-wrap\s*\{[^}]*overflow-x:\s*auto/s)
    expect(styles).toMatch(/\.demo-table-wrap\s*\{[^}]*max-height:/s)
    expect(styles).toMatch(/\.demo-table thead th\s*\{[^}]*position:\s*sticky/s)
    expect(styles).toContain('@media (prefers-reduced-motion: reduce)')
    expect(styles).toContain('animation-iteration-count: 1 !important')
    expect(read('vite.config.ts')).toContain("ignored: ['**/target/**']")
  })
})
