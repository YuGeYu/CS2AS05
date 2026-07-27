import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { inflateSync } from 'node:zlib'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { describe, expect, it } from 'vitest'

import { KNIVES } from '@/data/panel/knives'
import { usePanelStore } from '@/stores/panel'
import KnivesView from '@/views/KnivesView.vue'

interface ManifestItem {
  id: number
  file: string
  sha256: string
  bytes: number
  width: number
  height: number
  source: string
}

const manifest = JSON.parse(readFileSync('src/assets/knives/manifest.json', 'utf8')) as { items: ManifestItem[] }

function nonTransparentRatio(png: Buffer) {
  expect(png.readUInt32BE(16)).toBeGreaterThan(0)
  expect(png.readUInt32BE(20)).toBeGreaterThan(0)
  expect(png[24]).toBe(8)
  expect(png[25]).toBe(6)
  const width = png.readUInt32BE(16)
  const height = png.readUInt32BE(20)
  const chunks: Buffer[] = []
  for (let offset = 8; offset < png.length;) {
    const length = png.readUInt32BE(offset)
    const type = png.toString('ascii', offset + 4, offset + 8)
    if (type === 'IDAT') chunks.push(png.subarray(offset + 8, offset + 8 + length))
    offset += length + 12
  }
  const raw = inflateSync(Buffer.concat(chunks))
  const stride = width * 4
  let previous = Buffer.alloc(stride)
  let visible = 0
  for (let row = 0; row < height; row++) {
    const filter = raw[row * (stride + 1)]
    const source = raw.subarray(row * (stride + 1) + 1, (row + 1) * (stride + 1))
    const decoded = Buffer.alloc(stride)
    for (let index = 0; index < stride; index++) {
      const left = index >= 4 ? decoded[index - 4] : 0
      const up = previous[index]
      const upperLeft = index >= 4 ? previous[index - 4] : 0
      const predictor = left + up - upperLeft
      const pa = Math.abs(predictor - left)
      const pb = Math.abs(predictor - up)
      const pc = Math.abs(predictor - upperLeft)
      const paeth = pa <= pb && pa <= pc ? left : pb <= pc ? up : upperLeft
      const adjustment = [0, left, up, Math.floor((left + up) / 2), paeth][filter]
      decoded[index] = (source[index] + adjustment) & 0xFF
    }
    for (let index = 3; index < stride; index += 4) if (decoded[index] > 0) visible++
    previous = decoded
  }
  return visible / (width * height)
}

describe('knife image contract', () => {
  it('pins 20 decodable, non-placeholder Panel resources', () => {
    expect(manifest.items.map(item => item.id)).toEqual(KNIVES.map(knife => knife.id))
    for (const item of manifest.items) {
      const bytes = readFileSync(`src/assets/knives/${item.file}`)
      expect(bytes.length).toBe(item.bytes)
      expect(createHash('sha256').update(bytes).digest('hex').toUpperCase()).toBe(item.sha256)
      expect(bytes.readUInt32BE(16)).toBe(item.width)
      expect(bytes.readUInt32BE(20)).toBe(item.height)
      expect(nonTransparentRatio(bytes)).toBeGreaterThan(0.02)
      expect(item.source).toContain(`/assets/${item.id}-`)
    }
  })

  it('renders an accessible, stable card for every knife', () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const panel = usePanelStore()
    panel.snapshot = {
      rootPath: 'root', ready: true, missingFiles: [], cs2Running: false,
      mode: { current: 'bots', insecure: true, writable: true },
      difficulty: { current: 'Low', available: ['Low', 'Medium', 'High'] },
      presets: { aim: 'mixed', nades: 'normal', writable: true },
      botItems: { profiles: true, agents: true, music: true, weapons: true, knives: true, gloves: true, stickers: true, charms: true, writable: true },
      dropKnives: { bindKey: '\\', selected: KNIVES.map(knife => knife.id), writable: true },
    }
    const wrapper = mount(KnivesView, { global: { plugins: [pinia] } })
    const cards = wrapper.findAll('.knife-grid button')
    expect(cards).toHaveLength(20)
    cards.forEach((card, index) => {
      const knife = KNIVES[index]
      expect(card.attributes('aria-pressed')).toBe('true')
      expect(card.text()).toContain(knife.name)
      expect(card.text()).toContain(`subclass ${knife.id}`)
      expect(card.get('img').attributes('alt')).toBe(knife.name)
      expect(card.get('img').attributes('src')).toBe(knife.image)
    })
    const css = readFileSync('src/styles/main.css', 'utf8')
    expect(css).toContain('aspect-ratio: 4 / 3')
    expect(css).toContain('object-fit: contain')
  })
})
