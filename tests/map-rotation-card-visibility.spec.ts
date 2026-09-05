import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

const overview = readFileSync('src/views/OverviewView.vue', 'utf8')
const notice = readFileSync('src/components/MapRotationDefaultNotice.vue', 'utf8')
const drawer = readFileSync('src/components/AppearanceSettingsDrawer.vue', 'utf8')

describe('temporary MapRotation UI boundary', () => {
  it('removes the editable card from Overview without removing the capability files', () => {
    expect(overview).not.toContain('MapRotationDefaultControl')
    expect(overview).not.toContain('map_rotation_default')
  })

  it('hides Theme Settings entry while retaining capability source', () => {
    expect(drawer).not.toContain('自动换图默认状态（开发中）')
    expect(drawer).not.toContain('MapRotationDefaultNotice')
    expect(notice).toContain('功能开发中，当前暂时无效')
    expect(notice).toContain('助手中的配置回读不等于游戏插件当前运行状态')
    expect(notice).toContain('aria-modal="true"')
    expect(notice).toContain('关闭自动换图默认状态说明')
    expect(notice).toContain('@click.self="close"')
    expect(notice).not.toContain('get_map_rotation_default')
    expect(notice).not.toContain('set_map_rotation_default')
    expect(notice).not.toContain('reset_map_rotation_default')
  })
})
