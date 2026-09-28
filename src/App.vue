<script setup lang="ts">
import { defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow, type CloseRequestedEvent } from '@tauri-apps/api/window'
import AppTitlebar from '@/components/AppTitlebar.vue'
import AppShell from '@/components/AppShell.vue'
import AppearanceSettingsDrawer from '@/components/AppearanceSettingsDrawer.vue'
import AnnouncementCenter from '@/components/AnnouncementCenter.vue'
import GlobalToast from '@/components/GlobalToast.vue'
import PendingUpdateExitModal from '@/components/PendingUpdateExitModal.vue'
import PromotionPushModal from '@/components/PromotionPushModal.vue'
import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { closeSoftwareUpdate, deferSoftwareUpdateInstall, installSoftwareUpdate, openSoftwareUpdateDownload, openSoftwareUpdateReleasePage, showPendingSoftwareUpdate, softwareUpdateCoordinatorState, startSoftwareUpdateCoordinator, startSoftwareUpdateDownload } from '@/features/software-updates/coordinator'
import { hasPendingDownloadedUpdate, prepareDeferredUpdateForExit, softwareUpdaterState } from '@/features/software-updates/updater-state'
import { announcementState, loadAnnouncements } from '@/features/announcements/state'
import { initializeAppearancePreferences } from '@/composables/useAppearancePreferences'
import { markStartupIntroPlayed, shouldPlayStartupIntro } from '@/services/intro-schedule'
import { loadVaultResources } from '@/services/resource-vault'
import { choosePromotionKind, duePromotionCount, nextIndex, PROMOTION_INTERVAL_MS } from '@/features/promotion-push/scheduler'
import type { PromotionPayload } from '@/features/promotion-push/types'

const exitConfirmOpen = ref(false)
const closeChoiceOpen = ref(false)
const rememberCloseChoice = ref(false)
const exiting = ref(false)
const appearancePreferences = initializeAppearancePreferences()
const introOpen = ref(shouldPlayStartupIntro(appearancePreferences.skipIntro))
const easterEggOpen = ref(false)
const appearanceOpen = ref(false)
let easterEggTrigger: HTMLButtonElement | null = null
let unlistenClose: (() => void) | undefined
let promotionTimer: ReturnType<typeof setInterval> | undefined
const promotionStartedAt = Date.now()
const promotion = ref<PromotionPayload | null>(null)
const promotionState = { shownCount: 0, resourceIndex: 0, localIndex: 0, resources: [] as Awaited<ReturnType<typeof loadVaultResources>> }
const StartupIntro = defineAsyncComponent(() => import('@/components/intro/StartupIntro.vue'))
const EasterEggGame = defineAsyncComponent(() => import('@/components/easter-egg/EasterEggGame.vue'))

function requestClose() {
  onCloseRequested({ preventDefault() {} } as CloseRequestedEvent)
}
function onCloseRequested(event: CloseRequestedEvent) {
  const remembered = sessionStorage.getItem('cs2as05-close-choice')
  if (remembered === 'exit') {
    void getCurrentWindow().destroy()
    return
  }
  if (remembered === 'tray') {
    event.preventDefault()
    void getCurrentWindow().hide()
    return
  }
  event.preventDefault()
  closeChoiceOpen.value = true
}
function chooseClose(choice: 'exit' | 'tray') {
  if (rememberCloseChoice.value) sessionStorage.setItem('cs2as05-close-choice', choice)
  closeChoiceOpen.value = false
  if (choice === 'exit') {
    void invoke('destroy_scoreboard').catch(() => undefined)
    void getCurrentWindow().destroy()
  } else void getCurrentWindow().hide()
}

function returnToInstall() {
  exitConfirmOpen.value = false
  window.dispatchEvent(new CustomEvent('cs2as:show-pending-update'))
}

async function exitAnyway() {
  exiting.value = true
  await prepareDeferredUpdateForExit()
  await invoke('destroy_scoreboard').catch(() => undefined)
  await getCurrentWindow().destroy()
}

function openEasterEgg(trigger: HTMLButtonElement) {
  introOpen.value = false
  easterEggTrigger = trigger
  easterEggOpen.value = true
}

async function closeEasterEgg() {
  easterEggOpen.value = false
  await nextTick()
  easterEggTrigger?.focus()
  easterEggTrigger = null
}

