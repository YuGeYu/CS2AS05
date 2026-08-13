<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ImageOff } from 'lucide-vue-next'
import { loadForgeImage, retryForgeImage } from '@/features/skin-forge/image-cache'

const props = defineProps<{ src?: string; alt: string; loading?: 'lazy' | 'eager' }>()
const resolved = ref('')
const failed = ref(false)
const resolving = ref(false)
const visible = ref(false)
const host = ref<HTMLElement | null>(null)
const isTauri = computed(() => '__TAURI_INTERNALS__' in window)
let observer: IntersectionObserver | undefined
let requestVersion = 0
let controller: AbortController | undefined
async function resolve() {
  const version = ++requestVersion
  controller?.abort()
  controller = new AbortController()
  failed.value = false
  if (!props.src || !visible.value) return
  if (!isTauri.value) { resolved.value = props.src; return }
  resolving.value = true
  try {
    const next = await loadForgeImage(props.src, { signal: controller.signal, priority: props.loading === 'eager' ? 'eager' : 'lazy' })
    if (version === requestVersion) resolved.value = next
  } catch {
    if (version === requestVersion) failed.value = true
  } finally {
    if (version === requestVersion) resolving.value = false
  }
}
async function retry() {
  if (!props.src || resolving.value) return
  const version = ++requestVersion
  controller?.abort(); controller = new AbortController()
  failed.value = false; resolving.value = true
  try {
    const next = await retryForgeImage(props.src, { signal: controller.signal, priority: props.loading === 'eager' ? 'eager' : 'lazy' })
    if (version === requestVersion) resolved.value = next
  } catch { if (version === requestVersion) failed.value = true }
  finally { if (version === requestVersion) resolving.value = false }
}
watch(() => props.src, () => void resolve())
watch(visible, value => { if (value) void resolve() })
onMounted(() => {
  if (props.loading === 'eager' || typeof IntersectionObserver === 'undefined') { visible.value = true; return }
  observer = new IntersectionObserver(entries => {
    if (!entries.some(entry => entry.isIntersecting)) return
    visible.value = true
    observer?.disconnect()
  }, { rootMargin: '180px' })
  if (host.value) observer.observe(host.value)
})
onBeforeUnmount(() => { requestVersion += 1; controller?.abort(); observer?.disconnect() })
</script>
<template>
  <span ref="host" class="forge-image-host" :aria-busy="resolving || undefined">
    <img v-if="resolved && !failed" :src="resolved" :alt="alt" :loading="props.loading ?? 'lazy'" decoding="async" @error="failed = true" />
    <span v-else-if="failed" class="forge-image-fallback" role="status" aria-live="polite"><ImageOff :size="26" aria-hidden="true" /><span>图片加载失败</span><button type="button" class="secondary-button forge-image-retry" :disabled="resolving" @click="retry">重试加载</button></span>
    <span v-else class="forge-image-loading" role="img" :aria-label="`${alt} 图片正在载入`" />
  </span>
</template>
