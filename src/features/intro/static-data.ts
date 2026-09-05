import type { IntroData, SupporterAcknowledgement } from './types'
import { REFERENCE_PROJECTS } from '@/features/support/reference-projects'

export const STATIC_SUPPORTERS: SupporterAcknowledgement[] = [
  { id: '53e0cc57-6374-4b88-a9cf-aa3874dc4c72', nickname: '鲍里斯的眼睛是湖绿色', message: null, amountCents: 3000, sortOrder: 0, isVisible: true, createdAt: '2026-08-18T16:09:54.868Z', updatedAt: '2026-08-18T16:09:54.868Z' },
  { id: '4ceba221-220d-4742-a075-d391ff22682b', nickname: '超级烦他那', message: '加油，软件很便捷', amountCents: 2000, sortOrder: 0, isVisible: true, createdAt: '2026-08-05T02:07:12.956Z', updatedAt: '2026-08-05T02:07:12.956Z' },
  { id: '608ed49d-3db1-47a6-969b-a9d65396779f', nickname: '年华.', message: '加油', amountCents: 2000, sortOrder: 0, isVisible: true, createdAt: '2026-07-27T13:09:10.387Z', updatedAt: '2026-07-27T13:09:10.387Z' },
  { id: '25b51a9f-4489-4856-b79a-0939dab558df', nickname: '雪浪风尘', message: '好玩', amountCents: 15400, sortOrder: 0, isVisible: true, createdAt: '2026-07-20T17:01:42.353Z', updatedAt: '2026-07-27T13:22:25.912Z' },
  { id: '17f5de10-0ae6-48da-883b-cbb82147def5', nickname: '我走路带风', message: '使用不错，期待继续更新', amountCents: 500, sortOrder: 0, isVisible: true, createdAt: '2026-07-11T15:30:52.438Z', updatedAt: '2026-07-11T15:30:52.438Z' },
  { id: '7d59129c-ebee-43b6-8bf4-03eb7d53c6e2', nickname: null, message: '感谢您的付出和坚持', amountCents: 1000, sortOrder: 1, isVisible: true, createdAt: '2026-07-11T15:31:16.591Z', updatedAt: '2026-07-11T15:31:16.591Z' },
]
export const STATIC_INTRO_DATA: IntroData = {
  supporters: STATIC_SUPPORTERS,
  upstream: { fullName: 'ed0ard/CS2-Bot-Improver', description: 'CS2 Bot 行为增强项目，本助手基于其完整能力构建。', url: 'https://github.com/ed0ard/CS2-Bot-Improver', license: 'AGPL-3.0', stars: null, forks: null, pushedAt: null },
  fetchedAt: null,
  sources: { supporters: 'fallback', upstream: 'fallback' },
}
export const STATIC_REFERENCE_PROJECTS = REFERENCE_PROJECTS
