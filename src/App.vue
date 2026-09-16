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
import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { closeSoftwareUpdate, deferSoftwareUpdateInstall, installSoftwareUpdate, openSoftwareUpdateDownload, openSoftwareUpdateReleasePage, showPendingSoftwareUpdate, softwareUpdateCoordinatorState, startSoftwareUpdateCoordinator, startSoftwareUpdateDownload } from '@/features/software-updates/coordinator'
import { hasPendingDownloadedUpdate, prepareDeferredUpdateForExit, softwareUpdaterState } from '@/features/software-updates/updater-state'
import { announcementState, loadAnnouncements } from '@/features/announcements/state'
import { initializeAppearancePreferences } from '@/composables/useAppearancePreferences'
import { markStartupIntroPlayed, shouldPlayStartupIntro } from '@/services/intro-schedule'

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

onMounted(async () => {
  if (introOpen.value) markStartupIntroPlayed()
  window.addEventListener('cs2as:show-pending-update', showPendingSoftwareUpdate)
  void startSoftwareUpdateCoordinator()
  void loadAnnouncements()
  if (isTauri()) unlistenClose = await getCurrentWindow().onCloseRequested(onCloseRequested)
})
onBeforeUnmount(() => {
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
  <div v-if="closeChoiceOpen" class="app-modal-backdrop" role="presentation">
    <section class="app-modal" role="dialog" aria-modal="true" aria-labelledby="close-choice-title">
      <h2 id="close-choice-title">要如何关闭助手？</h2>
      <p>保留在系统托盘后，语音大厅连接仍会继续保持。</p>
      <label class="modal-check"><input v-model="rememberCloseChoice" type="checkbox" /> 记住选择，下次不再提醒</label>
      <div class="modal-actions"><button type="button" @click="chooseClose('tray')">留在系统托盘</button><button class="primary-button" type="button" @click="chooseClose('exit')">退出程序</button></div>
    </section>
  </div>
</template>
