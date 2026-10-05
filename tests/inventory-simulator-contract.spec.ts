import { readFileSync } from 'node:fs'

import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(path, 'utf8')

describe('Inventory Simulator migration contract', () => {
  it('uses the InventorySimulator 3.5.0 release with its upstream ws and pet defaults', () => {
    const convars = read('third_party/cs2-css-inventory-simulator/upstream/source/InventorySimulator/Services/ConVars.cs')
    const notice = read('NOTICE.md')
    expect(convars).toMatch(/invsim_ws_enabled[\s\S]*?false/)
    expect(convars).toMatch(/invsim_ws_immediately[\s\S]*?false/)
    expect(convars).toMatch(/invsim_ws_cooldown[\s\S]*?30/)
    expect(convars).toMatch(/invsim_pet_enabled[\s\S]*?true/)
    expect(convars).toMatch(/invsim_pet_respawn[\s\S]*?false/)
    expect(convars).toMatch(/invsim_pet_respawn_warmup_only[\s\S]*?true/)
    expect(convars).toMatch(/invsim_pet_free_roam[\s\S]*?false/)
    expect(notice).toContain('3.5.0')
  })

  it('pins the bundled release manifest and runtime version gate', () => {
    const manifest = read('src-tauri/resources/inventory-simulator/manifest.json')
    const backend = read('src-tauri/src/commands/inventory_simulator.rs')
    expect(manifest).toContain('"resourceVersion": "3.5.0-release"')
    expect(manifest).toContain('157086c73718920285d3d761fe22c47a98090ec1')
    expect(manifest).toContain('9119DA8EFA655156DDB65D0F79F395A4DC1EEB158D3F357545797A8133FB8315')
    expect(backend).toContain('const RESOURCE_VERSION: &str = "3.5.0-release"')
    expect(backend).toContain('const UPSTREAM_TAG: &str = "3.5.0"')
    expect(backend).toContain('const UPSTREAM_COMMIT: &str = "157086c73718920285d3d761fe22c47a98090ec1"')
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
    for (const text of ['一键启用库存换肤', '打开饰品工坊', '复制 !ws', '启动本地 BOT', '第一次用，照着做就行', '暖身阶段复活', '宠物自由漫游']) {
      expect(view).toContain(text)
    }
    expect(view).toContain('inventory.cstrike.app')
    expect(view).toContain('invsim_ws_enabled true')
    expect(view).toContain('-insecure')
    expect(view).not.toContain('paintKit')
    expect(view).not.toContain('player_loadout')
    expect(view).not.toContain('input type="number"')
  })
})
