import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(resolve(process.cwd(), path), 'utf8')

describe('0.5.2 installer contract', () => {
  it('uses the single install view and exposes the upstream Panel action', () => {
    expect(read('src/App.vue')).toContain("import InstallView from '@/views/InstallView.vue'")
    expect(read('src/App.vue')).toContain("import AppTitlebar from '@/components/AppTitlebar.vue'")
    expect(read('src/views/InstallView.vue')).toContain('打开 Panel')
    expect(read('src/services/tauri/cs2.ts')).toContain("invoke<OperationResult>('open_upstream_panel')")
    expect(read('src-tauri/src/lib.rs')).toContain('commands::cs2::open_upstream_panel')
  })

  it('verifies the 0.5.2 minimal customization package before installation', () => {
    const service = read('src-tauri/src/services/cs2.rs')
    expect(service).toContain('const CUSTOM_ZIP_SHA256')
    expect(service).toContain('verify_custom_zip(&zip_path)?')
    expect(service).toContain('最小定制')
    expect(read('src/views/InstallView.vue')).toContain('0.5.2')
    expect(read('src/views/InstallView.vue')).toContain('0.5.2 定制资源包')
    expect(read('src/views/InstallView.vue')).toContain('CS2-Bot-Improver v1.4.2')
  })

  it('keeps support and sources in the single-screen installer', () => {
    const installView = read('src/views/InstallView.vue')
    const support = read('src/components/SupportActions.vue')
    const rust = read('src-tauri/src/lib.rs')
    expect(installView).toContain('<SupportActions />')
    for (const label of ['检查更新', '打开官网', '查看/编辑意见', '关于与来源']) expect(support).toContain(label)
    for (const command of ['open_official_site', 'open_idea_page', 'open_release_page', 'open_upstream_project', 'open_update_download']) {
      expect(rust).toContain(`commands::support::${command}`)
    }
    expect(read('src/services/tauri/support.ts')).not.toContain('openExternalUrl')
    expect(read('src/App.vue')).not.toContain('RouterView')
  })

  it('does not reintroduce removed Plus or AI control surfaces', () => {
    const activeSources = [
      read('src/App.vue'),
      read('src/views/InstallView.vue'),
      read('src/stores/cs2.ts'),
      read('src-tauri/src/lib.rs'),
      read('src-tauri/src/commands/cs2.rs'),
    ].join('\n')

    for (const removed of ['numakkiyu', 'CS2-Bot-Improver-Plus', 'BotTaunt', 'AiApi', 'PlusRuntimeStatus']) {
      expect(activeSources).not.toContain(removed)
    }
  })

  it('keeps theme and native window controls in the single-app shell', () => {
    expect(read('src/app/create-app.ts')).toContain('initializeTheme()')
    expect(read('src/components/AppTitlebar.vue')).toContain('getCurrentWindow')
    expect(read('src/components/AppTitlebar.vue')).toContain('startDragging')
  })
})
