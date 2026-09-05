import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(path, 'utf8')

describe('Inventory Simulator migration contract', () => {
  it('uses only the fixed Ian Lucas runtime and enables the documented ws flow', () => {
    const convars = read('third_party/cs2-css-inventory-simulator/upstream/source/InventorySimulator/Services/ConVars.cs')
    const notice = read('NOTICE.md')
    expect(convars).toMatch(/invsim_ws_enabled[\s\S]*?true/)
    expect(convars).toMatch(/invsim_ws_immediately[\s\S]*?false/)
    expect(convars).toMatch(/invsim_ws_cooldown[\s\S]*?30/)
    expect(notice).toContain('5e3c96283b3d3f5aeba44822a38031df2e213376')
    expect(notice).toContain('3.1.0-cs2as.1')
  })

  it('registers the new Tauri surface and removes the old editor commands', () => {
    const lib = read('src-tauri/src/lib.rs')
    const config = read('src-tauri/tauri.conf.json')
    expect(lib).toContain('inventory_simulator_get_status')
    expect(lib).toContain('inventory_simulator_install')
    expect(lib).toContain('inventory_simulator_remove')
    expect(lib).toContain('inventory_simulator_open_workshop')
    expect(lib).not.toContain('skin_forge_')
    expect(config).toContain('resources/inventory-simulator')
    expect(config).not.toContain('resources/skin-forge')
    expect(config).not.toContain('$APPLOCALDATA/skin-forge/cache/images')
  })

  it('keeps deletion scoped to exact legacy targets', () => {
    const backend = read('src-tauri/src/commands/inventory_simulator.rs')
    expect(backend).toContain('const LEGACY_PLUGIN_NAME: &str = "PlayerSkinMod"')
    expect(backend).toContain('checked_child(&root, "skin-forge")')
    expect(backend).toContain('ensure_removable_directory')
    expect(backend).toContain('const CONFIG_RELATIVE: &str = "addons/counterstrikesharp/configs/plugins/InventorySimulator"')
    expect(backend).toContain('inventory_simulator_remove')
    expect(backend).toContain('拒绝清理非白名单目录')
    expect(backend).not.toContain('remove_dir_all(&plugin_parent)')
    expect(backend).not.toContain('remove_dir_all(&csgo)')
  })

  it('presents the complete simple player workflow without a local item editor', () => {
    const view = read('src/views/InventorySimulatorView.vue')
    for (const text of ['一键启用库存换肤', '打开饰品工坊', '复制 !ws', '启动本地 BOT', '第一次用，照着做就行']) {
      expect(view).toContain(text)
    }
    expect(view).toContain('inventory.cstrike.app')
    expect(view).toContain('-insecure')
    expect(view).not.toContain('paintKit')
    expect(view).not.toContain('player_loadout')
    expect(view).not.toContain('input type="number"')
  })
})
