import { createHash } from 'node:crypto'
import { cp, mkdir, readFile, readdir, stat, writeFile } from 'node:fs/promises'
import path from 'node:path'
import ts from 'typescript'

const [upstreamRoot] = process.argv.slice(2)
if (!upstreamRoot) throw new Error('Usage: node scripts/import-cs-demo-manager-maps.mjs <upstream-root>')

const commit = '8961f5072fe4d42803dde68e8e71b3c90b216504'
const sourceBase = path.join(upstreamRoot, 'static', 'images', 'maps', 'cs2')
const frontendBase = path.resolve('src', 'assets', 'maps', 'cs2')
const tauriBase = path.resolve('src-tauri', 'resources', 'demo-maps')
const manifestPath = path.resolve('third_party', 'cs-demo-manager', 'map-manifest.json')
const sha256 = async (file) => createHash('sha256').update(await readFile(file)).digest('hex').toUpperCase()
const manifest = { repository: 'https://github.com/akiver/cs-demo-manager', commit, generatedAt: '2026-07-30', assets: [] }

for (const kind of ['radars', 'thumbnails']) {
  const sourceDir = path.join(sourceBase, kind)
  const destinationDir = path.join(frontendBase, kind)
  await mkdir(destinationDir, { recursive: true })
  for (const name of (await readdir(sourceDir)).filter((entry) => entry.endsWith('.png')).sort()) {
    const source = path.join(sourceDir, name)
    const destination = path.join(destinationDir, name)
    await cp(source, destination)
    const info = await stat(destination)
    manifest.assets.push({ upstreamPath: `static/images/maps/cs2/${kind}/${name}`, downstreamPath: `src/assets/maps/cs2/${kind}/${name}`, bytes: info.size, sha256: await sha256(destination) })
    if (kind === 'radars') {
      const resource = path.join(tauriBase, 'radars', name)
      await mkdir(path.dirname(resource), { recursive: true })
      await cp(source, resource)
      const resourceInfo = await stat(resource)
      manifest.assets.push({ upstreamPath: `static/images/maps/cs2/${kind}/${name}`, downstreamPath: `src-tauri/resources/demo-maps/radars/${name}`, bytes: resourceInfo.size, sha256: await sha256(resource) })
    }
  }
}

const metadataSourcePath = path.join(upstreamRoot, 'src', 'node', 'database', 'maps', 'default-maps.ts')
const sourceText = await readFile(metadataSourcePath, 'utf8')
const sourceFile = ts.createSourceFile(metadataSourcePath, sourceText, ts.ScriptTarget.Latest, true)
let cs2Array
sourceFile.forEachChild((node) => {
  if (!ts.isFunctionDeclaration(node)) return
  node.body?.forEachChild((statement) => {
    if (!ts.isVariableStatement(statement)) return
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name) && declaration.name.text === 'cs2Maps' && ts.isArrayLiteralExpression(declaration.initializer)) cs2Array = declaration.initializer
    }
  })
})
if (!cs2Array) throw new Error('Unable to locate cs2Maps in upstream default-maps.ts')

const numberValue = (node) => {
  if (ts.isNumericLiteral(node)) return Number(node.text)
  if (ts.isPrefixUnaryExpression(node) && node.operator === ts.SyntaxKind.MinusToken && ts.isNumericLiteral(node.operand)) return -Number(node.operand.text)
  throw new Error(`Unsupported number node: ${node.getText(sourceFile)}`)
}
const maps = cs2Array.elements.map((element) => {
  if (!ts.isObjectLiteralExpression(element)) throw new Error('Expected map object')
  const result = {}
  for (const property of element.properties) {
    if (!ts.isPropertyAssignment(property)) continue
    const key = property.name.getText(sourceFile).replaceAll("'", '')
    if (key === 'game') continue
    result[key] = ts.isStringLiteral(property.initializer) ? property.initializer.text : numberValue(property.initializer)
  }
  return result
})

const resourceRadars = new Set(await readdir(path.join(tauriBase, 'radars')))
const metadataRecords = await Promise.all(maps.map(async (map) => {
  const radarAsset = `${map.name}.png`
  if (!resourceRadars.has(radarAsset)) throw new Error(`Missing radar asset for ${map.name}`)
  const lowerRadarAsset = resourceRadars.has(`${map.name}_lower.png`) ? `${map.name}_lower.png` : null
  return {
    name: map.name, positionX: map.position_x, positionY: map.position_y, scale: map.scale,
    thresholdZ: map.threshold_z, radarSize: 1024, radarAsset,
    radarSha256: await sha256(path.join(tauriBase, 'radars', radarAsset)), lowerRadarAsset,
    lowerRadarSha256: lowerRadarAsset ? await sha256(path.join(tauriBase, 'radars', lowerRadarAsset)) : null,
    upstreamCommit: commit,
  }
}))

const metadata = `// Mechanically converted from akiver/cs-demo-manager ${commit}. See third_party/cs-demo-manager/PROVENANCE.md.\n` +
`export interface Cs2MapMetadata {\n  name: string\n  positionX: number\n  positionY: number\n  scale: number\n  thresholdZ: number\n  radarSize: number\n  radarAsset: string\n  radarSha256: string\n  lowerRadarAsset: string | null\n  lowerRadarSha256: string | null\n  upstreamCommit: string\n}\n\n` +
`export const CS2_MAP_METADATA: readonly Cs2MapMetadata[] = ${JSON.stringify(metadataRecords, null, 2)}\n\n` +
`export const findCs2Map = (name: string) => CS2_MAP_METADATA.find((map) => map.name === name)\n\n` +
`export const scaleDemoCoordinate = (map: Cs2MapMetadata, imageSize: number, x: number, y: number) => ({\n  x: ((x - map.positionX) / map.scale) * imageSize / map.radarSize,\n  y: ((map.positionY - y) / map.scale) * imageSize / map.radarSize,\n})\n\n` +
`export const mapLayer = (map: Cs2MapMetadata, z: number): 'upper' | 'lower' => z < map.thresholdZ ? 'lower' : 'upper'\n`
await mkdir(path.resolve('src', 'data'), { recursive: true })
const tsPath = path.resolve('src', 'data', 'cs2-map-metadata.ts')
await writeFile(tsPath, metadata)
await mkdir(tauriBase, { recursive: true })
const jsonPath = path.join(tauriBase, 'cs2-map-metadata.json')
await writeFile(jsonPath, `${JSON.stringify(metadataRecords, null, 2)}\n`)
manifest.metadata = {
  upstreamPath: 'src/node/database/maps/default-maps.ts', downstreamPath: 'src/data/cs2-map-metadata.ts',
  mapCount: maps.length, sha256: await sha256(tsPath), rustResourcePath: 'src-tauri/resources/demo-maps/cs2-map-metadata.json',
  rustResourceSha256: await sha256(jsonPath),
}
await writeFile(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`)
console.log(`Imported ${manifest.assets.length} asset copies and ${maps.length} map records.`)
