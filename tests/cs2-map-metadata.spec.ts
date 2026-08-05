import { describe, expect, it } from 'vitest'
import { CS2_MAP_METADATA, findCs2Map, mapLayer, scaleDemoCoordinate } from '@/data/cs2-map-metadata'

describe('CS2 map metadata', () => {
  it('keeps the fixed upstream map set unique', () => {
    expect(CS2_MAP_METADATA).toHaveLength(44)
    expect(new Set(CS2_MAP_METADATA.map((map) => map.name)).size).toBe(CS2_MAP_METADATA.length)
  })

  it('maps the dust2 origin to the top-left radar pixel', () => {
    const dust2 = findCs2Map('de_dust2')
    expect(dust2).toBeDefined()
    expect(scaleDemoCoordinate(dust2!, 1024, -2476, 3239)).toEqual({ x: 0, y: 0 })
  })

  it('selects lower layers using the upstream Z threshold', () => {
    const nuke = findCs2Map('de_nuke')
    const vertigo = findCs2Map('de_vertigo')
    expect(mapLayer(nuke!, -496)).toBe('lower')
    expect(mapLayer(nuke!, -495)).toBe('upper')
    expect(mapLayer(vertigo!, 11699)).toBe('lower')
    expect(mapLayer(vertigo!, 11700)).toBe('upper')
  })

  it('returns no metadata for an unknown map', () => {
    expect(findCs2Map('workshop_unknown')).toBeUndefined()
  })
})
