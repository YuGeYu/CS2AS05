export interface StickerInfo {
  id: number;
  offsetX?: number;
  offsetY?: number;
  wear?: number;
  scale?: number;
  rotation?: number;
}

export interface KeychainInfo {
  id: number;
  offsetX?: number;
  offsetY?: number;
  offsetZ?: number;
  seed?: number;
}

export interface StatTrakInfo {
  enabled: boolean;
  count: number;
}

export type Team = 'ct' | 't';

export interface Loadout {
  // Legacy shared weapon maps (kept for plugin backward compat; mirror CT values)
  weaponPaints: Record<number, number>;
  weaponWears: Record<number, number>;
  weaponSeeds: Record<number, number>;
  // Per-team weapon skins (v1.6.0+)
  weaponPaintsCt: Record<number, number>;
  weaponWearsCt: Record<number, number>;
  weaponSeedsCt: Record<number, number>;
  weaponPaintsT: Record<number, number>;
  weaponWearsT: Record<number, number>;
  weaponSeedsT: Record<number, number>;
  weaponStickers: Record<number, StickerInfo[]>;
  weaponKeychains: Record<number, KeychainInfo>;
  weaponNametags: Record<number, string>;
  weaponStatTrak: Record<number, StatTrakInfo>;
  // Legacy shared knife fields (mirror CT values)
  knifeIndex: number;
  knifePaint: number;
  knifeWear: number;
  knifeSeed: number;
  // Per-team knives (v1.6.0+)
  knifeIndexCt: number;
  knifePaintCt: number;
  knifeWearCt: number;
  knifeSeedCt: number;
  knifeIndexT: number;
  knifePaintT: number;
  knifeWearT: number;
  knifeSeedT: number;
  gloveIndexCt: number;
  glovePaintCt: number;
  gloveWearCt: number;
  gloveSeedCt: number;
  gloveDefIndexCt: number;
  gloveIndexT: number;
  glovePaintT: number;
  gloveWearT: number;
  gloveSeedT: number;
  gloveDefIndexT: number;
  agentModelCt: number;
  agentModelT: number;
  agentModelPathCt: string;
  agentModelPathT: string;
  musicKit: number;
  useRandom: boolean;
}
