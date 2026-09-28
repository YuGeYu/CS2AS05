import type { VaultResource } from '@/features/resource-vault/types'

export type PromotionKind = 'resource' | 'local'

export type PromotionPayload =
  | { kind: 'resource'; resource: VaultResource }
  | { kind: 'local'; localId: 'community' | 'donate' }

