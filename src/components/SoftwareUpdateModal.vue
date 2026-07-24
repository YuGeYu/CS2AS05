<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Download, ExternalLink, X } from 'lucide-vue-next'

import type { SoftwareRelease } from '@/features/software-updates/types'

const props = defineProps<{
  release: SoftwareRelease
  downloadError?: boolean
  downloading?: boolean
}>()

const emit = defineEmits<{
  close: []
  download: []
  openReleasePage: []
}>()

const dialog = ref<HTMLElement | null>(null)
const critical = computed(() => props.release.isCritical || props.release.severity === 'critical')
const heading = computed(() => {
  if (critical.value) return '强制更新'
  if (props.release.severity === 'recommended') return '推荐更新'
  return '发现新版本'
})

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && !critical.value) emit('close')
}

onMounted(() => {
  document.addEventListener('keydown', onKeydown)
  dialog.value?.focus()
})
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <div class="modal-backdrop" role="presentation">
      <section ref="dialog" class="update-dialog" role="dialog" aria-modal="true" aria-labelledby="update-title" tabindex="-1">
        <header class="modal-heading">
          <div>
            <p class="overline">版本 {{ release.version }}</p>
            <h2 id="update-title">{{ heading }}</h2>
          </div>
          <button v-if="!critical" class="icon-button" type="button" aria-label="关闭更新提示" @click="emit('close')">
            <X :size="18" />
          </button>
        </header>
        <span class="update-severity" :data-severity="release.severity">{{ release.title }}</span>
        <p class="update-summary">{{ release.summary }}</p>
        <ul v-if="release.items.length" class="update-list">
          <li v-for="(item, index) in release.items" :key="index">{{ item }}</li>
        </ul>
        <p v-if="release.download.code" class="download-code">提取码：<strong>{{ release.download.code }}</strong></p>
        <p v-if="downloadError" class="inline-error" role="alert">下载链接暂不可直接打开，请改用官网更新日志。</p>
        <div class="update-actions">
          <button v-if="!critical" class="secondary-button" type="button" @click="emit('close')">稍后再说</button>
          <button v-if="downloadError" class="secondary-button" type="button" @click="emit('openReleasePage')">
            <ExternalLink :size="18" /><span>打开官网更新日志</span>
          </button>
          <button class="primary-button" type="button" :disabled="downloading" @click="emit('download')">
            <Download :size="18" /><span>{{ downloading ? '正在打开...' : release.download.label || '获取更新' }}</span>
          </button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
