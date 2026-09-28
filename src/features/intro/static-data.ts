import type { IntroData, SupporterAcknowledgement } from './types'
import { REFERENCE_PROJECTS } from '@/features/support/reference-projects'

// Sponsor identities are intentionally not bundled or displayed.
export const STATIC_SUPPORTERS: SupporterAcknowledgement[] = []
export const STATIC_INTRO_DATA: IntroData = {
  supporters: STATIC_SUPPORTERS,
  upstream: { fullName: 'ed0ard/CS2-Bot-Improver', description: 'CS2 Bot 行为增强项目，本助手基于其完整能力构建。', url: 'https://github.com/ed0ard/CS2-Bot-Improver', license: 'AGPL-3.0', stars: null, forks: null, pushedAt: null },
  fetchedAt: null,
  sources: { supporters: 'fallback', upstream: 'fallback' },
}
export const STATIC_REFERENCE_PROJECTS = REFERENCE_PROJECTS
