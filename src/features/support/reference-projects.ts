import type { ReferenceProjectId } from '@/services/tauri/support'

export interface ReferenceProject {
  id: ReferenceProjectId
  repository: string
  description: string
  license?: string
  group: '核心上游' | 'Demo 参考' | '饰品能力'
}

export const REFERENCE_PROJECTS: ReferenceProject[] = [
  { id: 'bot-improver', repository: 'ed0ard/CS2-Bot-Improver', description: '助手基础功能与 BOT 增强能力的核心上游。', license: 'AGPL-3.0 或更高版本', group: '核心上游' },
  { id: 'botvision', repository: 'XBribo/CS2-Bot-Vision', description: '本地 BOT 视觉组件的上游来源，随资源包并行提供。', license: 'AGPL-3.0', group: '核心上游' },
  { id: 'demotracer', repository: 'unicbm/demotracer', description: 'Demo 事件追踪与数据处理的参考实现。', group: 'Demo 参考' },
  { id: 'demoparser', repository: 'LaihoE/demoparser', description: 'Demo 解析流程与数据结构的参考项目。', group: 'Demo 参考' },
  { id: 'cs-demo-manager', repository: 'akiver/cs-demo-manager', description: 'Demo 管理与复盘体验的参考项目。', group: 'Demo 参考' },
  { id: 'inventory-simulator', repository: 'ianlucas/cs2-css-inventory-simulator', description: '按 SteamID 提供完整 CS2 库存模拟能力。', group: '饰品能力' },
  { id: 'threejs', repository: 'mrdoob/three.js', description: '开屏 WebGL 场景使用的渲染库。', license: 'MIT', group: '核心上游' },
  { id: 'gametracking-cs2', repository: 'SteamDatabase/GameTracking-CS2', description: 'Demo 协议和 CS2 数据结构更新的参考来源。', group: 'Demo 参考' },
  { id: 'insight-agent', repository: 'DrEAmSs59/CS2-insight-agent', description: 'Demo 播放与文件交互语义的行为参考。', license: 'PolyForm Noncommercial 1.0.0', group: 'Demo 参考' },
]

export const REFERENCE_PROJECT_GROUPS = ['核心上游', 'Demo 参考', '饰品能力'].map(title => ({
  title,
  projects: REFERENCE_PROJECTS.filter(project => project.group === title),
}))
