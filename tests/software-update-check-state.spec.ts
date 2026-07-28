import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => ({ request: vi.fn() }))
vi.mock('@/services/software-updates', () => ({ requestSoftwareUpdate: mocks.request }))

import { checkForSoftwareUpdates, resetSoftwareUpdateStateForTests } from '@/features/software-updates/state'
import { resetSoftwareUpdateCoordinatorForTests, startSoftwareUpdateCoordinator } from '@/features/software-updates/coordinator'

describe('software update check request ownership', () => {
  beforeEach(() => {
    mocks.request.mockReset()
    resetSoftwareUpdateStateForTests()
    resetSoftwareUpdateCoordinatorForTests()
  })

  it('shares one network request across rapid remount checks', async () => {
    let resolve!: (value: { status: 'current', payload: object }) => void
    mocks.request.mockReturnValue(new Promise(done => { resolve = done }))
    const first = checkForSoftwareUpdates(false)
    const second = checkForSoftwareUpdates(false)
    expect(first).toBe(second)
    expect(mocks.request).toHaveBeenCalledOnce()
    resolve({ status: 'current', payload: {} })
    await expect(first).resolves.toMatchObject({ status: 'current' })
  })

  it('owns exactly one automatic cold-start check', async () => {
    mocks.request.mockResolvedValue({ status: 'current', payload: {} })
    const first = startSoftwareUpdateCoordinator()
    const second = startSoftwareUpdateCoordinator()
    expect(first).toBe(second)
    await first
    expect(mocks.request).toHaveBeenCalledExactlyOnceWith({ manual: false })
  })
})
