import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(resolve(process.cwd(), path), 'utf8')

describe('BOT plugin launch gate', () => {
  it('keeps the gate in the atomic backend launch command', () => {
    const command = read('src-tauri/src/commands/panel.rs')
    const service = read('src-tauri/src/services/panel.rs')
    expect(command).toContain('app: AppHandle')
    expect(service).toContain('if mode == "bots"')
    expect(service).toContain('ensure_current_bot_plugin(app, root_path)')
    expect(service).toContain('BOT_PLUGIN_HIGHER_VERSION_UNTRUSTED')
  })

  it('does not change the manual installation IPC', () => {
    expect(read('src/services/tauri/cs2.ts')).toContain("invoke<OperationResult>('install_bot_package', { rootPath, keepBackup })")
    expect(read('src/views/InstallView.vue')).not.toContain('inspect_bot_plugin_version')
  })
})
