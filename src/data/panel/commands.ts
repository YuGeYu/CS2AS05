import raw from './commands.txt?raw'

export const COMMANDS_TXT = raw

export interface Team { index: number; name: string; ct: string; t: string }

export function parseTeams(text: string): Team[] {
  const lines = text.split(/\r?\n/)
  const start = lines.findIndex(line => line.trim().toUpperCase() === 'ADD TEAMS')
  const endOffset = lines.slice(start + 1).findIndex(line => line.trim().toUpperCase() === 'COORDINATED BUY')
  const region = start < 0 ? [] : lines.slice(start + 1, endOffset < 0 ? lines.length : start + 1 + endOffset)
  const teams: Team[] = []
  let current: Team | undefined
  for (const line of region) {
    const trimmed = line.trim()
    const heading = trimmed.match(/^(\d+)\.\s*(.+)$/)
    if (heading) {
      current = { index: Number(heading[1]), name: heading[2]!, ct: '', t: '' }
      teams.push(current)
    } else if (current && trimmed.startsWith('bot_add_ct')) current.ct = trimmed
    else if (current && trimmed.startsWith('bot_add_t')) current.t = trimmed
  }
  return teams.filter(team => team.name && (team.ct || team.t))
}

export const TEAMS = parseTeams(raw)

const categoryLabels: Record<string, string> = {
  'GAME MODE': '游戏模式', 'CONNECTION': '连接', 'BOT AIM STYLE': 'Bot 瞄准',
  'BOT NADE THROWING': 'Bot 投掷物', 'BOT MANAGEMENT': 'Bot 管理', 'ADD TEAMS': '队伍预设',
  'COORDINATED BUY': '协同购买',
}

export type CommandEntry = { id: number; display: string; copy: string; category?: string; copyable: boolean }

export function parseCommands(text = raw): CommandEntry[] {
  let category = '常用命令'
  const entries: CommandEntry[] = []
  text.split(/\r?\n/).forEach((line, id) => {
    const trimmed = line.trim()
    if (!trimmed) return
    const translated = categoryLabels[trimmed.toUpperCase()]
    if (translated) {
      category = translated
      entries.push({ id, display: translated, copy: '', category, copyable: false })
      return
    }
    const team = trimmed.match(/^\d+\.\s*(.+)$/)
    if (team) {
      entries.push({ id, display: trimmed, copy: team[1]!.trim(), category, copyable: true })
      return
    }
    const takesArgument = /^(bot_add|bot_kick|bot_place|bot_goto_mark|bot_loadout)\b/i.test(trimmed)
    entries.push({ id, display: line, copy: takesArgument ? `${trimmed} ` : trimmed, category, copyable: true })
  })
  return entries
}
