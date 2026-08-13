import type { ReferenceProjectId } from '@/services/tauri/support'

export interface ReferenceProject {
  id: ReferenceProjectId
  repository: string
  description: string
  license?: string
  group: '核心上游' | 'Demo 参考' | '皮肤参考'
}

export const REFERENCE_PROJECTS: ReferenceProject[] = [
  { id: 'bot-improver', repository: 'ed0ard/CS2-Bot-Improver', description: '助手基础功能与 BOT 增强能力的核心上游。', license: 'AGPL-3.0 或更高版本', group: '核心上游' },
  { id: 'demotracer', repository: 'unicbm/demotracer', description: 'Demo 事件追踪与数据处理的参考实现。', group: 'Demo 参考' },
  { id: 'demoparser', repository: 'LaihoE/demoparser', description: 'Demo 解析流程与数据结构的参考项目。', group: 'Demo 参考' },
  { id: 'cs-demo-manager', repository: 'akiver/cs-demo-manager', description: 'Demo 管理与复盘体验的参考项目。', group: 'Demo 参考' },
  { id: 'skin-forge', repository: 'kaecho/CS2-Skin-Forge', description: 'PlayerSkinMod 装备目录与工坊交互的参考项目。', group: '皮肤参考' },
]

export const REFERENCE_PROJECT_GROUPS = ['核心上游', 'Demo 参考', '皮肤参考'].map(title => ({
  title,
  projects: REFERENCE_PROJECTS.filter(project => project.group === title),
}))
