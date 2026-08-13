import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ cache: vi.fn(), convert: vi.fn((path: string) => `asset://${path}`) }))
vi.mock('@/services/tauri/skinForge', () => ({ skinForgeCacheImage: mocks.cache }))
vi.mock('@tauri-apps/api/core', () => ({ convertFileSrc: mocks.convert }))

import { forgeImageQueueLimits, loadForgeImage, resetForgeImageQueueForTests } from '@/features/skin-forge/image-cache'

describe('skin forge image cache queue', () => {
  beforeEach(() => {
    vi.useRealTimers()
    resetForgeImageQueueForTests()
    mocks.cache.mockReset()
    mocks.convert.mockClear()
  })

  it('deduplicates URLs and limits 20 requests to four active invokes', async () => {
    let active = 0
    let maximum = 0
    mocks.cache.mockImplementation((url: string) => new Promise(resolve => {
      active += 1
      maximum = Math.max(maximum, active)
      setTimeout(() => { active -= 1; resolve({ path: `C:/cache/${encodeURIComponent(url)}.png` }) }, 2)
    }))
    const duplicate = loadForgeImage('https://image.test/same')
    expect(loadForgeImage('https://image.test/same')).toBe(duplicate)
    const requests = [duplicate, ...Array.from({ length: 19 }, (_, index) => loadForgeImage(`https://image.test/${index}`))]
    await expect(Promise.all(requests)).resolves.toHaveLength(20)
    expect(maximum).toBe(forgeImageQueueLimits.maxConcurrent)
    expect(mocks.cache).toHaveBeenCalledTimes(20)
  })

  it('retries IMAGE_BUSY twice and converts only the returned cache path', async () => {
    vi.useFakeTimers()
    mocks.cache
      .mockRejectedValueOnce('[IMAGE_BUSY] first')
      .mockRejectedValueOnce(new Error('[IMAGE_BUSY] second'))
      .mockResolvedValueOnce({ path: 'C:/cache/final.webp' })
    const result = loadForgeImage('https://image.test/retry')
    await vi.runAllTimersAsync()
    await expect(result).resolves.toBe('asset://C:/cache/final.webp')
    expect(mocks.cache).toHaveBeenCalledTimes(3)
    expect(mocks.convert).toHaveBeenCalledExactlyOnceWith('C:/cache/final.webp')
  })

  it('does not retry non-busy failures or retry forever', async () => {
    mocks.cache.mockRejectedValueOnce('[IMAGE_MIME] unsupported')
    await expect(loadForgeImage('https://image.test/mime')).rejects.toBe('[IMAGE_MIME] unsupported')
    expect(mocks.cache).toHaveBeenCalledOnce()

    vi.useFakeTimers()
    mocks.cache.mockReset().mockRejectedValue('[IMAGE_BUSY] full')
    const exhausted = loadForgeImage('https://image.test/full').catch(error => error)
    await vi.runAllTimersAsync()
    await expect(exhausted).resolves.toBe('[IMAGE_BUSY] full')
    expect(mocks.cache).toHaveBeenCalledTimes(3)
  })
})
