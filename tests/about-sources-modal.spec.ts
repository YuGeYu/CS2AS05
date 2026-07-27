import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const openUpstream = vi.hoisted(() => vi.fn())
vi.mock('@/services/tauri/support', () => ({ openUpstreamProject: openUpstream }))

import AboutSourcesModal from '@/components/AboutSourcesModal.vue'

describe('about sources modal', () => {
  beforeEach(() => openUpstream.mockReset().mockResolvedValue(undefined))

  it('requires the second layer before exposing the upstream action and resets on reopen', async () => {
    const wrapper = mount(AboutSourcesModal, { props: { open: true }, global: { stubs: { Teleport: true } } })
    await flushPromises()
    expect(wrapper.text()).toContain('关于 CS2 人机增强助手')
    expect(wrapper.text()).not.toContain('ed0ard/CS2-Bot-Improver')

    await wrapper.get('.source-link').trigger('click')
    expect(wrapper.text()).toContain('ed0ard/CS2-Bot-Improver')
    expect(wrapper.text()).toContain('v1.4.3')
    expect(wrapper.text()).toContain('GNU Affero General Public License')
    await wrapper.get('.source-link').trigger('click')
    expect(openUpstream).toHaveBeenCalledOnce()

    await wrapper.get('[aria-label="返回关于页面"]').trigger('click')
    expect(wrapper.text()).toContain('关于 CS2 人机增强助手')
    await wrapper.setProps({ open: false })
    await wrapper.setProps({ open: true })
    await flushPromises()
    expect(wrapper.text()).not.toContain('ed0ard/CS2-Bot-Improver')
  })
})
