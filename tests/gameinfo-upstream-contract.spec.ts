import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { execFileSync } from 'node:child_process'
import { describe, expect, it } from 'vitest'

const officialPath = 'D:/SteamLibrary/steamapps/common/Counter-Strike Global Offensive/game/csgo/gameinfo.gi'
const zipPath = 'src-tauri/resources/CS2BotImprover.zip'
const sha = (bytes: Uint8Array) => createHash('sha256').update(bytes).digest('hex').toUpperCase()
const zipEntry = (name: string) => execFileSync('tar', ['-xOf', zipPath, name])

describe('gameinfo official and BOT variants', () => {
  it('keeps Online/root bytes aligned with the current Steam baseline', async () => {
    const official = readFileSync(officialPath)
    const root = zipEntry('gameinfo.gi')
    const online = zipEntry('backup/Online/gameinfo.gi')
    expect(Buffer.compare(root, official)).toBe(0)
    expect(Buffer.compare(online, official)).toBe(0)
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

  it('keeps the bundled plugin marker on the current package version', () => {
    const packageVersion = JSON.parse(readFileSync('package.json', 'utf8')).version
    const marker = JSON.parse(execFileSync('tar', ['-xOf', zipPath, 'addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json'], { encoding: 'utf8' }))
    expect(marker.version).toBe(packageVersion)
  })

  it('ships the bundled NadeSystem binary', () => {
    const bundled = zipEntry('addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll')
    expect(bundled.length).toBeGreaterThan(0)
  })

  it('ships a manifest matching every upstream gameinfo entry', () => {
    const manifest = JSON.parse(execFileSync('tar', ['-xOf', zipPath, 'gameinfo.manifest.json'], { encoding: 'utf8' })) as {
      entries: Record<string, { sha256: string; size: number }>
    }
    for (const name of ['gameinfo.gi', 'backup/Online/gameinfo.gi', 'backup/WithBots/gameinfo.gi', 'backup/SkinOnly/gameinfo.gi']) {
      const bytes = zipEntry(name)
      expect(manifest.entries[name]).toEqual({ sha256: sha(bytes), size: bytes.length })
    }
    const skinOnly = zipEntry('backup/SkinOnly/gameinfo.gi').toString('utf8')
    expect(skinOnly).not.toContain('csgo/overrides')
    expect(skinOnly).not.toContain('botprofile.vpk')
    expect(Buffer.compare(zipEntry('backup/SkinOnly/gameinfo.gi'), readFileSync(officialPath))).toBe(0)
  })
})
