import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(path, 'utf8')

describe('CS2 手动确认解锁契约', () => {
  it('后端暴露会话确认，并让写入 gate 读取确认状态', () => {
    const cs2 = read('src-tauri/src/services/cs2.rs')
    const commands = read('src-tauri/src/commands/cs2.rs')
    const bot = read('src-tauri/src/services/bot_difficulty.rs')
    const panel = read('src-tauri/src/services/panel.rs')
    expect(cs2).toContain('confirm_cs2_closed')
    expect(cs2).toContain('check_cs2_process_for_write')
    expect(commands).toContain('revoke_cs2_closed_confirmation')
    expect(bot).toContain('check_cs2_process_for_write(root_path)')
    expect(panel).toContain('check_cs2_process_for_write')
  })

  it('前端使用同一个运行时解锁状态，不持久化确认', () => {
    const store = read('src/stores/cs2.ts')
    const strip = read('src/components/StatusStrip.vue')
    const overview = read('src/views/OverviewView.vue')
    expect(store).toContain('writeUnlocked')
    expect(store).toContain('confirmClosedByPlayer')
    expect(strip).toContain('我确认 CS2 已关闭，解锁本次会话')
    expect(overview).toContain('cs2.writeUnlocked')
  })
})
