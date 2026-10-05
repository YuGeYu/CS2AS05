import { execFileSync } from 'node:child_process'
import { describe, expect, it } from 'vitest'

const zipPath = 'src-tauri/resources/CS2BotImprover.zip'
const zipEntry = (name: string) => execFileSync('tar', ['-xOf', zipPath, name])

describe('gameinfo official and BOT variants', () => {
  it('keeps the v1.4.5 root and Online entries present', async () => {
    const root = zipEntry('gameinfo.gi')
    const online = zipEntry('backup/Online/gameinfo.gi')
    expect(root.length).toBeGreaterThan(0)
    expect(online.length).toBeGreaterThan(0)
  })

  it('adds exactly the two BOT SearchPaths without polluting Online', async () => {
    const online = zipEntry('backup/Online/gameinfo.gi').toString('utf8')
    const bots = zipEntry('backup/WithBots/gameinfo.gi').toString('utf8')
    expect(online).not.toContain('csgo/overrides/botprofile.vpk')
    expect(online).not.toContain('csgo/addons/metamod')
    expect((bots.match(/csgo\/overrides\/botprofile\.vpk/g) || []).length).toBe(1)
    expect((bots.match(/csgo\/addons\/metamod/g) || []).length).toBe(1)
    expect(bots.indexOf('csgo/overrides/botprofile.vpk')).toBeLessThan(bots.indexOf('csgo/addons/metamod'))
  })

  it('keeps the official v1.4.5 archive free of downstream markers', () => {
    expect(() => zipEntry('addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json')).toThrow(/./)
  })

  it('ships the bundled NadeSystem binary', () => {
    const bundled = zipEntry('addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll')
    expect(bundled.length).toBeGreaterThan(0)
  })

  it('ships every file checked by the upstream Panel after installation', () => {
    for (const name of [
      'cfg/gamemode_armsrace.cfg', 'cfg/gamemode_casual.cfg',
      'cfg/gamemode_competitive.cfg', 'cfg/gamemode_competitive2v2.cfg',
      'cfg/gamemode_deathmatch.cfg', 'cfg/gamemode_dm_freeforall.cfg',
      'cfg/gamemode_retakecasual.cfg', 'cfg/gamemode_teamdeathmatch.cfg',
      'cfg/gamemode_workshop.cfg', 'cfg/my_bot_rush_config.cfg', 'gameinfo.gi',
    ]) expect(zipEntry(name).length).toBeGreaterThan(0)
  })
  it('ships every BOT difficulty VPK required by the panel', () => {
    for (const name of [
      'overrides/Low/botprofile.vpk',
      'overrides/Medium/botprofile.vpk',
      'overrides/High/botprofile.vpk',
      'overrides/botprofile.vpk',
    ]) {
      expect(zipEntry(name).length).toBeGreaterThan(0)
    }
  })

  it('does not require a downstream manifest in the official archive', () => {
    expect(() => zipEntry('gameinfo.manifest.json')).toThrow(/./)
  })
})
