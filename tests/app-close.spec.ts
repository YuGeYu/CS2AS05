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
    onCloseRequested: vi.fn(async (handler: typeof runtime.closeHandler) => {
      runtime.closeHandler = handler
      return vi.fn()
    }),
  }
  return {
    appWindow,
    prepareExit: vi.fn(async () => undefined),
    runtime,
  }
})

vi.mock('@tauri-apps/api/core', () => ({ isTauri: () => true }))
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => mocks.appWindow }))
vi.mock('@/features/software-updates/updater-state', () => ({
  hasPendingDownloadedUpdate: () => mocks.runtime.pending,
  prepareDeferredUpdateForExit: mocks.prepareExit,
  softwareUpdaterState: {
    get phase() { return mocks.runtime.phase },
    version: '0.5.3',
  },
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
  })

  it.each(['idle', 'installing', 'restarting'])('allows native close in %s phase without a pending download', async (phase) => {
    mocks.runtime.phase = phase
    const wrapper = mountApp()
    await flushPromises()
    const event = { preventDefault: vi.fn() }
    await mocks.runtime.closeHandler?.(event)
    expect(event.preventDefault).not.toHaveBeenCalled()
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
})
