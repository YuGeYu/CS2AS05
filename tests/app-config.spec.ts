import { describe, expect, it } from 'vitest'

import { appConfig } from '@/config/app'

describe('release app configuration', () => {
  it('keeps production update checks enabled without ignored env files', () => {
    expect(appConfig).toMatchObject({
      channel: 'prod',
      updateFeedUrl: 'https://cs2as.600318.xyz/api/software-updates/cs2-bot-improver',
      updaterEnabled: true,
      projectId: 'cs2-bot-improver',
    })
  })
})
