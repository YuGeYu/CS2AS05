import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const overview = readFileSync(resolve('src/views/OverviewView.vue'), 'utf8')
const workbench = readFileSync(resolve('src/components/BotDifficultyWorkbench.vue'), 'utf8')
const service = readFileSync(resolve('src/services/tauri/bot-difficulty.ts'), 'utf8')

describe('BOT 强度工坊契约', () => {
  it('生产概览提供受控入口并保留工坊组件', () => {
    expect(overview).toContain('自定义强度')
    expect(overview).toContain('BotDifficultyWorkbench')
    expect(workbench).toContain('只影响 BOT 模式，在线模式不会使用这些文件。')
    expect(workbench).toContain('aria-label="关闭人机强度工坊"')
  })

  it('前端只通过受限 profile ID 调用 Tauri，不接收任意路径', () => {
    expect(service).toContain("invoke<BotProfileList>('list_bot_profiles'")
    expect(service).toContain("invoke<BotProfileDocument>('open_bot_profile'")
    expect(service).toContain("invoke<BotProfileOperation>('save_bot_profile'")
    expect(workbench).not.toContain('localStorage')
  })
})
