export interface SupporterAcknowledgement {
  id: string
  nickname: string | null
  message: string | null
  amountCents: number
  sortOrder: number
  isVisible: boolean
  createdAt: string
  updatedAt: string
}

export interface UpstreamProjectSummary {
  fullName: string
  description: string
  url: string
  license: string
  stars: number | null
  forks: number | null
  pushedAt: string | null
}

export interface IntroData {
  supporters: SupporterAcknowledgement[]
  upstream: UpstreamProjectSummary
  fetchedAt: string | null
  sources: {
    supporters: 'network' | 'cache' | 'fallback'
    upstream: 'network' | 'cache' | 'fallback'
  }
}

export interface IntroAcknowledgementCard {
  id: string
  kind: 'supporter' | 'upstream'
  eyebrow: string
  title: string
  message: string
  detail: string
  updatedAt: string
}
