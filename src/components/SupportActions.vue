<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { CircleArrowUp, Globe2, MessageSquareText } from 'lucide-vue-next'

import AboutSourcesModal from '@/components/AboutSourcesModal.vue'
import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { appConfig } from '@/config/app'
import { checkForSoftwareUpdates, dismissRelease, shouldPresentRelease } from '@/features/software-updates/state'
import type { SoftwareRelease } from '@/features/software-updates/types'
import { openIdeaPage, openOfficialSite, openReleasePage, openUpdateDownload } from '@/services/tauri/support'

const checking = ref(false)
const statusMessage = ref('启动后自动检查更新，也可以随时手动检查。')
const activeRelease = ref<SoftwareRelease | null>(null)
const aboutOpen = ref(false)
const downloadError = ref(false)
const downloading = ref(false)

async function check(manual = false) {
  checking.value = true
  statusMessage.value = '正在检查更新...'
  try {
    const result = await checkForSoftwareUpdates(manual)
    if (result.status === 'disabled') statusMessage.value = '自动更新检查当前已关闭。'
    if (result.status === 'current') statusMessage.value = `当前 ${appConfig.appVersion} 已是最新版本。`
    if (result.status === 'failed') statusMessage.value = result.message
    if (result.status === 'available') {
      statusMessage.value = `发现新版本 ${result.release.version}。`
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
    statusMessage.value = failure
  }
}

function closeUpdate() {
  if (!activeRelease.value) return
  dismissRelease(activeRelease.value)
  activeRelease.value = null
  downloadError.value = false
}

async function downloadUpdate() {
  if (!activeRelease.value) return
  downloading.value = true
  downloadError.value = false
  try {
    await openUpdateDownload(activeRelease.value.download.url)
  } catch {
    downloadError.value = true
  } finally {
    downloading.value = false
  }
}

onMounted(() => void check(false))
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
    :downloading="downloading"
    @close="closeUpdate"
    @download="downloadUpdate"
    @open-release-page="openPage(openReleasePage, '打开官网更新日志失败，请稍后重试。')"
  />
  <AboutSourcesModal :open="aboutOpen" @close="aboutOpen = false" />
</template>
