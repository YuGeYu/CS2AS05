<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { AlertTriangle, CheckCircle2, Info, ShieldAlert } from 'lucide-vue-next'

import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import type { ToastMessage } from '@/types/cs2'

const store = useCs2Store()
const panel = usePanelStore()
const visible = ref(false)
const toast = ref<ToastMessage>({ tone: 'info', title: '提示', message: '' })
let toastTimer: ReturnType<typeof setTimeout> | null = null

const toastState = computed(() => toast.value.tone)
const toastIcon = computed(() => ({ ready: CheckCircle2, warn: AlertTriangle, danger: ShieldAlert, info: Info })[toast.value.tone])

function showToast(nextToast: ToastMessage) {
  if (!nextToast.message.trim()) {
    return
  }

  toast.value = nextToast
  visible.value = true
  if (toastTimer) {
    clearTimeout(toastTimer)
  }
  toastTimer = setTimeout(() => {
    visible.value = false
    toastTimer = null
  }, Math.min(10_000, Math.max(800, nextToast.durationMs ?? 4_000)))
}

function isToastMessage(value: unknown): value is ToastMessage {
  if (!value || typeof value !== 'object') return false
  const detail = value as Record<string, unknown>
  return ['ready', 'warn', 'danger', 'info'].includes(String(detail.tone))
    && typeof detail.title === 'string' && detail.title.length <= 80
    && typeof detail.message === 'string' && detail.message.length <= 500
    && (detail.durationMs === undefined || typeof detail.durationMs === 'number')
}

function onToast(event: Event) {
  const detail = (event as CustomEvent<unknown>).detail
  if (isToastMessage(detail)) showToast(detail)
}

watch(
  () => store.message,
  (nextMessage) => {
    if (nextMessage) {
      showToast(nextMessage)
    }
  },
)

watch(() => panel.lastError, (error) => {
  if (error) showToast({ tone: 'danger', title: 'Panel 操作失败', message: error })
})

window.addEventListener('cs2as:toast', onToast)

onBeforeUnmount(() => {
  window.removeEventListener('cs2as:toast', onToast)
  if (toastTimer) {
    clearTimeout(toastTimer)
  }
})
</script>

<template>
  <div v-if="visible" class="global-toast" :data-state="toastState" :role="toastState === 'danger' ? 'alert' : 'status'" :aria-live="toastState === 'danger' ? 'assertive' : 'polite'">
    <component :is="toastIcon" class="floating-toast__icon" :size="20" aria-hidden="true" />
    <div><div class="floating-toast__title">{{ toast.title }}</div><div class="floating-toast__body">{{ toast.message }}</div></div>
  </div>
</template>
