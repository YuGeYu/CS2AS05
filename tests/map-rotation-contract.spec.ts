import { readFileSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { describe, expect, it } from 'vitest'

const source = readFileSync(
  'third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs',
  'utf8',
)

describe('MapRotation release contract', () => {
  it('starts automatic rotation disabled while retaining an explicit enable switch', () => {
    expect(source).toContain('_enabled = LoadDefaultEnabled();')
    expect(source).toContain('lbtv_map_rotation [0|1]')
    expect(source).toContain('ScheduleNextMap')
    expect(source).toContain('System.Text.Json')
    expect(source).toContain('AutoChangeDelaySeconds = 15.0f')
    expect(source).toContain('DefaultConfigRelativePath')
    expect(source).toContain('private const bool DefaultEnabled = false;')
    expect(source).toContain('lbtv_map_next')
  })

  it('ships the external default config in the release zip', () => {
    const zip = readFileSync('src-tauri/resources/CS2BotImprover.zip')
    expect(zip.length).toBeGreaterThan(1)
    expect(createHash('sha256').update(zip).digest('hex').toUpperCase()).toBe('EC3280E6848CA06B094FBAC2741674BA7E5586DF6679DB8D6ECC3B43109BB648')
  })
})
