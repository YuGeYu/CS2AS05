// @vitest-environment jsdom

import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({
  check: vi.fn(),
  relaunch: vi.fn(),
  releaseCheck: vi.fn(),
}))

vi.mock('@/services/tauri/software-updater', () => ({
  checkTauriUpdater: mocks.check,
  relaunchAfterUpdate: mocks.relaunch,
}))
vi.mock('@/services/software-updates', () => ({ requestSoftwareUpdate: mocks.releaseCheck }))

import {
  deferDownloadedSoftwareUpdate,
  downloadSoftwareUpdate,
  initializeSoftwareUpdaterState,
  installDownloadedSoftwareUpdate,
  resetSoftwareUpdaterStateForTests,
  softwareUpdaterState,
} from '@/features/software-updates/updater-state'

function createResource(version = '0.5.4') {
  return {
    version,
    download: vi.fn(async (listener: (event: unknown) => void) => {
      listener({ event: 'Started', data: { contentLength: 100 } })
      listener({ event: 'Progress', data: { chunkLength: 40 } })
      listener({ event: 'Progress', data: { chunkLength: 60 } })
      listener({ event: 'Finished' })
    }),
    install: vi.fn(async () => undefined),
    close: vi.fn(async () => undefined),
  }
}

describe('application-level software updater state', () => {
  beforeEach(async () => {
    const values = new Map<string, string>()
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: {
        getItem: (key: string) => values.get(key) ?? null,
        setItem: (key: string, value: string) => values.set(key, value),
        removeItem: (key: string) => values.delete(key),
        clear: () => values.clear(),
      },
    })
    vi.clearAllMocks()
    await resetSoftwareUpdaterStateForTests()
  })

  it('downloads with real byte events and waits for installation confirmation', async () => {
    const resource = createResource()
    mocks.check.mockResolvedValue(resource)
    await downloadSoftwareUpdate('0.5.4')
    expect(softwareUpdaterState.phase).toBe('downloaded')
    expect(softwareUpdaterState.downloadedBytes).toBe(100)
    expect(resource.install).not.toHaveBeenCalled()
  })

  it('retains the verified resource for same-session deferred installation', async () => {
    const resource = createResource()
    mocks.check.mockResolvedValue(resource)
    mocks.releaseCheck.mockResolvedValue({ status: 'available', release: { version: '0.5.4', isActive: true } })
    await downloadSoftwareUpdate('0.5.4')
    deferDownloadedSoftwareUpdate()
    expect(softwareUpdaterState.phase).toBe('deferred-current-session')
    await installDownloadedSoftwareUpdate()
    expect(resource.download).toHaveBeenCalledOnce()
    expect(resource.install).toHaveBeenCalledOnce()
    expect(mocks.relaunch).toHaveBeenCalledOnce()
  })

  it('restores only a reminder after process restart', async () => {
    window.localStorage.setItem('cs2-bot-improver.deferred-update.v1', JSON.stringify({ version: '0.5.4', needsDownload: true }))
    initializeSoftwareUpdaterState()
    expect(softwareUpdaterState.phase).toBe('deferred-reminder-only')
    expect(softwareUpdaterState.reminderNeedsDownload).toBe(true)
  })

  it('rejects a downloaded resource when the release is no longer authoritative', async () => {
    const resource = createResource()
    mocks.check.mockResolvedValue(resource)
    mocks.releaseCheck.mockResolvedValue({ status: 'available', release: { version: '0.5.5', isActive: true } })
    await downloadSoftwareUpdate('0.5.4')
    await expect(installDownloadedSoftwareUpdate()).rejects.toThrow('不再是最新版')
    expect(resource.install).not.toHaveBeenCalled()
    expect(resource.close).toHaveBeenCalledOnce()
  })
})
