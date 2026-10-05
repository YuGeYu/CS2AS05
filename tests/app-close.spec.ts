import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const mocks = vi.hoisted(() => {
  const runtime = {
    closeHandler: undefined as ((event: { preventDefault: () => void }) => Promise<void>) | undefined,
    pending: false,
    phase: 'idle',
  }
  const appWindow = {
    destroy: vi.fn(async () => undefined),
    hide: vi.fn(async () => undefined),
    onCloseRequested: vi.fn(async (handler: typeof runtime.closeHandler) => {
      runtime.closeHandler = handler
      return vi.fn()
    }),
  }
  return {
    appWindow,
    prepareExit: vi.fn(async () => undefined),
    preferences: { closeChoice: null as 'exit' | 'tray' | null },
    runtime,
  }
})

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true, invoke: vi.fn(async () => undefined) }))
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => mocks.appWindow }))
vi.mock('@/features/software-updates/coordinator', () => ({
  closeSoftwareUpdate: vi.fn(),
  deferSoftwareUpdateInstall: vi.fn(),
  installSoftwareUpdate: vi.fn(),
  openSoftwareUpdateDownload: vi.fn(),
  openSoftwareUpdateReleasePage: vi.fn(),
  showPendingSoftwareUpdate: vi.fn(),
  softwareUpdateCoordinatorState: { activeRelease: null, downloadError: false },
  startSoftwareUpdateCoordinator: vi.fn(async () => undefined),
  startSoftwareUpdateDownload: vi.fn(),
}))
vi.mock('@/features/software-updates/updater-state', () => ({
  hasPendingDownloadedUpdate: () => mocks.runtime.pending,
  prepareDeferredUpdateForExit: mocks.prepareExit,
  softwareUpdaterState: {
    get phase() { return mocks.runtime.phase },
    version: '0.5.3',
  },
}))
vi.mock('@/services/tauri/support', () => ({
  getAssistantPreferences: vi.fn(async () => ({ autostartEnabled: false, promotionPushDisabled: false, closeChoice: mocks.preferences.closeChoice })),
  setCloseChoice: vi.fn(async (choice: 'exit' | 'tray' | null) => {
    mocks.preferences.closeChoice = choice
    return { autostartEnabled: false, promotionPushDisabled: false, closeChoice: choice }
  }),
  setPromotionPushDisabled: vi.fn(),
}))

import App from '@/App.vue'

function mountApp() {
  return mount(App, {
    global: {
      stubs: {
        AppShell: true,
        AppTitlebar: true,
        GlobalToast: true,
        PendingUpdateExitModal: {
          emits: ['exitAnyway', 'returnInstall'],
          template: '<button data-test="exit-anyway" @click="$emit(\'exitAnyway\')">退出</button>',
        },
      },
    },
  })
}

describe('application close behavior', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.runtime.closeHandler = undefined
    mocks.runtime.pending = false
    mocks.runtime.phase = 'idle'
    mocks.preferences.closeChoice = null
    mocks.appWindow.hide.mockClear()
  })

  it.each(['idle', 'installing', 'restarting'])('opens the close choice in %s phase without a pending download', async (phase) => {
    mocks.runtime.phase = phase
    const wrapper = mountApp()
    await flushPromises()
    const event = { preventDefault: vi.fn() }
    await mocks.runtime.closeHandler?.(event)
    expect(event.preventDefault).toHaveBeenCalledOnce()
    expect(wrapper.get('[role="dialog"]').text()).toContain('要如何关闭助手')
    wrapper.unmount()
  })

  it('confirms a pending download and destroys the window when the user exits anyway', async () => {
    mocks.runtime.phase = 'downloaded'
    mocks.runtime.pending = true
    const wrapper = mountApp()
    await flushPromises()
    const event = { preventDefault: vi.fn() }
    await mocks.runtime.closeHandler?.(event)
    expect(event.preventDefault).toHaveBeenCalledOnce()
    await wrapper.get('[data-test="exit-anyway"]').trigger('click')
    await flushPromises()
    expect(mocks.prepareExit).toHaveBeenCalledOnce()
    expect(mocks.appWindow.destroy).toHaveBeenCalledOnce()
    wrapper.unmount()
  })

  it('cancels the close choice with Escape without hiding or destroying', async () => {
    const wrapper = mountApp()
    await flushPromises()
    await mocks.runtime.closeHandler?.({ preventDefault: vi.fn() })
    await wrapper.get('[role="dialog"]').trigger('keydown', { key: 'Escape' })
    expect(wrapper.find('[role="dialog"]').exists()).toBe(false)
    expect(mocks.appWindow.hide).not.toHaveBeenCalled()
    expect(mocks.appWindow.destroy).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('persists a remembered tray choice and applies it on the next close', async () => {
    const first = mountApp()
    await flushPromises()
    await mocks.runtime.closeHandler?.({ preventDefault: vi.fn() })
    await first.get('.modal-check input').setValue(true)
    await first.get('.close-choice-option').trigger('click')
    await flushPromises()
    expect(mocks.preferences.closeChoice).toBe('tray')
    expect(mocks.appWindow.hide).toHaveBeenCalledOnce()
    first.unmount()

    const second = mountApp()
    await flushPromises()
    const event = { preventDefault: vi.fn() }
    await mocks.runtime.closeHandler?.(event)
    expect(event.preventDefault).toHaveBeenCalledOnce()
    expect(mocks.appWindow.hide).toHaveBeenCalledTimes(2)
    expect(second.find('[role="dialog"]').exists()).toBe(false)
    second.unmount()
  })
})
