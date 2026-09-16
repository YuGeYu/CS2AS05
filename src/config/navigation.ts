import { Bot, Boxes, ChartNoAxesCombined, Gauge, Headphones, ScrollText, Settings, Sword, WalletCards, LibraryBig, MessagesSquare } from 'lucide-vue-next'

export type ViewKey = 'overview' | 'presets' | 'items' | 'knives' | 'inventory' | 'commands' | 'demoReview' | 'quickSupport' | 'resourceVault' | 'community' | 'install'
export type NavigationGroup = 'core' | 'tools' | 'review' | 'support'

export const NAV_ITEMS = [
  { key: 'overview', label: '概览', icon: Gauge, required: false, group: 'core' },
  { key: 'presets', label: '人机预设', icon: Bot, required: false, group: 'core' },
  { key: 'items', label: 'Bot 物品', icon: Boxes, required: false, group: 'tools' },
  { key: 'knives', label: '刀具', icon: Sword, required: false, group: 'tools' },
  { key: 'commands', label: '命令', icon: ScrollText, required: false, group: 'tools' },
  { key: 'inventory', label: '库存换肤', icon: WalletCards, required: false, group: 'review' },
  { key: 'demoReview', label: '对局复盘', icon: ChartNoAxesCombined, required: false, group: 'review' },
  { key: 'quickSupport', label: '快快客服', icon: Headphones, required: false, group: 'support' },
  { key: 'resourceVault', label: '资源阁', icon: LibraryBig, required: false, group: 'support' },
  { key: 'community', label: '玩家圈子', icon: MessagesSquare, required: false, group: 'support' },
  { key: 'install', label: '安装与诊断', icon: Settings, required: true, group: 'support' },
] as const

export const NAVIGATION_GROUPS = [
  { key: 'core', label: '核心工作台' },
  { key: 'tools', label: 'BOT 工具' },
  { key: 'review', label: '复盘与扩展' },
  { key: 'support', label: '支持与维护' },
] as const

export const NAV_ITEM_KEYS = NAV_ITEMS.map(item => item.key) as ViewKey[]
