import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

const openReference = vi.hoisted(() => vi.fn())
vi.mock('@/services/tauri/support', () => ({ openReferenceProject: openReference }))

import AboutSourcesModal from '@/components/AboutSourcesModal.vue'

describe('about sources modal', () => {
  beforeEach(() => openReference.mockReset().mockResolvedValue(undefined))

  it('requires the second layer before exposing the upstream action and resets on reopen', async () => {
    const wrapper = mount(AboutSourcesModal, { props: { open: true }, global: { stubs: { Teleport: true } } })
    await flushPromises()
    expect(wrapper.text()).toContain('关于 CS2 人机增强助手')
    expect(wrapper.text()).not.toContain('ed0ard/CS2-Bot-Improver')

    await wrapper.get('.source-link').trigger('click')
    expect(wrapper.text()).toContain('ed0ard/CS2-Bot-Improver')
    expect(wrapper.text()).toContain('unicbm/demotracer')
    expect(wrapper.text()).toContain('LaihoE/demoparser')
    expect(wrapper.text()).toContain('akiver/cs-demo-manager')
    expect(wrapper.text()).toContain('kaecho/CS2-Skin-Forge')
    const projectButtons = wrapper.findAll('.source-project button')
    expect(projectButtons).toHaveLength(5)
    for (const button of projectButtons) await button.trigger('click')
    expect(openReference.mock.calls.map(call => call[0])).toEqual(['bot-improver', 'demotracer', 'demoparser', 'cs-demo-manager', 'skin-forge'])

    await wrapper.get('[aria-label="返回关于页面"]').trigger('click')
    expect(wrapper.text()).toContain('关于 CS2 人机增强助手')
    await wrapper.setProps({ open: false })
    await wrapper.setProps({ open: true })
    await flushPromises()
    expect(wrapper.text()).not.toContain('ed0ard/CS2-Bot-Improver')
  })
})
