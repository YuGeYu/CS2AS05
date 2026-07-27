import { appConfig } from '@/config/app'
import type {
  SoftwareRelease,
  SoftwareReleaseDownload,
  SoftwareReleaseSelfUpdate,
  SoftwareUpdatePayload,
  SoftwareUpdateSeverity,
  UpdateCheckResult,
} from '@/features/software-updates/types'

const UPDATE_TIMEOUT_MS = 6_000
const SEVERITIES = new Set<SoftwareUpdateSeverity>(['normal', 'recommended', 'critical'])

export interface UpdateCheckOptions {
  manual?: boolean
  fetchImpl?: typeof fetch
}

export async function requestSoftwareUpdate(options: UpdateCheckOptions = {}): Promise<UpdateCheckResult> {
  const manual = options.manual ?? false
  if (!appConfig.updateFeedUrl || (!manual && !appConfig.updaterEnabled)) return { status: 'disabled' }

  let url: URL
  try {
    url = new URL(appConfig.updateFeedUrl)
  } catch {
    return { status: 'failed', message: '更新地址配置无效，请稍后重试。' }
  }
  url.searchParams.set('currentVersion', appConfig.appVersion)
  if (!url.searchParams.has('channel')) url.searchParams.set('channel', appConfig.channel)

  const controller = new AbortController()
  const timeout = window.setTimeout(() => controller.abort(), UPDATE_TIMEOUT_MS)
  try {
    const response = await (options.fetchImpl ?? fetch)(url, {
      cache: 'no-store',
      signal: controller.signal,
    })
    if (!response.ok) return { status: 'failed', message: `官网暂时无法响应更新检查（HTTP ${response.status}）。` }

    let body: unknown
    try {
      body = await response.json()
    } catch {
      return { status: 'failed', message: '官网返回了无法识别的更新信息。' }
    }

    const payload = parsePayload(body)
    if (!payload) return { status: 'failed', message: '官网更新信息格式不完整，请稍后重试。' }
    if (!payload.hasUpdate) return { status: 'current', payload }
    if (!payload.latest) return { status: 'failed', message: '官网提示有更新，但没有提供有效版本信息。' }
    return { status: 'available', payload, release: payload.latest }
  } catch (error) {
    const message = error instanceof DOMException && error.name === 'AbortError'
      ? '更新检查超时，请稍后重试。'
      : '暂时无法连接官网，请检查网络后重试。'
    return { status: 'failed', message }
  } finally {
    window.clearTimeout(timeout)
  }
}

function parsePayload(value: unknown): SoftwareUpdatePayload | null {
  if (!isRecord(value) || value.projectId !== appConfig.projectId) return null
  if (typeof value.channel !== 'string' || typeof value.currentVersion !== 'string' || typeof value.hasUpdate !== 'boolean') return null
  if (!Array.isArray(value.history)) return null

  const latest = value.latest === null ? null : parseRelease(value.latest)
  if (value.latest !== null && latest === null) return null
  const history = value.history.map(parseRelease).filter((item): item is SoftwareRelease => item !== null)
  return {
    projectId: value.projectId,
    channel: value.channel,
    currentVersion: value.currentVersion,
    hasUpdate: value.hasUpdate,
    latest,
    history,
  }
}

function parseRelease(value: unknown): SoftwareRelease | null {
  if (!isRecord(value)) return null
  const severity = value.severity
  if (typeof severity !== 'string' || !SEVERITIES.has(severity as SoftwareUpdateSeverity)) return null
  const download = parseDownload(value.download)
  const requiredStrings = ['id', 'projectId', 'channel', 'version', 'title', 'summary', 'publishedAt'] as const
  if (!requiredStrings.every((key) => typeof value[key] === 'string') || !download) return null
  if (typeof value.isCritical !== 'boolean' || typeof value.isActive !== 'boolean' || !Array.isArray(value.items)) return null

  return {
    id: value.id as string,
    projectId: value.projectId as string,
    channel: value.channel as string,
    version: value.version as string,
    title: value.title as string,
    summary: value.summary as string,
    items: value.items.slice(0, 40).filter((item): item is string => typeof item === 'string'),
    severity: severity as SoftwareUpdateSeverity,
    isCritical: value.isCritical,
    isActive: value.isActive,
    publishedAt: value.publishedAt as string,
    download,
    selfUpdate: parseSelfUpdate(value.selfUpdate),
  }
}

function parseSelfUpdate(value: unknown): SoftwareReleaseSelfUpdate {
  if (!isRecord(value)) return { available: false, reason: 'artifact_not_ready', target: 'windows', arch: 'x86_64', size: 0, sha256: '' }
  const reasons = new Set(['available', 'r2_disabled', 'artifact_not_ready', 'unsupported_platform'])
  const reason = typeof value.reason === 'string' && reasons.has(value.reason) ? value.reason as SoftwareReleaseSelfUpdate['reason'] : 'artifact_not_ready'
  return {
    available: value.available === true,
    reason,
    target: typeof value.target === 'string' ? value.target : 'windows',
    arch: typeof value.arch === 'string' ? value.arch : 'x86_64',
    size: typeof value.size === 'number' && Number.isFinite(value.size) ? Math.max(0, value.size) : 0,
    sha256: typeof value.sha256 === 'string' ? value.sha256 : '',
  }
}

function parseDownload(value: unknown): SoftwareReleaseDownload | null {
  if (!isRecord(value)) return null
  if (!['type', 'label', 'url', 'code'].every((key) => typeof value[key] === 'string')) return null
  return { type: value.type as string, label: value.label as string, url: value.url as string, code: value.code as string }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}
