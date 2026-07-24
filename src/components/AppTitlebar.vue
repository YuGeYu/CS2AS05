<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { Copy, Minus, Moon, Square, Sun, X } from 'lucide-vue-next'
import { getCurrentWindow } from '@tauri-apps/api/window'

import { useThemePreference } from '@/composables/useThemePreference'

const appWindow = getCurrentWindow()
const isMaximized = ref(false)
const { theme, toggleTheme } = useThemePreference()
let unlistenResize: (() => void) | undefined

function isTauriRuntime() {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window
}

async function syncMaximized() {
  if (!isTauriRuntime()) return
  isMaximized.value = await appWindow.isMaximized()
}

async function startDragging() {
  if (!isTauriRuntime()) return
  await appWindow.startDragging()
}

async function toggleMaximize() {
  if (!isTauriRuntime()) return
  await appWindow.toggleMaximize()
  await syncMaximized()
}

onMounted(async () => {
  if (!isTauriRuntime()) return
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
      <span class="titlebar-mark">CS2</span>
      <span class="titlebar-name">CS2 人机增强助手</span>
    </div>
    <div class="titlebar-spacer" />
    <div class="titlebar-controls" @mousedown.stop @dblclick.stop>
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
      <button class="titlebar-button" type="button" title="最小化" aria-label="最小化" @click="isTauriRuntime() && appWindow.minimize()"><Minus :size="18" /></button>
      <button class="titlebar-button" type="button" :title="isMaximized ? '还原窗口' : '最大化'" :aria-label="isMaximized ? '还原窗口' : '最大化'" @click="toggleMaximize">
        <Copy v-if="isMaximized" :size="17" />
        <Square v-else :size="17" />
      </button>
      <button class="titlebar-button titlebar-button--close" type="button" title="关闭" aria-label="关闭" @click="isTauriRuntime() && appWindow.close()"><X :size="18" /></button>
    </div>
  </header>
</template>
