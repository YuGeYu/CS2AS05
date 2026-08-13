export interface Knife {
  defindex: number;
  name: string;
  nameZh: string;
  codename: string;
}

import { getWeaponDefaultImage } from './weaponImages';

export function getKnifeImageUrl(defindex: number): string {
  return getWeaponDefaultImage(defindex);
}

export const knives: Knife[] = [
  { defindex: 500, name: 'Bayonet', nameZh: '刺刀', codename: 'bayonet' },
  { defindex: 503, name: 'Classic Knife', nameZh: '经典匕首', codename: 'css' },
  { defindex: 505, name: 'Flip Knife', nameZh: '折叠刀', codename: 'flip' },
  { defindex: 506, name: 'Gut Knife', nameZh: '穿肠刀', codename: 'gut' },
  { defindex: 507, name: 'Karambit', nameZh: '爪子刀', codename: 'karambit' },
  { defindex: 508, name: 'M9 Bayonet', nameZh: 'M9刺刀', codename: 'm9_bayonet' },
  { defindex: 509, name: 'Huntsman Knife', nameZh: '猎杀者匕首', codename: 'tactical' },
  { defindex: 512, name: 'Falchion Knife', nameZh: '弯刀', codename: 'falchion' },
  { defindex: 514, name: 'Bowie Knife', nameZh: '鲍伊猎刀', codename: 'survival_bowie' },
  { defindex: 515, name: 'Butterfly Knife', nameZh: '蝴蝶刀', codename: 'butterfly' },
  { defindex: 516, name: 'Shadow Daggers', nameZh: '暗影双匕', codename: 'push' },
  { defindex: 517, name: 'Paracord Knife', nameZh: '系绳刀', codename: 'cord' },
  { defindex: 518, name: 'Survival Knife', nameZh: '求生刀', codename: 'canis' },
  { defindex: 519, name: 'Ursus Knife', nameZh: '熊刀', codename: 'ursus' },
  { defindex: 520, name: 'Navaja Knife', nameZh: '折刀', codename: 'gypsy_jackknife' },
  { defindex: 521, name: 'Nomad Knife', nameZh: '流浪者匕首', codename: 'outdoor' },
  { defindex: 522, name: 'Stiletto Knife', nameZh: '短剑', codename: 'stiletto' },
  { defindex: 523, name: 'Talon Knife', nameZh: '海豹短刀', codename: 'widowmaker' },
  { defindex: 525, name: 'Skeleton Knife', nameZh: '骷髅匕首', codename: 'skeleton' },
  { defindex: 526, name: 'Kukri Knife', nameZh: '廓尔喀弯刀', codename: 'kukri' },
];
