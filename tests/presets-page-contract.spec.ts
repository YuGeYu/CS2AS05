import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

describe('人机预设页面 UI contract', () => {
  const source = readFileSync('src/views/PresetsView.vue', 'utf8')

  it('keeps the Chinese workbench hierarchy and existing value controls', () => {
    expect(source).toContain('BOT 行为工作台')
    expect(source).toContain('瞄准策略')
    expect(source).toContain('投掷物频率')
    expect(source).toContain('本地 BOT 对局')
    expect(source).toContain('退出 CS2 后应用')
    expect(source).toContain('复制命令')
    expect(source).toContain("@update:model-value=\"setAim\"")
    expect(source).toContain("@update:model-value=\"setNades\"")
  })

  it('exposes accessible status and command-preview affordances', () => {
    expect(source).toContain('aria-live="polite"')
    expect(source).toContain('aria-label="当前配置摘要"')
    expect(source).toContain('id="team-select"')
    expect(source).toContain(':title="commandPreview"')
    expect(source).toContain('请先选择队伍')
  })
})
