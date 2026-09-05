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

  it('deletes only a selected library Demo after an in-app confirmation', () => {
    const view = read('src/views/DemoReviewView.vue')
    const frontend = read('src/services/tauri/demo.ts')
    const command = read('src-tauri/src/commands/demo.rs')
    const service = read('src-tauri/src/services/demo.rs')
    expect(view).toContain('aria-label="删除 Demo"')
    expect(view).toContain('role="alertdialog"')
    expect(frontend).toContain("invoke<void>('delete_demo_file', { demoId })")
    expect(command).toContain('demo::delete_file(&app, demo_id)')
    expect(service).toContain('只允许删除录像库中登记的 .dem 文件')
    expect(service).toContain('DELETE FROM demo_files WHERE id=?1')
  })

  it('uses argument arrays and never accepts an arbitrary reveal path from the frontend', () => {
    const playback = read('src-tauri/src/demo/playback.rs')
    const service = read('src/services/tauri/demo.ts')
    expect(playback).toContain('Command::new("explorer.exe")')
    expect(playback).toContain('.arg("/select,")')
    expect(playback).toContain('.arg(path)')
    expect(service).toContain("invoke<RevealDemoResult>('reveal_demo_file', { demoId })")
  })

  it('exposes a separate terminal analysis-job delete action without deleting the Demo', () => {
    const view = read('src/views/DemoReviewView.vue')
    const frontend = read('src/services/tauri/demo.ts')
    const command = read('src-tauri/src/commands/demo.rs')
    const service = read('src-tauri/src/services/demo.rs')
    expect(view).toContain('aria-label="删除任务记录"')
    expect(view).toContain('删除这条分析任务？')
    expect(view).toContain('原始 Demo 文件、录像库记录和已有对局报告不会删除')
    expect(frontend).toContain("invoke<void>('delete_analysis_job', { jobId })")
    expect(command).toContain('demo::delete_analysis_job(&app, job_id)')
    expect(service).toContain("DELETE FROM analysis_jobs WHERE id=?1 AND stage IN ('error','canceled')")
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
