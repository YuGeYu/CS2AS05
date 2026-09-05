import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(resolve(process.cwd(), path), 'utf8')

describe('0.5.5 integrated Panel contract', () => {
  it('uses the application shell and retains the upstream Panel compatibility action', () => {
    expect(read('src/App.vue')).toContain("import AppShell from '@/components/AppShell.vue'")
    expect(read('src/App.vue')).toContain("import AppTitlebar from '@/components/AppTitlebar.vue'")
    expect(read('src/views/InstallView.vue')).toContain('打开原版 Panel')
    expect(read('src/services/tauri/cs2.ts')).toContain("invoke<OperationResult>('open_upstream_panel')")
    expect(read('src-tauri/src/lib.rs')).toContain('commands::cs2::open_upstream_panel')
  })

  it('verifies the unchanged minimal customization package before installation', () => {
    const service = read('src-tauri/src/services/cs2.rs')
    expect(service).toContain('const CUSTOM_ZIP_SHA256')
    expect(service).toContain('verify_custom_zip(&zip_path)?')
    expect(service).toContain('最小定制')
    expect(read('src/views/InstallView.vue')).toContain('appConfig.appVersion')
    expect(read('src/views/InstallView.vue')).toContain('CS2-Bot-Improver v1.4.4')
    expect(read('src-tauri/src/services/cs2.rs')).not.toContain('bot_randomizer_options.json')
  })

  it('keeps support and sources in the installation diagnostics view', () => {
    const installView = read('src/views/InstallView.vue')
    const support = read('src/components/SupportActions.vue')
    const rust = read('src-tauri/src/lib.rs')
    expect(installView).toContain('<SupportActions />')
    expect(installView.indexOf('<SupportActions />')).toBeLessThan(installView.indexOf('class="directory-section"'))
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

  it('provides native maintenance and fault-report controls with narrow cleanup scope', () => {
    const view = read('src/views/InstallView.vue')
    const support = read('src-tauri/src/services/support.rs')
    const commands = read('src-tauri/src/lib.rs')
    for (const label of ['开机启动', '清除数据', '提交故障', '诊断日志将随工单提交']) expect(view).toContain(label)
    for (const command of ['get_assistant_preferences', 'set_assistant_autostart', 'clear_assistant_data', 'submit_fault_report']) {
      expect(commands).toContain(`commands::support::${command}`)
    }
    expect(support).toContain('AI_CHAT_SESSIONS_FILE')
    expect(support).toContain('CS2 文件、插件、Demo 与复盘记录均已保留')
    expect(support).not.toContain('remove_dir_all(root_path)')
  })

  it('removes manual diagnostics viewing while retaining automatic fault-log attachment', () => {
    const view = read('src/views/InstallView.vue')
    const store = read('src/stores/cs2.ts')
    expect(view).not.toContain('查看诊断日志')
    expect(view).not.toContain('toggleDiagnostics')
    expect(view).not.toContain('diagnosticsOpen')
    expect(view).toContain('submitFaultReport')
    expect(store).not.toContain('refreshDiagnostics')
    expect(store).not.toContain('getDiagnosticsPayload')
    expect(read('src-tauri/src/services/support.rs')).toContain('get_diagnostics_payload(root_path)')
  })
})
