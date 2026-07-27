import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

import { COMMANDS_TXT, parseCommands, parseTeams, TEAMS } from '@/data/panel/commands'
import { KNIVES } from '@/data/panel/knives'
import { captureKeyName } from '@/features/panel/key-capture'

describe('Panel v1.4.3 static contract', () => {
  it('bundles the pinned upstream commands plus the two 0.5.4 commands', () => {
    const bytes = readFileSync('src/data/panel/commands.txt')
    expect(createHash('sha256').update(bytes).digest('hex').toUpperCase()).toBe('E0D7C5F1247DE1E766A75CA656AFBED22BF9309FB104FFA2E83AD505E53101E7')
    expect(COMMANDS_TXT).toContain('ADD TEAMS')
    expect(parseCommands()).toHaveLength(178)
    expect(parseCommands().filter(entry => entry.copy === 'br_reroll')).toHaveLength(1)
    expect(parseCommands().filter(entry => entry.copy === 'bot_nades less')).toHaveLength(1)
  })

  it('pins the v1.4.3 fixture to the bundled resource and Panel summaries', () => {
    const fixture = JSON.parse(readFileSync('tests/fixtures/panel-v1.4.3/manifest.json', 'utf8'))
    const bundle = readFileSync('src-tauri/resources/CS2BotImprover.zip')
    expect(createHash('sha256').update(bundle).digest('hex').toUpperCase()).toBe(fixture.bundleSha256)
    expect(fixture).toMatchObject({
      source: 'ed0ard/CS2-Bot-Improver@d1d83982db88fbdb686b2bf13aa8c6f9d65a4604',
      panelSha256: '3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD',
      commands: { sha256: 'E0D7C5F1247DE1E766A75CA656AFBED22BF9309FB104FFA2E83AD505E53101E7', parsedEntries: 178, teams: 40 },
    })
  })

  it('parses all 40 complete CT/T team presets', () => {
    expect(TEAMS).toHaveLength(40)
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
