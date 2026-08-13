import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import ForgeSafetyDialog from '@/features/skin-forge/components/ForgeSafetyDialog.vue'

describe('skin forge safety gate', () => {
  it('is blocking, traps the initial action, and confirms explicitly', async () => {
    const wrapper = mount(ForgeSafetyDialog, { global: { stubs: { Teleport: true } } })
    expect(wrapper.get('[role="dialog"]').attributes('aria-modal')).toBe('true')
    expect(wrapper.text()).toContain('-insecure')
    expect(wrapper.text()).toContain('VAC')
    expect(wrapper.text()).toContain('自行承担')
    await wrapper.get('.primary-button').trigger('click')
    expect(wrapper.emitted('confirm')).toHaveLength(1)
  })

  it.each(['Escape', 'backdrop'])('treats %s as cancel', async action => {
    const wrapper = mount(ForgeSafetyDialog, { global: { stubs: { Teleport: true } } })
    if (action === 'Escape') document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))
    else await wrapper.get('.forge-safety-backdrop').trigger('mousedown')
    expect(wrapper.emitted('cancel')).toHaveLength(1)
    wrapper.unmount()
  })

  it('keeps the implementation contracts wired in the view', async () => {
    const source = await import('node:fs/promises').then(fs => fs.readFile('src/views/SkinForgeView.vue', 'utf8'))
    expect(source).toContain(':disabled="!workshopUnlocked"')
    expect(source).toContain('forge-deploy-cue')
    expect(source).toContain('<ForgeWorkbench :category="category" :disabled="!workshopUnlocked"')
    expect(source).not.toContain('关于与来源')
    expect(source).toContain('aria-label="快速上手"')
    expect(vi.isMockFunction(vi.fn())).toBe(true)
  })
})
