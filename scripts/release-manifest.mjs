import { createHash } from 'node:crypto'
import { mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
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

const nsisDir = resolve(process.cwd(), 'src-tauri', 'target', 'release', 'bundle', 'nsis')
const files = await readdir(nsisDir)
const installerName = files.find((name) => name.endsWith('_x64-setup.exe') && name.includes(version))
if (!installerName) throw new Error(`signed NSIS installer for ${version} was not found in ${nsisDir}`)
const installerPath = resolve(nsisDir, installerName)
const signaturePath = `${installerPath}.sig`
const [installerBytes, signature, installerStat] = await Promise.all([
  readFile(installerPath),
  readFile(signaturePath, 'utf8'),
  stat(installerPath),
])

const manifest = {
  version,
  channel,
  projectId,
  notes: `Release manifest generated for ${channel}.`,
  pub_date: new Date().toISOString(),
  platforms: {
    'windows-x86_64': {
      // Keep local manifests portable and never leak the builder's filesystem path.
      // A deployment layer may replace this basename with its public download URL.
      installer: installerName,
      signature: signature.trim(),
      sha256: createHash('sha256').update(installerBytes).digest('hex').toUpperCase(),
      size: installerStat.size,
    },
  },
}

await mkdir(outDir, { recursive: true })
await writeFile(outFile, `${JSON.stringify(manifest, null, 2)}\n`, 'utf8')

console.log(`wrote ${outFile}`)
