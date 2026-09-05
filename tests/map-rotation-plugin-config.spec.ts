import { readFileSync } from 'node:fs'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  'third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs',
  'utf8',
)

describe('MapRotation plugin config contract', () => {
  it('reads only the assistant lower-case boolean schema and reports invalid inputs', () => {
    expect(source).toContain('TryGetProperty("enabled", out var enabled)')
    expect(source).toContain('JsonValueKind.True or JsonValueKind.False')
    expect(source).toContain('enabled-missing')
    expect(source).toContain('enabled-not-boolean')
    expect(source).toContain('invalid-json')
    expect(source).toContain('[MAP_ROTATION_CONFIG_INVALID]')
    expect(source).toContain('source=config')
  })

  it('keeps the single-load memory model and disabled automatic-rotation gates', () => {
    expect(source).toContain('_enabled = LoadDefaultEnabled();')
    expect(source).toContain('if (!_enabled)')
    expect(source).toContain('lbtv_map_next')
    expect(source).not.toContain('File.WriteAll')
  })
})