async function showNextPromotion() {
  const kind = choosePromotionKind()
  if (kind === 'resource') {
    try {
      if (!promotionState.resources.length) promotionState.resources = await loadVaultResources()
      const resources = [...promotionState.resources]
        .sort((a, b) => Date.parse(b.updatedAt || b.publishedAt || b.createdAt) - Date.parse(a.updatedAt || a.publishedAt || a.createdAt))
        .slice(0, 10)
      if (resources.length) {
        const index = nextIndex(promotionState.resourceIndex, resources.length)
        const resource = resources[index]
        if (!resource) return
        promotionState.resourceIndex = index + 1
        promotion.value = { kind: 'resource', resource }
        return
      }
    } catch {
      // Resource push falls back to the local push library when the vault is unavailable.
    }
  }
  const localItems = ['community', 'donate'] as const
  const index = nextIndex(promotionState.localIndex, localItems.length)
  const localId = localItems[index]
  if (!localId) return
  promotionState.localIndex = index + 1
  promotion.value = { kind: 'local', localId }
}

function checkPromotionSchedule() {
  const due = duePromotionCount(promotionStartedAt)
  if (due <= promotionState.shownCount) return
  promotionState.shownCount = due
  void showNextPromotion()
}

onMounted(async () => {
  if (introOpen.value) markStartupIntroPlayed()
  window.addEventListener('cs2as:show-pending-update', showPendingSoftwareUpdate)
  void startSoftwareUpdateCoordinator()
  void loadAnnouncements()
  promotionTimer = setInterval(checkPromotionSchedule, Math.min(PROMOTION_INTERVAL_MS, 15_000))
  if (isTauri()) unlistenClose = await getCurrentWindow().onCloseRequested(onCloseRequested)
})
onBeforeUnmount(() => {
  if (promotionTimer) clearInterval(promotionTimer)
  unlistenClose?.()
  window.removeEventListener('cs2as:show-pending-update', showPendingSoftwareUpdate)
})
</script>

<template>
  <div class="app-frame">
    <AppTitlebar @request-close="requestClose" @open-appearance="appearanceOpen = true" />
    <AppShell @open-easter-egg="openEasterEgg" />
  </div>
  <GlobalToast />
  <AppearanceSettingsDrawer :open="appearanceOpen" @close="appearanceOpen = false" />
  <AnnouncementCenter v-if="announcementState.open" />
  <SoftwareUpdateModal v-if="softwareUpdateCoordinatorState.activeRelease" :release="softwareUpdateCoordinatorState.activeRelease" :download-error="softwareUpdateCoordinatorState.downloadError" :phase="softwareUpdaterState.phase" :downloaded-bytes="softwareUpdaterState.downloadedBytes" :total-bytes="softwareUpdaterState.totalBytes" :updater-error="softwareUpdaterState.error" @close="closeSoftwareUpdate" @self-update="startSoftwareUpdateDownload" @quark="openSoftwareUpdateDownload" @defer-install="deferSoftwareUpdateInstall" @install="installSoftwareUpdate" @open-release-page="openSoftwareUpdateReleasePage" />
  <Suspense><StartupIntro v-if="introOpen && !easterEggOpen" @close="introOpen = false" /></Suspense>
  <Suspense><EasterEggGame v-if="easterEggOpen" @close="closeEasterEgg" /></Suspense>
  <PendingUpdateExitModal v-if="exitConfirmOpen" :version="softwareUpdaterState.version" :exiting="exiting" @return-install="returnToInstall" @exit-anyway="exitAnyway" />
  <PromotionPushModal :promotion="promotion" @close="promotion = null" />
  <div v-if="closeChoiceOpen" class="app-modal-backdrop" role="presentation">
    <section class="app-modal" role="dialog" aria-modal="true" aria-labelledby="close-choice-title">
      <h2 id="close-choice-title">要如何关闭助手？</h2>
      <p>保留在系统托盘后，语音大厅连接仍会继续保持。</p>
      <label class="modal-check"><input v-model="rememberCloseChoice" type="checkbox" /> 记住选择，下次不再提醒</label>
      <div class="modal-actions"><button type="button" @click="chooseClose('tray')">留在系统托盘</button><button class="primary-button" type="button" @click="chooseClose('exit')">退出程序</button></div>
    </section>
  </div>
</template>
