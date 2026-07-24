export type SoftwareUpdateSeverity = 'normal' | 'recommended' | 'critical'

export interface SoftwareReleaseDownload {
  type: string
  label: string
  url: string
  code: string
}

export interface SoftwareRelease {
  id: string
  projectId: string
  channel: string
  version: string
  title: string
  summary: string
  items: string[]
  severity: SoftwareUpdateSeverity
  isCritical: boolean
  isActive: boolean
  publishedAt: string
  download: SoftwareReleaseDownload
}

export interface SoftwareUpdatePayload {
  projectId: string
  channel: string
  currentVersion: string
  hasUpdate: boolean
  latest: SoftwareRelease | null
  history: SoftwareRelease[]
}

export type UpdateCheckResult =
  | { status: 'disabled' }
  | { status: 'current'; payload: SoftwareUpdatePayload }
  | { status: 'available'; payload: SoftwareUpdatePayload; release: SoftwareRelease }
  | { status: 'failed'; message: string }
