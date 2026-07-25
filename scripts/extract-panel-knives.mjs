import { createHash } from 'node:crypto'
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { basename, dirname, join, resolve } from 'node:path'
import { brotliDecompressSync } from 'node:zlib'

const PANEL_SIZE = 5_839_872
const PANEL_SHA256 = '9C3AB83909E506C0D4BD4886C961DFC0E871DA71BB47E1D1BEA7EF2CCFE40AB2'
const KNIFE_ASSETS = [
  [500, '/assets/500-BKswtCUt.png'], [503, '/assets/503-sdv4wYJo.png'],
  [505, '/assets/505-Br3G-jWL.png'], [506, '/assets/506-D4N01_Y4.png'],
  [507, '/assets/507-DCFsT_US.png'], [508, '/assets/508-BxKQZIyO.png'],
  [509, '/assets/509-BJBFmupj.png'], [512, '/assets/512-D87kgCqM.png'],
  [514, '/assets/514-CeHKomc3.png'], [515, '/assets/515-DrzqOd2m.png'],
  [516, '/assets/516-uONSAF4X.png'], [517, '/assets/517-Br-grYKH.png'],
  [518, '/assets/518-pEq81KNT.png'], [519, '/assets/519-6ytRlVWg.png'],
  [520, '/assets/520-DmibQjrZ.png'], [521, '/assets/521-cMs5-Orc.png'],
  [522, '/assets/522-DORaOLGu.png'], [523, '/assets/523-Ct9YawQc.png'],
  [525, '/assets/525-B5R7N9fK.png'], [526, '/assets/526-qlTJ7JDk.png'],
]

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex').toUpperCase()
}

function pngDimensions(bytes) {
  if (!bytes.subarray(0, 8).equals(Buffer.from('89504E470D0A1A0A', 'hex'))) {
    throw new Error('Decoded asset is not a PNG')
  }
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) }
}

const panelPath = resolve(process.argv[2] ?? '.tmp-panel-extract/Panel v1.4.2.exe')
const outputDir = resolve(process.argv[3] ?? 'src/assets/knives')
const panel = readFileSync(panelPath)
if (panel.length !== PANEL_SIZE || sha256(panel) !== PANEL_SHA256) {
  throw new Error(`Unexpected Panel binary: ${panel.length} bytes, SHA256 ${sha256(panel)}`)
}

mkdirSync(outputDir, { recursive: true })
const items = KNIFE_ASSETS.map(([id, assetPath]) => {
  const marker = Buffer.from(assetPath)
  const markerOffset = panel.indexOf(marker)
  if (markerOffset < 0 || panel.indexOf(marker, markerOffset + 1) >= 0) {
    throw new Error(`Expected exactly one embedded resource marker for ${assetPath}`)
  }
  const png = brotliDecompressSync(panel.subarray(markerOffset + marker.length))
  const dimensions = pngDimensions(png)
  if (dimensions.width < 2 || dimensions.height < 2) {
    throw new Error(`Invalid PNG dimensions for subclass ${id}`)
  }
  const file = `${id}.png`
  writeFileSync(join(outputDir, file), png)
  return {
    id,
    file,
    sha256: sha256(png),
    bytes: png.length,
    ...dimensions,
    source: `Panel v1.4.2.exe embedded resource ${assetPath}`,
  }
})

const manifest = {
  upstream: 'ed0ard/CS2-Bot-Improver',
  tag: 'v1.4.2',
  commit: '97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc',
  panelFile: basename(panelPath),
  panelSha256: PANEL_SHA256,
  extraction: 'Brotli-decompressed Tauri embedded resources mapped by the numeric asset path',
  items,
}
writeFileSync(join(outputDir, 'manifest.json'), `${JSON.stringify(manifest, null, 2)}\n`)
console.log(`Extracted ${items.length} knife images (${items.reduce((sum, item) => sum + item.bytes, 0)} bytes) to ${dirname(join(outputDir, 'manifest.json'))}`)
