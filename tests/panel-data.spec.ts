import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

import { COMMANDS_TXT, parseCommands, parseTeams, TEAMS } from '@/data/panel/commands'
import { KNIVES } from '@/data/panel/knives'
import { captureKeyName } from '@/features/panel/key-capture'

describe('Panel v1.4.2 static contract', () => {
  it('bundles the fixed upstream commands file without transformation', () => {
    const bytes = readFileSync('src/data/panel/commands.txt')
    expect(createHash('sha256').update(bytes).digest('hex').toUpperCase()).toBe('185893ADB080565E77447066E42256C58E76A8343459C0C7C3ED1D723A21C139')
    expect(COMMANDS_TXT).toContain('ADD TEAMS')
    expect(parseCommands()).toHaveLength(176)
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
