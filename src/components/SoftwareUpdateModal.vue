<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { Download, ExternalLink, RotateCw, ShieldCheck, X } from 'lucide-vue-next'

import type { SoftwareRelease } from '@/features/software-updates/types'
import type { UpdaterPhase } from '@/features/software-updates/updater-state'

const props = defineProps<{
  release: SoftwareRelease
  downloadError?: boolean
  phase?: UpdaterPhase
  downloadedBytes?: number
  totalBytes?: number | null
  updaterError?: string
}>()

const emit = defineEmits<{
  close: []
  selfUpdate: []
  quark: []
  deferInstall: []
  install: []
  openReleasePage: []
}>()

const dialog = ref<HTMLElement | null>(null)
const phase = computed(() => props.phase ?? 'idle')
const critical = computed(() => props.release.isCritical || props.release.severity === 'critical')
const busy = computed(() => ['checking', 'downloading', 'installing', 'restarting'].includes(phase.value))
const downloaded = computed(() => ['downloaded', 'deferred-current-session'].includes(phase.value))
const progressPercent = computed(() => props.totalBytes ? Math.min(100, Math.round((props.downloadedBytes ?? 0) / props.totalBytes * 100)) : null)
const heading = computed(() => {
  if (downloaded.value) return '更新已下载并通过签名检查'
  if (critical.value) return '强制更新'
  if (props.release.severity === 'recommended') return '推荐更新'
  return '发现新版本'
})
const unavailableText = computed(() => ({
  r2_disabled: '官网直连自更新当前已暂停，请使用夸克更新。',
  artifact_not_ready: '官网直连安装包尚未就绪，请使用夸克更新。',
  unsupported_platform: '当前系统暂不支持便捷自更新，请使用夸克更新。',
  available: '',
}[props.release.selfUpdate.reason]))

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && !critical.value && !busy.value) emit('close')
}

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`
  return `${(value / 1024 / 1024).toFixed(1)} MB`
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
          <button v-if="!critical && !busy" class="icon-button" type="button" aria-label="关闭更新提示" @click="emit('close')">
            <X :size="18" />
          </button>
        </header>

        <template v-if="downloaded">
          <div class="update-verified"><ShieldCheck :size="20" /><span>新版本已下载并通过签名检查。现在安装并重启程序吗？</span></div>
          <div class="update-actions">
            <button class="secondary-button" type="button" @click="emit('deferInstall')">稍后安装</button>
            <button class="primary-button" type="button" @click="emit('install')"><RotateCw :size="18" /><span>安装并重启</span></button>
          </div>
        </template>

        <template v-else>
          <span class="update-severity" :data-severity="release.severity">{{ release.title }}</span>
          <p class="update-summary">{{ release.summary }}</p>
          <ul v-if="release.items.length" class="update-list">
            <li v-for="(item, index) in release.items" :key="index">{{ item }}</li>
          </ul>
          <p class="update-cost-note">官网直连自更新产生的服务费用全部由我们官方承担，不会向你收费。我们会使用赞助资金维持这项服务，并且只推送最新版本。赞助资金用尽，或因运营、安全、维护等情况需要暂停时，我们会关闭官网直连自更新；届时请使用我们提供的夸克链接更新。</p>
          <p v-if="release.download.code" class="download-code">提取码：<strong>{{ release.download.code }}</strong></p>
          <p v-if="!release.selfUpdate.available && unavailableText" class="update-channel-note">{{ unavailableText }}</p>
          <p v-if="downloadError" class="inline-error" role="alert">夸克链接暂不可直接打开，请改用官网更新日志。</p>
          <p v-if="updaterError" class="inline-error" role="alert">{{ updaterError }}</p>

          <div v-if="phase === 'checking' || phase === 'downloading'" class="update-progress" aria-live="polite">
            <div class="update-progress__track" role="progressbar" :aria-valuenow="progressPercent ?? undefined" aria-valuemin="0" aria-valuemax="100">
              <span :style="{ width: progressPercent === null ? '18%' : `${progressPercent}%` }" :data-indeterminate="progressPercent === null"></span>
            </div>
            <span v-if="phase === 'checking'">正在向官网确认更新...</span>
            <span v-else-if="totalBytes">{{ formatBytes(downloadedBytes ?? 0) }} / {{ formatBytes(totalBytes) }} · {{ progressPercent }}%</span>
            <span v-else>已下载 {{ formatBytes(downloadedBytes ?? 0) }}</span>
          </div>

          <div v-if="phase === 'installing' || phase === 'restarting'" class="update-progress" aria-live="polite">
            <span>{{ phase === 'installing' ? '正在交给 Windows 安装器，请勿关闭程序...' : '正在尝试重启程序...' }}</span>
          </div>

          <div class="update-actions">
            <button v-if="!critical && !busy" class="secondary-button" type="button" @click="emit('close')">稍后再说</button>
            <button v-if="downloadError" class="secondary-button" type="button" @click="emit('openReleasePage')">
              <ExternalLink :size="18" /><span>打开官网更新日志</span>
            </button>
            <button class="secondary-button" :class="{ 'primary-button': phase === 'failed' || !release.selfUpdate.available }" type="button" :disabled="phase === 'installing' || phase === 'restarting'" @click="emit('quark')">
              <ExternalLink :size="18" /><span>使用夸克更新</span>
            </button>
            <button v-if="release.selfUpdate.available" class="primary-button" type="button" :disabled="busy" @click="emit('selfUpdate')">
              <Download :size="18" /><span>{{ phase === 'checking' ? '正在确认...' : phase === 'downloading' ? '正在下载...' : phase === 'failed' ? '重试便捷自更新' : '便捷自更新' }}</span>
            </button>
          </div>
        </template>
      </section>
    </div>
  </Teleport>
</template>
