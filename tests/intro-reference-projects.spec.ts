import { describe, expect, it } from 'vitest'
import { REFERENCE_PROJECT_GROUPS, REFERENCE_PROJECTS } from '@/features/support/reference-projects'

describe('intro reference projects', () => {
  it('shares the five upstream projects with about and sources', () => {
    expect(REFERENCE_PROJECTS.map(item => item.repository)).toEqual([
      'ed0ard/CS2-Bot-Improver',
      'unicbm/demotracer',
      'LaihoE/demoparser',
      'akiver/cs-demo-manager',
      'kaecho/CS2-Skin-Forge',
    ])
    expect(REFERENCE_PROJECT_GROUPS.flatMap(group => group.projects)).toEqual(REFERENCE_PROJECTS)
  })
})
