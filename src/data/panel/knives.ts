import knife500 from '@/assets/knives/500.png'
import knife503 from '@/assets/knives/503.png'
import knife505 from '@/assets/knives/505.png'
import knife506 from '@/assets/knives/506.png'
import knife507 from '@/assets/knives/507.png'
import knife508 from '@/assets/knives/508.png'
import knife509 from '@/assets/knives/509.png'
import knife512 from '@/assets/knives/512.png'
import knife514 from '@/assets/knives/514.png'
import knife515 from '@/assets/knives/515.png'
import knife516 from '@/assets/knives/516.png'
import knife517 from '@/assets/knives/517.png'
import knife518 from '@/assets/knives/518.png'
import knife519 from '@/assets/knives/519.png'
import knife520 from '@/assets/knives/520.png'
import knife521 from '@/assets/knives/521.png'
import knife522 from '@/assets/knives/522.png'
import knife523 from '@/assets/knives/523.png'
import knife525 from '@/assets/knives/525.png'
import knife526 from '@/assets/knives/526.png'

export interface Knife {
  id: number
  name: string
  image: string
}

export const KNIVES: readonly Knife[] = [
  { id: 500, name: '刺刀', image: knife500 },
  { id: 503, name: '经典匕首', image: knife503 },
  { id: 505, name: '折叠刀', image: knife505 },
  { id: 506, name: '穿肠刀', image: knife506 },
  { id: 507, name: '爪子刀', image: knife507 },
  { id: 508, name: 'M9 刺刀', image: knife508 },
  { id: 509, name: '猎杀者匕首', image: knife509 },
  { id: 512, name: '弯刀', image: knife512 },
  { id: 514, name: '鲍伊猎刀', image: knife514 },
  { id: 515, name: '蝴蝶刀', image: knife515 },
  { id: 516, name: '暗影双匕', image: knife516 },
  { id: 517, name: '系绳匕首', image: knife517 },
  { id: 518, name: '求生匕首', image: knife518 },
  { id: 519, name: '熊刀', image: knife519 },
  { id: 520, name: '折刀', image: knife520 },
  { id: 521, name: '流浪者匕首', image: knife521 },
  { id: 522, name: '短剑', image: knife522 },
  { id: 523, name: '锯齿爪刀', image: knife523 },
  { id: 525, name: '骷髅匕首', image: knife525 },
  { id: 526, name: '廓尔喀刀', image: knife526 },
]
