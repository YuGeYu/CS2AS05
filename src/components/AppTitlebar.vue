<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { Copy, Megaphone, Minus, Moon, Square, Sun, X } from 'lucide-vue-next'
import { getCurrentWindow } from '@tauri-apps/api/window'
import appIcon from '@/assets/app-icon.png'

import { useThemePreference } from '@/composables/useThemePreference'
import { announcementState, openAnnouncementCenter } from '@/features/announcements/state'

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

const appWindow = isTauriRuntime() ? getCurrentWindow() : null
const isMaximized = ref(false)
const windowActionBusy = ref(false)
const { theme, toggleTheme } = useThemePreference()
let unlistenResize: (() => void) | undefined

async function syncMaximized() {
  if (!appWindow) return
  const currentWindow = appWindow
  isMaximized.value = await currentWindow.isMaximized()
}

async function startDragging() {
  if (!appWindow) return
  try { await appWindow.startDragging() } catch { /* native window may be closing */ }
}

function toggleMaximize() {
  if (!appWindow || windowActionBusy.value) return
  windowActionBusy.value = true
  let result: unknown
  try { result = appWindow.toggleMaximize() } catch { windowActionBusy.value = false; return }
  void Promise.resolve(result)
    .then(() => syncMaximized())
    .catch(() => undefined)
    .finally(() => { windowActionBusy.value = false })
}

function minimizeWindow() {
  if (!appWindow) return
  void Promise.resolve(appWindow.minimize()).catch(() => undefined)
}

function closeWindow() {
  if (!appWindow) return
  void Promise.resolve(appWindow.close()).catch(() => undefined)
}

onMounted(async () => {
  if (!isTauriRuntime() || !appWindow) return
  await syncMaximized()
  unlistenResize = await appWindow.onResized(() => {
    void syncMaximized()
  })
})

onBeforeUnmount(() => {
  unlistenResize?.()
})
</script>

<template>
  <header class="app-titlebar" @mousedown.left="startDragging" @dblclick="toggleMaximize">
    <div class="titlebar-brand" aria-hidden="true">
      <img class="titlebar-mark" :src="appIcon" alt="" />
      <span class="titlebar-name">CS2 人机增强助手</span>
    </div>
    <div class="titlebar-spacer" />
    <div class="titlebar-controls" @mousedown.stop @dblclick.stop>
      <button
        class="titlebar-announcement-button"
        type="button"
        data-announcement-trigger
        :data-tone="announcementState.latest?.severity || 'idle'"
        :aria-expanded="announcementState.open"
        title="查看官网公告"
        aria-label="查看官网公告"
        @click="openAnnouncementCenter"
      >
        <Megaphone :size="17" /><span>公告</span><b v-if="announcementState.notices.length">{{ announcementState.notices.length }}</b>
      </button>
      <button
        class="titlebar-button"
        type="button"
        :title="theme === 'dark' ? '切换浅色模式' : '切换深色模式'"
        :aria-label="theme === 'dark' ? '切换浅色模式' : '切换深色模式'"
        @click="toggleTheme"
      >
        <Sun v-if="theme === 'dark'" :size="18" />
        <Moon v-else :size="18" />
      </button>
      <button class="titlebar-button" type="button" title="最小化" aria-label="最小化" @click="minimizeWindow"><Minus :size="18" /></button>
      <button class="titlebar-button" type="button" :title="isMaximized ? '还原窗口' : '最大化'" :aria-label="isMaximized ? '还原窗口' : '最大化'" @click="toggleMaximize">
        <Copy v-if="isMaximized" :size="17" />
        <Square v-else :size="17" />
      </button>
      <button class="titlebar-button titlebar-button--close" type="button" title="关闭" aria-label="关闭" @click="closeWindow"><X :size="18" /></button>
    </div>
  </header>
</template>
