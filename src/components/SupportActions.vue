<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { CircleArrowUp, Download, Globe2, MessageSquareText, RotateCw } from 'lucide-vue-next'

import AboutSourcesModal from '@/components/AboutSourcesModal.vue'
import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { appConfig } from '@/config/app'
import { checkForSoftwareUpdates, dismissRelease, shouldPresentRelease } from '@/features/software-updates/state'
import {
  deferDownloadedSoftwareUpdate,
  discardStaleDownloadedUpdate,
  downloadSoftwareUpdate,
  initializeSoftwareUpdaterState,
  installDownloadedSoftwareUpdate,
  softwareUpdaterState,
} from '@/features/software-updates/updater-state'
import type { SoftwareRelease } from '@/features/software-updates/types'
import { openIdeaPage, openOfficialSite, openReleasePage, openUpdateDownload } from '@/services/tauri/support'

const checking = ref(false)
const baseStatusMessage = ref('启动后自动检查更新，也可以随时手动检查。')
const activeRelease = ref<SoftwareRelease | null>(null)
const latestRelease = ref<SoftwareRelease | null>(null)
const aboutOpen = ref(false)
const downloadError = ref(false)

const statusMessage = computed(() => {
  if (softwareUpdaterState.phase === 'deferred-current-session') return `v${softwareUpdaterState.version} 已下载，等待安装。`
  if (softwareUpdaterState.phase === 'deferred-reminder-only') return `上次选择稍后安装 v${softwareUpdaterState.version}；程序关闭后安装包不会保留，需要重新下载。`
  if (softwareUpdaterState.phase === 'failed' && softwareUpdaterState.error) return softwareUpdaterState.error
  return baseStatusMessage.value
})
const hasInstallAction = computed(() => softwareUpdaterState.phase === 'deferred-current-session')
const hasRedownloadReminder = computed(() => softwareUpdaterState.phase === 'deferred-reminder-only')

async function check(manual = false) {
  checking.value = true
  baseStatusMessage.value = '正在检查更新...'
  try {
    const result = await checkForSoftwareUpdates(manual)
    if (result.status === 'disabled') baseStatusMessage.value = '自动更新检查当前已关闭。'
    if (result.status === 'current') {
      baseStatusMessage.value = `当前 ${appConfig.appVersion} 已是最新版本。`
      if (softwareUpdaterState.phase === 'deferred-reminder-only') await discardStaleDownloadedUpdate()
    }
    if (result.status === 'failed') baseStatusMessage.value = result.message
    if (result.status === 'available') {
      if (softwareUpdaterState.version && softwareUpdaterState.version !== result.release.version) await discardStaleDownloadedUpdate()
      latestRelease.value = result.release
      baseStatusMessage.value = `发现新版本 ${result.release.version}。`
      if (shouldPresentRelease(result.release, manual)) activeRelease.value = result.release
    }
  } finally {
    checking.value = false
  }
}

async function openPage(action: () => Promise<void>, failure: string) {
  try {
    await action()
  } catch {
    baseStatusMessage.value = failure
  }
}

function closeUpdate() {
  if (!activeRelease.value || ['checking', 'downloading', 'installing', 'restarting'].includes(softwareUpdaterState.phase)) return
  dismissRelease(activeRelease.value)
  activeRelease.value = null
  downloadError.value = false
}

async function openQuarkUpdate() {
  if (!activeRelease.value) return
  downloadError.value = false
  try {
    await openUpdateDownload(activeRelease.value.download.url)
  } catch {
    downloadError.value = true
  }
}

async function startSelfUpdate() {
  if (!activeRelease.value) return
  try {
    await downloadSoftwareUpdate(activeRelease.value.version)
  } catch {
    // The application-level updater state exposes the recoverable error and keeps Quark available.
  }
}

function deferInstall() {
  deferDownloadedSoftwareUpdate()
  activeRelease.value = null
}

async function installUpdate() {
  try {
    await installDownloadedSoftwareUpdate()
  } catch {
    // The modal remains open with an actionable fallback.
  }
}

async function resumePendingUpdate() {
  if (hasInstallAction.value && latestRelease.value) {
    activeRelease.value = latestRelease.value
    return
  }
  await check(true)
}

function showPendingUpdate() {
  if (latestRelease.value) activeRelease.value = latestRelease.value
}

onMounted(() => {
  initializeSoftwareUpdaterState()
  window.addEventListener('cs2as:show-pending-update', showPendingUpdate)
  void check(false)
})
onBeforeUnmount(() => window.removeEventListener('cs2as:show-pending-update', showPendingUpdate))
</script>

<template>
  <section class="support-section" aria-labelledby="support-title">
    <div class="support-copy">
      <p class="overline">支持与服务</p>
      <h2 id="support-title">更新与官网</h2>
      <p class="support-status" aria-live="polite">{{ statusMessage }}</p>
      <button class="about-trigger" type="button" @click="aboutOpen = true">关于与来源</button>
    </div>
    <div class="support-actions">
      <button v-if="hasInstallAction || hasRedownloadReminder" class="primary-button" type="button" :disabled="checking" @click="resumePendingUpdate">
        <RotateCw v-if="hasInstallAction" :size="18" />
        <Download v-else :size="18" />
        <span>{{ hasInstallAction ? '安装已下载版本并重启' : '重新下载并安装' }}</span>
      </button>
      <button class="secondary-button" type="button" :disabled="checking" @click="check(true)">
        <CircleArrowUp :size="18" :class="{ spinning: checking }" />
        <span>{{ checking ? '检查中...' : '检查更新' }}</span>
      </button>
      <button class="secondary-button" type="button" @click="openPage(openOfficialSite, '打开官网失败，请稍后重试。')">
        <Globe2 :size="18" /><span>打开官网</span>
      </button>
      <button class="secondary-button" type="button" @click="openPage(openIdeaPage, '打开意见页失败，请稍后重试。')">
        <MessageSquareText :size="18" /><span>查看/编辑意见</span>
      </button>
    </div>
  </section>

  <SoftwareUpdateModal
    v-if="activeRelease"
    :release="activeRelease"
    :download-error="downloadError"
    :phase="softwareUpdaterState.phase"
    :downloaded-bytes="softwareUpdaterState.downloadedBytes"
    :total-bytes="softwareUpdaterState.totalBytes"
    :updater-error="softwareUpdaterState.error"
    @close="closeUpdate"
    @self-update="startSelfUpdate"
    @quark="openQuarkUpdate"
    @defer-install="deferInstall"
    @install="installUpdate"
    @open-release-page="openPage(openReleasePage, '打开官网更新日志失败，请稍后重试。')"
  />
  <AboutSourcesModal :open="aboutOpen" @close="aboutOpen = false" />
</template>
