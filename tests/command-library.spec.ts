import { describe, expect, it } from 'vitest'
import { readFileSync } from 'node:fs'

describe('desktop command library', () => {
  it('uses the official snapshot endpoint without embedding the website', () => {
    const service = readFileSync('src/services/tauri/support.ts', 'utf8')
    const view = readFileSync('src/views/CommandLibraryView.vue', 'utf8')
    expect(service).toContain('https://cs2as.600318.xyz/api/command-library')
    expect(service).toContain("invoke<CommandLibraryPayload>('get_command_library'")
    expect(service).toContain('isTauri()')
    expect(service).toContain("credentials: 'omit'")
    expect(view).toContain('指令研究所')
    expect(view).toContain('按首字符筛选')
    expect(view).toContain('复制命令')
    expect(view).not.toContain('<iframe')
  })
})
