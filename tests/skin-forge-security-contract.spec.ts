import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('skin forge security contracts', () => {
  it('scopes the Tauri asset protocol only to the image cache', () => {
    const config = JSON.parse(readFileSync('src-tauri/tauri.conf.json', 'utf8'))
    const protocol = config.app.security.assetProtocol
    expect(protocol.enable).toBe(true)
    expect(protocol.scope).toEqual(['$APPLOCALDATA/skin-forge/cache/images/**'])
    expect(protocol.scope).not.toContain('$HOME/**')
    expect(protocol.scope).not.toContain('$LOCALDATA/**')
    expect(protocol.scope).not.toContain('**')
  })

  it('checks plugin readiness before draft and target writes', () => {
    const source = readFileSync('src-tauri/src/commands/skin_forge.rs', 'utf8')
    const save = source.slice(source.indexOf('pub fn skin_forge_save_loadout'), source.indexOf('#[tauri::command]\npub fn skin_forge_reset_loadout'))
    const reset = source.slice(source.indexOf('pub fn skin_forge_reset_loadout'), source.indexOf('#[tauri::command]\npub fn skin_forge_check_plugin'))
    expect(save.indexOf('require_plugin_ready')).toBeLessThan(save.indexOf('write_atomically'))
    expect(reset.indexOf('require_plugin_ready')).toBeLessThan(reset.indexOf('write_atomically'))
    expect(source).toContain('[PLAYER_SKIN_MOD_REQUIRED]')
  })
})
