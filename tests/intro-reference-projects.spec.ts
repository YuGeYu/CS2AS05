import { describe, expect, it } from 'vitest'
import { REFERENCE_PROJECT_GROUPS, REFERENCE_PROJECTS } from '@/features/support/reference-projects'

describe('intro reference projects', () => {
  it('shares all release upstream projects with about and sources', () => {
    expect(REFERENCE_PROJECTS.map(item => item.repository)).toEqual([
      'ed0ard/CS2-Bot-Improver',
      'XBribo/CS2-Bot-Vision',
      'unicbm/demotracer',
      'LaihoE/demoparser',
      'akiver/cs-demo-manager',
      'ianlucas/cs2-css-inventory-simulator',
      'mrdoob/three.js',
      'SteamDatabase/GameTracking-CS2',
      'DrEAmSs59/CS2-insight-agent',
    ])
    expect(REFERENCE_PROJECT_GROUPS.flatMap(group => group.projects).map(item => item.id).sort())
      .toEqual(REFERENCE_PROJECTS.map(item => item.id).sort())
  })
})
