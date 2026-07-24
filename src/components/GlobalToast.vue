<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'

import { useCs2Store } from '@/stores/cs2'

const store = useCs2Store()
const visible = ref(false)
const message = ref('')
let toastTimer: ReturnType<typeof setTimeout> | null = null

const toastState = computed<'ready' | 'warn' | 'danger' | 'info'>(() => {
  const text = message.value
  if (/(失败|错误|无法|异常|未能)/.test(text)) {
    return 'danger'
  }
  if (/(请先|需要|运行中|没有|未选择|未找到|待)/.test(text)) {
    return 'warn'
  }
  if (/(已保存|已复制|已选择|已安装|已删除|已恢复|已读取|成功)/.test(text)) {
    return 'ready'
  }
  return 'info'
})

const toastTitle = computed(() => {
  if (toastState.value === 'ready') {
    return '已完成'
  }
  if (toastState.value === 'warn') {
    return '需要处理'
  }
  if (toastState.value === 'danger') {
    return '操作失败'
  }
  return '提示'
})

function showToast(nextMessage: string) {
  if (!nextMessage.trim()) {
    return
  }

  message.value = nextMessage
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

onBeforeUnmount(() => {
  if (toastTimer) {
    clearTimeout(toastTimer)
  }
})
</script>

<template>
  <div v-if="visible" class="global-toast" :data-state="toastState" role="status" aria-live="polite">
    <div class="floating-toast__title">{{ toastTitle }}</div>
    <div class="floating-toast__body">{{ message }}</div>
  </div>
</template>
