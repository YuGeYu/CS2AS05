import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

const overview = readFileSync('src/views/OverviewView.vue', 'utf8')
const styles = readFileSync('src/styles/main.css', 'utf8')

describe('概览本地对局启动台契约', () => {
  it('保留真实启动边界并提供统一环境摘要', () => {
    expect(overview).toContain('overview-environment')
    expect(overview).toContain('environmentState')
    expect(overview).toContain('本地 BOT 对局，使用 -insecure')
    expect(overview).toContain('不修改在线模式文件')
    expect(overview).toContain('BotDifficultyWorkbench')
  })

  it('主启动面、快速配置和目录检查器具备响应式样式', () => {
    expect(overview).toContain('overview-hero')
    expect(overview).toContain('overview-config-grid')
    expect(overview).toContain('overview-side-panel')
    expect(styles).toContain('.overview-launchpad')
    expect(styles).toContain('@media (max-width: 700px)')
    expect(styles).toContain('@media (prefers-reduced-motion: reduce)')
  })
})
