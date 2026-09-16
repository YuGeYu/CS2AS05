import type { VaultResource } from '@/features/resource-vault/types'
const ENDPOINT = 'https://vault.600318.xyz/api/integrations/cs2as/resources'
let cache: { at: number; resources: VaultResource[] } | null = null
let inFlight: Promise<VaultResource[]> | null = null
export async function loadVaultResources(force = false): Promise<VaultResource[]> {
  if (!force && cache && Date.now() - cache.at < 60_000) return cache.resources
  if (inFlight) return inFlight
  inFlight = fetch(`${ENDPOINT}?limit=120`, { headers: { accept: 'application/json' }, cache: 'no-store' }).then(async response => {
    if (!response.ok) throw new Error(`资源网站暂时不可用（${response.status}）`)
    const payload = await response.json() as { resources?: unknown }
    if (!Array.isArray(payload.resources)) throw new Error('资源列表格式无效')
    const rows = payload.resources.filter((item): item is VaultResource => Boolean(item && typeof item === 'object' && typeof (item as VaultResource).id === 'string'))
    cache = { at: Date.now(), resources: rows }; return rows
  }).finally(() => { inFlight = null })
  return inFlight
}
