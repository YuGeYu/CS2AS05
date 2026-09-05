<script setup lang="ts">
import { computed, ref } from 'vue'
import { CircleArrowUp, Download, Globe2, HandHeart, Info, MessageSquareText, RotateCw } from 'lucide-vue-next'
import DonateModal from '@/components/DonateModal.vue'

import AboutSourcesModal from '@/components/AboutSourcesModal.vue'
import { checkSoftwareUpdates, resumeSoftwareUpdate, softwareUpdateCoordinatorState, softwareUpdateStatusMessage } from '@/features/software-updates/coordinator'
import { softwareUpdaterState } from '@/features/software-updates/updater-state'
import { openIdeaPage, openOfficialSite } from '@/services/tauri/support'

const aboutOpen = ref(false)
const donateOpen = ref(false)
const checking = computed(() => softwareUpdateCoordinatorState.checking)
const statusMessage = softwareUpdateStatusMessage
const hasInstallAction = computed(() => softwareUpdaterState.phase === 'deferred-current-session')
const hasRedownloadReminder = computed(() => softwareUpdaterState.phase === 'deferred-reminder-only')

async function openPage(action: () => Promise<void>, failure: string) {
  try {
    await action()
  } catch {
    // Link failures stay local to this compact support surface.
    console.warn(failure)
  }
}
</script>

<template>
  <section class="support-section" aria-labelledby="support-title">
    <div class="support-copy">
      <p class="overline">支持与服务</p>
      <h2 id="support-title">更新与官网</h2>
      <p class="support-status" aria-live="polite">{{ statusMessage }}</p>
      <button class="about-trigger" type="button" @click="aboutOpen = true"><Info :size="15" />关于与来源</button>
    </div>
    <div class="support-actions">
      <button v-if="hasInstallAction || hasRedownloadReminder" class="primary-button" type="button" :disabled="checking" @click="resumeSoftwareUpdate">
        <RotateCw v-if="hasInstallAction" :size="18" />
        <Download v-else :size="18" />
        <span>{{ hasInstallAction ? '安装已下载版本并重启' : '重新下载并安装' }}</span>
      </button>
      <button class="secondary-button" type="button" :disabled="checking" @click="checkSoftwareUpdates(true)">
        <CircleArrowUp :size="18" :class="{ spinning: checking }" />
        <span>{{ checking ? '检查中...' : '检查更新' }}</span>
      </button>
      <button class="secondary-button" type="button" @click="openPage(openOfficialSite, '打开官网失败，请稍后重试。')">
        <Globe2 :size="18" /><span>打开官网</span>
      </button>
      <button class="secondary-button" type="button" @click="openPage(openIdeaPage, '打开意见页失败，请稍后重试。')">
        <MessageSquareText :size="18" /><span>查看/编辑意见</span>
      </button>
      <button class="secondary-button" type="button" @click="donateOpen = true"><HandHeart :size="18" /><span>赞助开发</span></button>
    </div>
  </section>

  <AboutSourcesModal :open="aboutOpen" @close="aboutOpen = false" />
  <DonateModal :open="donateOpen" @close="donateOpen = false" />
</template>
