import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount } from '@vue/test-utils'

let maximized = false
let resizeHandler: (() => void) | undefined
const windowApi = {
  close: vi.fn(),
  isMaximized: vi.fn(async () => maximized),
  minimize: vi.fn(),
  onResized: vi.fn(async (handler: () => void) => {
    resizeHandler = handler
    return () => undefined
  }),
  startDragging: vi.fn(),
  toggleMaximize: vi.fn(async () => { maximized = !maximized }),
}

vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => windowApi }))

import AppTitlebar from '@/components/AppTitlebar.vue'
import { readFileSync } from 'node:fs'

describe('app titlebar', () => {
  beforeEach(() => {
    maximized = false
    resizeHandler = undefined
    Object.assign(windowApi, {
      close: vi.fn(),
      isMaximized: vi.fn(async () => maximized),
      minimize: vi.fn(),
      onResized: vi.fn(async (handler: () => void) => {
        resizeHandler = handler
        return () => undefined
      }),
      startDragging: vi.fn(),
      toggleMaximize: vi.fn(async () => { maximized = !maximized }),
    })
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
  })

  it('calls native drag and window controls', async () => {
    const wrapper = mount(AppTitlebar)
    await flushPromises()

    await wrapper.find('.app-titlebar').trigger('mousedown', { button: 0 })
    await wrapper.find('[aria-label="最小化"]').trigger('click')
    await wrapper.find('[aria-label="最大化"]').trigger('click')
    await wrapper.find('[aria-label="关闭"]').trigger('click')

    expect(windowApi.startDragging).toHaveBeenCalledOnce()
    expect(windowApi.minimize).toHaveBeenCalledOnce()
    expect(windowApi.toggleMaximize).toHaveBeenCalledOnce()
    expect(windowApi.close).toHaveBeenCalledOnce()
  })

  it('updates the restore icon after resize synchronization', async () => {
    const wrapper = mount(AppTitlebar)
    await flushPromises()
    maximized = true
    resizeHandler?.()
    await flushPromises()

    expect(wrapper.find('[aria-label="还原窗口"]').exists()).toBe(true)
  })

  it('keeps the titlebar above teleported overlays and reserves its drag strip', () => {
    const styles = readFileSync('src/styles/main.css', 'utf8')
    expect(styles).toContain('.app-titlebar { position: relative; z-index: 100;')
    expect(styles).toContain('.modal-backdrop { position: fixed; inset: 44px 0 0;')
    expect(styles).toContain('.appearance-drawer-layer { position: fixed; z-index: 70; inset: 44px 0 0;')
  })
})
