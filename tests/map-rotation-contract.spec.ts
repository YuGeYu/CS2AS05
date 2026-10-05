import { readFileSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { describe, expect, it } from 'vitest'

const zip = readFileSync('src-tauri/resources/CS2BotImprover.zip')

describe('官方 v1.4.5 资源包边界', () => {
  it('使用已核验的上游整包，不把下游自动换图或 BOT AI 聊天注入资源包', () => {
    expect(zip.length).toBeGreaterThan(1)
    expect(createHash('sha256').update(zip).digest('hex').toUpperCase()).toBe('AE37B86533ABFE0547C5AD4346D478CD846727509FC092842A81240EB0130140')
    const text = zip.toString('binary')
    expect(text).not.toContain('MapRotation')
    expect(text).not.toContain('CS2BotLlmChat')
  })
})
