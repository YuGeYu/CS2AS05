import { readFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { describe, expect, it } from 'vitest'

const overview = readFileSync(resolve('src/views/OverviewView.vue'), 'utf8')
const workbench = readFileSync(resolve('src/components/BotDifficultyWorkbench.vue'), 'utf8')
const service = readFileSync(resolve('src/services/tauri/bot-difficulty.ts'), 'utf8')
const commands = readFileSync(resolve('src-tauri/src/commands/bot_difficulty.rs'), 'utf8')
const serviceRust = readFileSync(resolve('src-tauri/src/services/bot_difficulty.rs'), 'utf8')

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
    expect(service).toContain("invoke<BotProfileOperation>('rename_bot_profile'")
    expect(service).toContain("invoke<BotProfileOperation>('delete_bot_profile'")
    expect(workbench).not.toContain('localStorage')
  })

  it('把 VPK 工作移出 Tauri 主线程，并丢弃过期的档案读取结果', () => {
    expect(commands).toContain('spawn_blocking')
    expect(commands).toContain('pub async fn open_bot_profile')
    expect(workbench).toContain('requestSequence')
    expect(workbench).toContain('sequence !== requestSequence.value')
    expect(workbench).toContain('requestDelete(profile)')
    expect(workbench).toContain('当前档案有未保存修改')
  })

  it('提取失败具备独立错误码、有限重试、可写性探测与陈旧目录清理', () => {
    expect(serviceRust).toContain('BOT_WORKSHOP_EXTRACT_FAILED')
    expect(serviceRust).toContain('BOT_WORKSHOP_WORKSPACE_UNWRITABLE')
    expect(serviceRust).toContain('EXTRACT_MAX_ATTEMPTS')
    expect(serviceRust).toContain('cleanup_stale_workspace')
    expect(serviceRust).toContain('STALE_WORKSPACE_AGE')
    expect(serviceRust).not.toContain('[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID] extract 退出码')
  })

  it('保存前校验编辑内容并向玩家解释重打包与回读失败', () => {
    expect(serviceRust).toContain('validate_botprofile_text')
    expect(serviceRust).toContain('BOT_WORKSHOP_DB_INVALID')
    expect(serviceRust).toContain('BOT_WORKSHOP_REPACK_FAILED')
    expect(serviceRust).toContain('BOT_WORKSHOP_VPK_ROUNDTRIP_MISMATCH')
    expect(serviceRust).toContain('candidate-output-')
    expect(serviceRust).toContain('"--output"')
    expect(workbench).toContain('档案内容格式不完整')
    expect(workbench).toContain('VPK 重打包失败')
  })
})
