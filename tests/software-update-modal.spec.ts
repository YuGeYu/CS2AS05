import { beforeEach, describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { dismissRelease, resetSoftwareUpdateStateForTests, shouldPresentRelease } from '@/features/software-updates/state'
import type { SoftwareRelease } from '@/features/software-updates/types'

const baseRelease: SoftwareRelease = {
  id: '1', projectId: 'cs2-bot-improver', channel: 'prod', version: '0.6.0', title: '新版',
  summary: '摘要', items: ['更新项'], severity: 'normal', isCritical: false, isActive: true,
  publishedAt: '2026-07-21', download: { type: 'quark', label: '下载更新', url: 'https://pan.quark.cn/s/abc', code: '' },
}

const mountModal = (release = baseRelease, props = {}) => mount(SoftwareUpdateModal, {
  props: { release, ...props },
  global: { stubs: { Teleport: true } },
})

describe('software update modal', () => {
  beforeEach(() => {
    const values = new Map<string, string>()
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      value: {
        getItem: (key: string) => values.get(key) ?? null,
        setItem: (key: string, value: string) => values.set(key, value),
        removeItem: (key: string) => values.delete(key),
      },
    })
    resetSoftwareUpdateStateForTests()
  })

  it('persists normal dismissal by version while manual checks remain visible', () => {
    dismissRelease(baseRelease)
    expect(shouldPresentRelease(baseRelease, false)).toBe(false)
    expect(shouldPresentRelease(baseRelease, true)).toBe(true)
  })

  it('dismisses recommended releases for the current runtime only', () => {
    const release = { ...baseRelease, severity: 'recommended' as const }
    dismissRelease(release)
    expect(shouldPresentRelease(release, false)).toBe(false)
  })

  it('does not offer later dismissal for critical updates', () => {
    const wrapper = mountModal({ ...baseRelease, severity: 'critical', isCritical: true })
    expect(wrapper.text()).toContain('强制更新')
    expect(wrapper.text()).not.toContain('稍后再说')
  })

  it('renders untrusted summary and items as text', () => {
    const wrapper = mountModal({ ...baseRelease, summary: '<img src=x onerror=alert(1)>', items: ['<script>alert(1)</script>'] })
    expect(wrapper.find('img').exists()).toBe(false)
    expect(wrapper.find('script').exists()).toBe(false)
    expect(wrapper.text()).toContain('<img src=x onerror=alert(1)>')
  })

  it('shows the fixed release-page fallback after download failure', () => {
    const wrapper = mountModal(baseRelease, { downloadError: true })
    expect(wrapper.text()).toContain('打开官网更新日志')
  })
})
