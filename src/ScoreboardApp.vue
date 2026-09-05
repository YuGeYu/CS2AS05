<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { getCurrentWindow, type CloseRequestedEvent } from '@tauri-apps/api/window'
import { X } from 'lucide-vue-next'
import PostMatchScoreboard from '@/components/scoreboard/PostMatchScoreboard.vue'
import type { DemoReport } from '@/types/demo'

interface PendingReport { reportId: number; sequence: number }

const report = ref<DemoReport | null>(null)
const error = ref('')
let unlistenLoad: UnlistenFn | undefined
let unlistenClose: UnlistenFn | undefined

function validPending(value: PendingReport | null): value is PendingReport {
  return value !== null && Number.isSafeInteger(value.reportId) && value.reportId > 0 && Number.isSafeInteger(value.sequence) && value.sequence > 0
}

async function hide() {
  try {
    await invoke('hide_scoreboard')
  } catch (value) {
    error.value = `无法关闭战报窗口：${String(value)}`
  }
}

async function load(pending: PendingReport) {
  if (!validPending(pending)) {
    error.value = '收到无效的战报请求。'
    return
  }
  error.value = ''
  document.body.dataset.scoreboardReady = 'false'
  try {
    const next = await invoke<DemoReport>('get_demo_report', { id: pending.reportId })
    if (next.schemaVersion < 6 || next.metricsVersion !== 'lb-rating-2.0') throw new Error('报告版本过旧，请重新解析。')
    report.value = next
    await nextTick()
    await invoke('scoreboard_present', { reportId: pending.reportId, sequence: pending.sequence })
    document.body.dataset.scoreboardReady = 'true'
  } catch (value) {
    error.value = String(value)
    await invoke('report_scoreboard_boot_error', { message: error.value }).catch(() => undefined)
  }
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') void hide()
}

async function onCloseRequested(event: CloseRequestedEvent) {
  event.preventDefault()
  await hide()
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  unlistenLoad = await listen<PendingReport>('scoreboard://load-report', event => void load(event.payload))
  unlistenClose = await getCurrentWindow().onCloseRequested(onCloseRequested)
  const pending = await invoke<PendingReport | null>('scoreboard_frontend_ready')
  if (validPending(pending)) await load(pending)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  unlistenLoad?.()
  unlistenClose?.()
})
</script>

<template><main class="scoreboard-shell"><header class="scoreboard-titlebar"><strong>本局战报</strong><span>{{ report?.summary.mapName || '地图未识别' }}</span><button type="button" title="关闭" aria-label="关闭本局战报" @click="hide"><X :size="19" /></button></header><p v-if="error" class="scoreboard-error" role="alert">{{ error }}</p><PostMatchScoreboard v-else-if="report" :report="report" /><div v-else class="scoreboard-loading">正在载入战报</div></main></template>
