<script setup lang="ts">
import { defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
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
import { getAssistantPreferences, setCloseChoice, setPromotionPushDisabled } from '@/services/tauri/support'
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
const promotionPushDisabled = ref(false)
const closeChoicePanel = ref<HTMLElement | null>(null)
const closeChoiceBusy = ref(false)
// 关闭选择属于桌面应用偏好，由 Tauri 应用数据提供跨重启记忆；不要退回 sessionStorage。
const closeChoicePreference = ref<'exit' | 'tray' | null>(null)
const StartupIntro = defineAsyncComponent(() => import('@/components/intro/StartupIntro.vue'))
const EasterEggGame = defineAsyncComponent(() => import('@/components/easter-egg/EasterEggGame.vue'))

watch(closeChoiceOpen, async open => {
  if (open) {
    rememberCloseChoice.value = false
    await nextTick()
    closeChoicePanel.value?.focus()
  }
})

function requestClose() {
  onCloseRequested({ preventDefault() {} } as CloseRequestedEvent)
}
function onCloseRequested(event: CloseRequestedEvent) {
  // 已下载更新必须先给玩家安装/延后选择，避免直接退出后丢失待安装状态。
  if (hasPendingDownloadedUpdate()) {
    event.preventDefault()
    exitConfirmOpen.value = true
    return
  }
  const remembered = closeChoicePreference.value
  if (remembered === 'exit') {
    event.preventDefault()
    void destroyAssistantWindow().catch(() => undefined)
    return
  }
  if (remembered === 'tray') {
    event.preventDefault()
    void getCurrentWindow().hide().catch(() => undefined)
    return
  }
  event.preventDefault()
  closeChoiceOpen.value = true
}
async function chooseClose(choice: 'exit' | 'tray') {
  if (closeChoiceBusy.value) return
  closeChoiceBusy.value = true
  try {
    if (rememberCloseChoice.value) {
      try {
        const preferences = await setCloseChoice(choice)
        closeChoicePreference.value = preferences.closeChoice
      } catch {
        // 关闭偏好写入失败时仍完成玩家刚才选择的关闭动作。
      }
    }
    closeChoiceOpen.value = false
    if (choice === 'exit') await destroyAssistantWindow().catch(() => undefined)
    else await getCurrentWindow().hide().catch(() => undefined)
  } finally {
    closeChoiceBusy.value = false
  }
}
function cancelCloseChoice() { if (!closeChoiceBusy.value) closeChoiceOpen.value = false }
function onCloseChoiceKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); cancelCloseChoice() }
  if (event.key !== 'Tab') return
  const focusable = Array.from(closeChoicePanel.value?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled)') || [])
  if (!focusable.length) return
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  if (!first || !last) return
  if (event.shiftKey && document.activeElement === first) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && document.activeElement === last) {
    event.preventDefault()
    first.focus()
  }
}

async function destroyAssistantWindow() {
  await invoke('destroy_scoreboard').catch(() => undefined)
  await getCurrentWindow().destroy()
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
  if (promotionPushDisabled.value || promotion.value) return
  const due = duePromotionCount(promotionStartedAt)
  if (due <= promotionState.shownCount) return
  promotionState.shownCount = due
  void showNextPromotion()
}

async function disablePromotionPush() {
  try {
    await setPromotionPushDisabled(true)
    promotionPushDisabled.value = true
    promotion.value = null
  } catch {
    // Keep the current dialog open when the preference cannot be saved.
  }
}

onMounted(async () => {
  if (introOpen.value) markStartupIntroPlayed()
  window.addEventListener('cs2as:show-pending-update', showPendingSoftwareUpdate)
  void startSoftwareUpdateCoordinator()
  void loadAnnouncements()
  try {
    const preferences = await getAssistantPreferences()
    promotionPushDisabled.value = preferences.promotionPushDisabled
    closeChoicePreference.value = preferences.closeChoice
  } catch { /* 偏好不可用时保持默认行为 */ }
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
  <PromotionPushModal :promotion="promotion" :disabled="promotionPushDisabled" @close="promotion = null" @disable="disablePromotionPush" />
  <div v-if="closeChoiceOpen" class="app-modal-backdrop" role="presentation">
    <section ref="closeChoicePanel" class="app-modal close-choice-modal" role="dialog" aria-modal="true" aria-labelledby="close-choice-title" aria-describedby="close-choice-description" :aria-busy="closeChoiceBusy" tabindex="-1" @keydown="onCloseChoiceKeydown">
      <div class="close-choice-modal__eyebrow">关闭助手</div>
      <h2 id="close-choice-title">要如何关闭助手？</h2>
      <p id="close-choice-description">留在系统托盘后，语音大厅连接仍会继续保持；选择“退出程序”会结束助手及相关连接。</p>
      <div class="close-choice-options" aria-label="关闭方式">
        <button class="close-choice-option" type="button" :disabled="closeChoiceBusy" @click="chooseClose('tray')"><span><strong>留在系统托盘</strong><small>后台保持运行，稍后可从托盘重新打开</small></span><span aria-hidden="true">›</span></button>
        <button class="close-choice-option close-choice-option--exit" type="button" :disabled="closeChoiceBusy" @click="chooseClose('exit')"><span><strong>退出程序</strong><small>结束助手进程并断开语音大厅连接</small></span><span aria-hidden="true">›</span></button>
      </div>
      <label class="modal-check"><input v-model="rememberCloseChoice" type="checkbox" :disabled="closeChoiceBusy" /> <span>记住这次选择，下次直接执行</span></label>
      <button class="close-choice-cancel" type="button" :disabled="closeChoiceBusy" @click="cancelCloseChoice">返回助手 <kbd>Esc</kbd></button>
    </section>
  </div>
</template>
