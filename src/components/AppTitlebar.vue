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
const currentContext = ref('概览')
const contextLabels: Record<string, string> = {
  overview: '概览', presets: '人机预设', items: 'Bot 物品', knives: '刀具', inventory: '库存换肤',
  commands: '命令', demoReview: '对局复盘', quickSupport: '快快客服', install: '安装与诊断',
}
let unlistenResize: (() => void) | undefined
function syncContext(event: Event) {
  const key = (event as CustomEvent<string>).detail
  if (key && contextLabels[key]) currentContext.value = contextLabels[key]
}

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
  window.addEventListener('cs2as:view-changed', syncContext)
  if (!isTauriRuntime() || !appWindow) return
  await syncMaximized()
  unlistenResize = await appWindow.onResized(() => {
    void syncMaximized()
  })
})

onBeforeUnmount(() => {
  window.removeEventListener('cs2as:view-changed', syncContext)
  unlistenResize?.()
})
</script>

<template>
  <header class="app-titlebar" role="banner" @mousedown.left="startDragging" @dblclick="toggleMaximize">
    <a class="skip-link" href="#main-content">跳转到主要内容</a>
    <div class="titlebar-brand">
      <img class="titlebar-mark" :src="appIcon" alt="" />
      <span class="titlebar-brand-copy"><span class="titlebar-name">CS2 人机增强助手</span><span class="titlebar-context" aria-live="polite">{{ currentContext }}</span></span>
    </div>
    <div class="titlebar-status" aria-label="当前工作区"><span class="titlebar-status-dot" aria-hidden="true" />工作台</div>
    <div class="titlebar-spacer" />
    <div class="titlebar-controls" aria-label="应用控制" @mousedown.stop @dblclick.stop>
      <div class="titlebar-control-group titlebar-control-group--app" aria-label="应用通知">
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
      </div>
      <div class="titlebar-control-group titlebar-control-group--appearance" aria-label="外观设置">
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
      </div>
      <div class="titlebar-control-group titlebar-control-group--window" aria-label="窗口控制">
      <button class="titlebar-button" type="button" title="最小化" aria-label="最小化" @click="minimizeWindow"><Minus :size="18" /></button>
      <button class="titlebar-button" type="button" :title="isMaximized ? '还原窗口' : '最大化'" :aria-label="isMaximized ? '还原窗口' : '最大化'" @click="toggleMaximize">
        <Copy v-if="isMaximized" :size="17" />
        <Square v-else :size="17" />
      </button>
      <button class="titlebar-button titlebar-button--close" type="button" title="关闭" aria-label="关闭" @click="closeWindow"><X :size="18" /></button>
      </div>
    </div>
  </header>
</template>
