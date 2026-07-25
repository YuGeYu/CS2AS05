<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { Check, FolderOpen, ScanSearch, Square, X } from 'lucide-vue-next'

import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: []; browse: [] }>()
const cs2 = useCs2Store()
const panel = usePanelStore()
const dialog = ref<HTMLElement | null>(null)
const stopping = ref(false)
const choosing = ref('')
const error = ref('')
let stopPromise: Promise<boolean> | null = null

const progress = computed(() => Math.min(100, (cs2.rootScan.elapsedMs / 10_000) * 100))
const resultText = computed(() => {
  if (cs2.rootScan.running) return `已检查 ${cs2.rootScan.checkedLocations} 个位置`
  return `扫描完成，找到 ${cs2.rootScan.candidates.length} 个候选目录`
})

watch(() => props.open, async (open) => {
  if (!open) return
  stopping.value = false
  choosing.value = ''
  stopPromise = null
  error.value = ''
  await nextTick()
  dialog.value?.focus()
  try {
    await cs2.scanSuggestedRoots()
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason)
  }
})

async function stopOnce() {
  if (!cs2.rootScan.running) return false
  if (!stopPromise) {
    stopping.value = true
    stopPromise = cs2.stopSuggestedRoots().finally(() => { stopping.value = false })
  }
  return stopPromise
}

async function close() {
  await stopOnce()
  emit('close')
}

async function choose(path: string) {
  choosing.value = path
  error.value = ''
  try {
    await stopOnce()
    await cs2.selectRoot(path)
    await panel.refresh(path)
    emit('close')
  } catch (reason) {
    error.value = reason instanceof Error ? reason.message : String(reason)
  } finally {
    choosing.value = ''
  }
}

async function browse() {
  await stopOnce()
  emit('close')
  emit('browse')
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.open) void close()
}

document.addEventListener('keydown', onKeydown)
onBeforeUnmount(() => {
  document.removeEventListener('keydown', onKeydown)
  void stopOnce()
})
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation">
      <section ref="dialog" class="root-suggestions" role="dialog" aria-modal="true" aria-labelledby="root-suggestions-title" tabindex="-1">
        <header class="modal-heading">
          <span class="root-suggestions__icon"><ScanSearch :size="20" /></span>
          <div><p class="overline">猜你想选</p><h2 id="root-suggestions-title">查找 CS2 游戏目录</h2></div>
          <button class="icon-button modal-close" type="button" aria-label="关闭目录推荐" @click="close"><X :size="18" /></button>
        </header>

        <div class="root-suggestions__status">
          <div><strong>{{ resultText }}</strong><span>{{ cs2.rootScan.currentLocation || '正在读取 Steam 配置…' }}</span></div>
          <div class="root-suggestions__progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
        </div>

        <ul v-if="cs2.rootScan.candidates.length" class="root-suggestions__list">
          <li v-for="candidate in cs2.rootScan.candidates" :key="candidate.path">
            <div><strong :title="candidate.path">{{ candidate.path }}</strong><span>{{ candidate.source }} · {{ candidate.confidence === 'verified' ? '已验证' : '可能有效' }}</span></div>
            <button class="secondary-button" type="button" :disabled="Boolean(choosing)" @click="choose(candidate.path)"><Check :size="17" />选择</button>
          </li>
        </ul>
        <div v-else-if="!cs2.rootScan.running" class="root-suggestions__empty"><FolderOpen :size="24" /><span>未找到可验证的 CS2 目录</span></div>
        <p v-if="error" class="inline-error" role="alert">{{ error }}</p>

        <footer class="dialog-actions">
          <button class="secondary-button" type="button" @click="browse"><FolderOpen :size="17" />选择目录</button>
          <button v-if="cs2.rootScan.running" class="danger-button" type="button" :disabled="stopping" @click="stopOnce"><Square :size="16" fill="currentColor" />{{ stopping ? '正在停止' : '停止扫描' }}</button>
          <button v-else class="primary-button" type="button" @click="close">完成</button>
        </footer>
      </section>
    </div>
  </Teleport>
</template>
