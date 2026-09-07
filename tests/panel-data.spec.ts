import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

import { COMMANDS_TXT, parseCommands, parseTeams, TEAMS } from '@/data/panel/commands'
import { KNIVES } from '@/data/panel/knives'
import { captureKeyName } from '@/features/panel/key-capture'

describe('Panel v1.4.4 static contract', () => {
  it('bundles the pinned upstream commands plus the two 0.5.4 commands', () => {
    const bytes = readFileSync('src/data/panel/commands.txt')
    expect(createHash('sha256').update(bytes).digest('hex').toUpperCase()).toBe('19BF9A3A2493DEA8355CDCC78E832D79EABD5950DB252B37BFE6F4C8652C7853')
    expect(COMMANDS_TXT).toContain('ADD TEAMS')
    expect(parseCommands()).toHaveLength(184)
    expect(parseCommands().filter(entry => entry.copy === 'br_reroll')).toHaveLength(1)
    expect(parseCommands().filter(entry => entry.copy === 'bot_nades less')).toHaveLength(1)
  })

  it('pins the v1.4.4 fixture to the bundled resource and Panel summaries', () => {
    const fixture = JSON.parse(readFileSync('tests/fixtures/panel-v1.4.3/manifest.json', 'utf8'))
    const bundle = readFileSync('src-tauri/resources/CS2BotImprover.zip')
    expect(createHash('sha256').update(bundle).digest('hex').toUpperCase()).toBe(fixture.bundleSha256)
    expect(fixture).toMatchObject({
      source: 'ed0ard/CS2-Bot-Improver@v1.4.4',
      panelSha256: '2797A3FE85E65959CAE9501525B67B3876CEF65152E88DC716F64D5485AC2182',
      commands: { sha256: '19BF9A3A2493DEA8355CDCC78E832D79EABD5950DB252B37BFE6F4C8652C7853', parsedEntries: 184, teams: 42 },
    })
  })

  it('parses all 42 complete CT/T team presets', () => {
    expect(TEAMS).toHaveLength(42)
    expect(TEAMS.slice(-2).map(team => team.name)).toEqual(['9z', 'FUT'])
    expect(parseTeams(COMMANDS_TXT).every(team => team.ct.startsWith('bot_add_ct') && team.t.startsWith('bot_add_t'))).toBe(true)
  })

  it('keeps the fixed 20-item knife allowlist', () => {
    expect(KNIVES.map(item => item.id)).toEqual([500, 503, 505, 506, 507, 508, 509, 512, 514, 515, 516, 517, 518, 519, 520, 521, 522, 523, 525, 526])
    expect(new Set(KNIVES.map(item => item.image)).size).toBe(20)
  })

  it('maps browser keyboard codes to Source bind names', () => {
    expect(captureKeyName({ code: 'KeyF', key: 'f' } as KeyboardEvent)).toBe('f')
    expect(captureKeyName({ code: 'Numpad7', key: '7' } as KeyboardEvent)).toBe('kp_7')
    expect(captureKeyName({ code: 'Backslash', key: '\\' } as KeyboardEvent)).toBe('\\')
    expect(captureKeyName({ code: 'MediaPlayPause', key: 'MediaPlayPause' } as KeyboardEvent)).toBeNull()
  })

  it('keeps match navigation scoped to the command list scroller', () => {
    const source = readFileSync('src/views/CommandsView.vue', 'utf8')
    expect(source).not.toContain('scrollIntoView')
    expect(source).toContain('list.scrollTop +=')
  })
})
