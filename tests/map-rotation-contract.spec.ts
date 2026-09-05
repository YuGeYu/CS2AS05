import { readFileSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  'third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs',
  'utf8',
)

describe('MapRotation release contract', () => {
  it('starts automatic rotation enabled while retaining an explicit pause switch', () => {
    expect(source).toContain('_enabled = LoadDefaultEnabled();')
    expect(source).toContain('lbtv_map_rotation [0|1]')
    expect(source).toContain('ScheduleNextMap')
    expect(source).toContain('System.Text.Json')
    expect(source).toContain('AutoChangeDelaySeconds = 15.0f')
    expect(source).toContain('DefaultConfigRelativePath')
    expect(source).toContain('lbtv_map_next')
  })

  it('ships the external default config in the release zip', () => {
    const zip = readFileSync('src-tauri/resources/CS2BotImprover.zip')
    expect(zip.length).toBeGreaterThan(1)
    expect(createHash('sha256').update(zip).digest('hex').toUpperCase()).toBe('634BC9B854A0F39349474EC73463E4CAED6454BB0344DA7C89B9A1D2D268FE7F')
  })
})
