import { beforeEach, describe, expect, it, vi } from 'vitest'

const { config } = vi.hoisted(() => ({
  config: {
    appVersion: '0.5.3',
    channel: 'prod',
    updateFeedUrl: 'https://cs2as.600318.xyz/api/software-updates/cs2-bot-improver',
    updaterEnabled: true,
    projectId: 'cs2-bot-improver',
  },
}))
vi.mock('@/config/app', () => ({ appConfig: config }))

import { checkForSoftwareUpdates, resetSoftwareUpdateStateForTests } from '@/features/software-updates/state'
import { requestSoftwareUpdate } from '@/services/software-updates'

const release = {
  id: 'release-1', projectId: 'cs2-bot-improver', channel: 'prod', version: '0.6.0',
  title: '更新', summary: '摘要', items: ['A'], severity: 'normal', isCritical: false,
  isActive: true, publishedAt: '2026-07-21T00:00:00Z',
  download: { type: 'quark', label: '下载', url: 'https://pan.quark.cn/s/abc', code: '1234' },
  selfUpdate: { available: true, reason: 'available', target: 'windows', arch: 'x86_64', size: 123, sha256: 'A'.repeat(64) },
} as const

function response(body: unknown, status = 200) {
  return new Response(JSON.stringify(body), { status, headers: { 'Content-Type': 'application/json' } })
}

function payload(hasUpdate: boolean, latest: unknown = release) {
  return { projectId: 'cs2-bot-improver', channel: 'prod', currentVersion: '0.5.3', hasUpdate, latest, history: [] }
}

describe('software update checks', () => {
  beforeEach(() => {
    config.updaterEnabled = true
    resetSoftwareUpdateStateForTests()
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  it('uses the real version and channel and trusts hasUpdate=false', async () => {
    const fetchMock = vi.fn(async (input: URL | RequestInfo) => {
      const url = new URL(String(input))
      expect(url.searchParams.get('currentVersion')).toBe('0.5.3')
      expect(url.searchParams.get('channel')).toBe('prod')
      return response(payload(false, { ...release, version: '0.5.0' }))
    })
    const result = await requestSoftwareUpdate({ fetchImpl: fetchMock as typeof fetch })
    expect(result.status).toBe('current')
  })

  it('returns available only when hasUpdate=true has a valid latest release', async () => {
    const available = await requestSoftwareUpdate({ fetchImpl: vi.fn(async () => response(payload(true))) as typeof fetch })
    const missing = await requestSoftwareUpdate({ fetchImpl: vi.fn(async () => response(payload(true, null))) as typeof fetch })
    expect(available.status).toBe('available')
    expect(missing).toMatchObject({ status: 'failed' })
  })

  it.each([
    ['HTTP error', vi.fn(async () => response({}, 503))],
    ['invalid JSON', vi.fn(async () => new Response('{', { status: 200 }))],
    ['network error', vi.fn(async () => { throw new TypeError('offline') })],
  ])('normalizes %s into a recoverable failure', async (_label, fetchMock) => {
    expect(await requestSoftwareUpdate({ fetchImpl: fetchMock as typeof fetch })).toMatchObject({ status: 'failed' })
  })

  it('aborts a slow request after six seconds', async () => {
    vi.useFakeTimers()
    const fetchMock = vi.fn((_input: URL | RequestInfo, init?: RequestInit) => new Promise<Response>((_resolve, reject) => {
      init?.signal?.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')))
    }))
    const pending = requestSoftwareUpdate({ fetchImpl: fetchMock as typeof fetch })
    await vi.advanceTimersByTimeAsync(6_000)
    await expect(pending).resolves.toMatchObject({ status: 'failed', message: expect.stringContaining('超时') })
  })

  it('disables automatic checks but permits manual checks', async () => {
    config.updaterEnabled = false
    const fetchMock = vi.fn(async () => response(payload(false)))
    expect(await requestSoftwareUpdate({ fetchImpl: fetchMock as typeof fetch })).toEqual({ status: 'disabled' })
    expect(fetchMock).not.toHaveBeenCalled()
    expect((await requestSoftwareUpdate({ manual: true, fetchImpl: fetchMock as typeof fetch })).status).toBe('current')
  })

  it('shares an in-flight request and permits another after completion', async () => {
    let resolveFetch!: (value: Response) => void
    globalThis.fetch = vi.fn()
      .mockImplementationOnce(() => new Promise<Response>((resolve) => { resolveFetch = resolve }))
      .mockResolvedValue(response(payload(false))) as typeof fetch
    const first = checkForSoftwareUpdates(true)
    const second = checkForSoftwareUpdates(true)
    expect(first).toBe(second)
    expect(fetch).toHaveBeenCalledOnce()
    resolveFetch(response(payload(false)))
    await first
    await checkForSoftwareUpdates(true)
    expect(fetch).toHaveBeenCalledTimes(2)
  })
})
