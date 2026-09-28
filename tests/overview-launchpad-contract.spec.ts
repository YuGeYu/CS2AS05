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
    expect(overview).toContain('getInventorySimulatorStatus')
    expect(overview).toContain("nextMode === 'skin_only'")
    expect(overview).toContain("detail: 'inventory'")
    expect(overview).toContain('返回概览页面再次点击“启动 CS2”')
    expect(overview).toContain('仅在 BOT 模式下自动录制')
    expect(overview).toContain('由于技术限制，暂未开放手动调整')
    expect(overview).toContain(':disabled="true"')
    expect(overview).toContain('const recordingEnabled = computed(() => panel.snapshot?.mode.current === \'bots\')')
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
