import { mkdir, writeFile } from 'node:fs/promises'
import { resolve } from 'node:path'

// 发布事故记录：2026-06-01 升级 0.3.7 时，曾因跨文件批量替换版本号误改 Cargo.lock
// 第三方依赖版本，并被 PowerShell 写入 UTF-8 BOM，导致 Cargo 和 JSON 解析失败。
// 以后升级版本时只能精确修改项目自身版本字段，不能全局替换 0.x.y，也不能用
// PowerShell 默认 Set-Content 写回 JSON/TOML/lock 文件。
const version = process.env.npm_package_version ?? '0.1.0'
const channel = process.env.RELEASE_CHANNEL ?? process.env.VITE_APP_CHANNEL ?? 'dev'
const projectId = process.env.PROJECT_ID ?? process.env.VITE_DEFAULT_PROJECT_ID ?? 'cs2-bot-improver'
const outDir = resolve(process.cwd(), 'dist-release', projectId)
const outFile = resolve(outDir, `updater-${channel}.json`)

const manifest = {
  version,
  channel,
  projectId,
  notes: `Release manifest generated for ${channel}.`,
  pub_date: new Date().toISOString(),
  platforms: {},
}

await mkdir(outDir, { recursive: true })
await writeFile(outFile, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8')

console.log(`wrote ${outFile}`)
