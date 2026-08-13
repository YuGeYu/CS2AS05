import { createHash } from 'node:crypto'
import { readdir, readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'
import process from 'node:process'

const repo = process.cwd()
const upstreamRoot = path.join(repo, 'third_party', 'CS2-Skin-Forge', 'upstream', 'Panel', 'src', 'data')
const generatedRoot = path.join(repo, 'src', 'features', 'skin-forge', 'data', 'generated')
const output = path.join(generatedRoot, 'catalog-manifest.json')
const files = ['weapons.ts', 'weaponImages.ts', 'skins.ts', 'stickers.ts', 'knives.ts', 'knifeSkins.ts', 'keychains.ts', 'localNames.ts', 'skinNamesEn.ts', 'nameMap.json']

function sha256(data) { return createHash('sha256').update(data).digest('hex').toUpperCase() }
function entriesIn(text, start, end) {
  const section = text.slice(text.indexOf(start), end ? text.indexOf(end) : undefined)
  return [...section.matchAll(/\{\s*id:\s*(\d+)/g)].map((match) => Number(match[1]))
}
function duplicateCount(values) { return values.length - new Set(values).size }

const sourceFiles = {}
const generatedFiles = {}
for (const file of files) {
  const source = await readFile(path.join(upstreamRoot, file))
  const generated = await readFile(path.join(generatedRoot, file))
  sourceFiles[file] = { bytes: source.byteLength, sha256: sha256(source) }
  generatedFiles[file] = { bytes: generated.byteLength, sha256: sha256(generated) }
}

const [weapons, skins, stickers, knives, knifeSkins, keychains] = await Promise.all(
  ['weapons.ts', 'skins.ts', 'stickers.ts', 'knives.ts', 'knifeSkins.ts', 'keychains.ts'].map((file) => readFile(path.join(generatedRoot, file), 'utf8')),
)
const weaponIds = [...weapons.matchAll(/\{\s*defindex:\s*(\d+),\s*name:/g)].map((match) => Number(match[1]))
const weaponPaintIds = entriesIn(skins, 'export const weaponPaints', 'export const knifePaints')
const glovePaintIds = entriesIn(skins, 'export const gloves', 'export const musicKits')
const musicIds = entriesIn(skins, 'export const musicKits', 'export const agentModels')
const agentIds = [...skins.slice(skins.indexOf('export const agentModels'), skins.indexOf('export const popularStickers')).matchAll(/\{\s*id:\s*['"]([^'"]+)/g)].map((match) => match[1])
const stickerIds = entriesIn(stickers, 'export const allStickers')
const knifeIds = [...knives.matchAll(/\{\s*defindex:\s*(\d+),\s*name:/g)].map((match) => Number(match[1]))
const knifePaintIds = entriesIn(knifeSkins, 'export const knifeSkinsByType')
const keychainIds = entriesIn(keychains, 'export const allKeychains')

const manifest = {
  schema: 1,
  upstream: {
    repo: 'https://github.com/kaecho/CS2-Skin-Forge',
    tag: 'v1.8.2',
    commit: '75f52fbd5fd0616dbbdd09a65c3a1981593400d1',
    previousCommit: 'b2edea17db9128609dd41f726f179cd965206433',
  },
  counts: {
    weapons: weaponIds.length,
    weaponPaintRows: weaponPaintIds.length,
    uniqueWeaponPaintKits: new Set(weaponPaintIds).size,
    maxWeaponPaintKit: Math.max(...weaponPaintIds),
    knives: knifeIds.length,
    knifePaintRows: knifePaintIds.length,
    gloves: (skins.slice(skins.indexOf('export const gloves'), skins.indexOf('export const musicKits')).match(/defindex:/g) ?? []).length,
    glovePaintRows: glovePaintIds.length,
    agents: agentIds.length,
    musicKits: musicIds.length,
    stickers: stickerIds.length,
    keychains: keychainIds.length,
  },
  duplicates: {
    weapons: duplicateCount(weaponIds),
    knives: duplicateCount(knifeIds),
    agents: duplicateCount(agentIds),
    musicKits: duplicateCount(musicIds),
    stickers: duplicateCount(stickerIds),
    keychains: duplicateCount(keychainIds),
  },
  files: { source: sourceFiles, generated: generatedFiles },
}

await writeFile(output, `${JSON.stringify(manifest, null, 2)}\n`)
const vendorRoot = path.join(repo, 'third_party', 'CS2-Skin-Forge', 'upstream')
async function walk(directory) {
  const result = []
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const absolute = path.join(directory, entry.name)
    if (entry.isDirectory()) result.push(...await walk(absolute))
    else result.push(absolute)
  }
  return result
}
const vendorHashes = []
for (const absolute of (await walk(vendorRoot)).sort()) {
  const relative = path.relative(vendorRoot, absolute).replaceAll('\\', '/')
  vendorHashes.push(`${sha256(await readFile(absolute))}  ${relative}`)
}
await writeFile(path.join(repo, 'third_party', 'CS2-Skin-Forge', 'SHA256SUMS.txt'), `${vendorHashes.join('\n')}\n`)
console.log(JSON.stringify(manifest.counts))
