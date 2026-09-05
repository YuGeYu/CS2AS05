import { Bot, Boxes, ChartNoAxesCombined, Gauge, Headphones, ScrollText, Settings, Sword, WalletCards } from 'lucide-vue-next'

export type ViewKey = 'overview' | 'presets' | 'items' | 'knives' | 'inventory' | 'commands' | 'demoReview' | 'quickSupport' | 'install'

export const NAV_ITEMS = [
  { key: 'overview', label: '概览', icon: Gauge, required: false },
  { key: 'presets', label: '人机预设', icon: Bot, required: false },
  { key: 'items', label: 'Bot 物品', icon: Boxes, required: false },
  { key: 'knives', label: '刀具', icon: Sword, required: false },
  { key: 'inventory', label: '库存换肤', icon: WalletCards, required: false },
  { key: 'commands', label: '命令', icon: ScrollText, required: false },
  { key: 'demoReview', label: '对局复盘', icon: ChartNoAxesCombined, required: false },
  { key: 'quickSupport', label: '快快客服', icon: Headphones, required: false },
  { key: 'install', label: '安装与诊断', icon: Settings, required: true },
] as const

export const NAV_ITEM_KEYS = NAV_ITEMS.map(item => item.key) as ViewKey[]
