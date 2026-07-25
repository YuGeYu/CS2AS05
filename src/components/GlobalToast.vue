<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import type { ToastMessage } from '@/types/cs2'

const store = useCs2Store()
const panel = usePanelStore()
const visible = ref(false)
const toast = ref<ToastMessage>({ tone: 'info', title: '提示', message: '' })
let toastTimer: ReturnType<typeof setTimeout> | null = null

const toastState = computed(() => toast.value.tone)

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
  }, 4000)
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

onBeforeUnmount(() => {
  if (toastTimer) {
    clearTimeout(toastTimer)
  }
})
</script>

<template>
  <div v-if="visible" class="global-toast" :data-state="toastState" role="status" aria-live="polite">
    <div class="floating-toast__title">{{ toast.title }}</div>
    <div class="floating-toast__body">{{ toast.message }}</div>
  </div>
</template>
