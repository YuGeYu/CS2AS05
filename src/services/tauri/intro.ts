import { invoke } from '@tauri-apps/api/core'

import type { SupporterAcknowledgement, UpstreamProjectSummary } from '@/features/intro/types'

export interface IntroNativePayload {
  supporters: SupporterAcknowledgement[]
  upstream: UpstreamProjectSummary
  sources: { supporters: 'network' | 'fallback'; upstream: 'network' | 'fallback' }
  diagnostics: { supporters: 'ok' | 'timeout' | 'network' | 'http' | 'invalid'; upstream: 'ok' | 'timeout' | 'network' | 'http' | 'invalid' }
}

export function getIntroPublicData() {
  return invoke<IntroNativePayload>('get_intro_public_data')
}
