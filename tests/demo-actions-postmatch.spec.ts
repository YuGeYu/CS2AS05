import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

const read = (path: string) => readFileSync(path, 'utf8')

describe('Demo actions and post-match contracts', () => {
  it('keeps Play and FolderSearch as accessible stable row actions', () => {
    const view = read('src/views/DemoReviewView.vue')
    expect(view).toContain('Play')
    expect(view).toContain('FolderSearch')
    expect(view).toContain('aria-label="用 CS2 播放 Demo"')
    expect(view).toContain('aria-label="在文件夹中显示 Demo"')
    expect(view).toContain('demo.rowBusy[item.id]')
  })

  it('uses argument arrays and never accepts an arbitrary reveal path from the frontend', () => {
    const playback = read('src-tauri/src/demo/playback.rs')
    const service = read('src/services/tauri/demo.ts')
    expect(playback).toContain('Command::new("explorer.exe")')
    expect(playback).toContain('.arg("/select,")')
    expect(playback).toContain('.arg(path)')
    expect(service).toContain("invoke<RevealDemoResult>('reveal_demo_file', { demoId })")
  })

  it('replaces the old done-list race with exact session events', () => {
    const service = read('src-tauri/src/services/demo.rs')
    const coordinator = read('src-tauri/src/demo/post_match.rs')
    const shell = read('src/components/AppShell.vue')
    expect(service).not.toContain('observe_assistant_launch')
    expect(coordinator).toContain('candidate.id')
    expect(coordinator).toContain('presentable_report')
    expect(shell).toContain('handledPostMatchSessions')
    expect(shell).toContain("listen<PostMatchReportFailed>('demo://report-failed'")
  })
})
