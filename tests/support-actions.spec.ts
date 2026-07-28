import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const mocks = vi.hoisted(() => ({
  check: vi.fn(), official: vi.fn(), idea: vi.fn(), releasePage: vi.fn(), download: vi.fn(), upstream: vi.fn(),
}))
vi.mock('@/features/software-updates/state', () => ({
  checkForSoftwareUpdates: mocks.check,
  dismissRelease: vi.fn(),
  shouldPresentRelease: vi.fn(() => true),
}))
vi.mock('@/services/tauri/support', () => ({
  openOfficialSite: mocks.official,
  openIdeaPage: mocks.idea,
  openReleasePage: mocks.releasePage,
  openUpdateDownload: mocks.download,
  openUpstreamProject: mocks.upstream,
}))

import SupportActions from '@/components/SupportActions.vue'

describe('support actions', () => {
  beforeEach(() => {
    Object.values(mocks).forEach((mock) => mock.mockReset())
    mocks.check.mockResolvedValue({ status: 'current', payload: {} })
    mocks.official.mockResolvedValue(undefined)
    mocks.idea.mockResolvedValue(undefined)
  })

  it('maps update, official-site, and idea buttons to their commands', async () => {
    const wrapper = mount(SupportActions, { global: { stubs: { Teleport: true } } })
    await flushPromises()
    expect(mocks.check).not.toHaveBeenCalled()
    expect(wrapper.text()).toContain('启动后自动检查更新')
    const buttons = wrapper.findAll('.support-actions button')
    await buttons[0]?.trigger('click')
    await buttons[1]?.trigger('click')
    await buttons[2]?.trigger('click')
    await flushPromises()
    expect(mocks.check).toHaveBeenLastCalledWith(true)
    expect(mocks.official).toHaveBeenCalledOnce()
    expect(mocks.idea).toHaveBeenCalledOnce()
  })

  it('disables duplicate checks and exposes a keyboard-accessible about entry', async () => {
    let resolve!: (value: unknown) => void
    mocks.check.mockReturnValue(new Promise((done) => { resolve = done }))
    const wrapper = mount(SupportActions, { global: { stubs: { Teleport: true } } })
    const checkButton = wrapper.findAll('.support-actions button')[0]
    await checkButton?.trigger('click')
    expect(checkButton?.attributes('disabled')).toBeDefined()
    expect(wrapper.get('.about-trigger').element.tagName).toBe('BUTTON')
    resolve({ status: 'current', payload: {} })
    await flushPromises()
  })

  it('shows a recoverable failure message', async () => {
    mocks.check.mockResolvedValue({ status: 'failed', message: '暂时无法连接官网' })
    const wrapper = mount(SupportActions, { global: { stubs: { Teleport: true } } })
    await wrapper.findAll('.support-actions button')[0]?.trigger('click')
    await flushPromises()
    expect(wrapper.text()).toContain('暂时无法连接官网')
    expect(wrapper.text()).toContain('打开官网')
    expect(wrapper.text()).toContain('查看/编辑意见')
  })
})
