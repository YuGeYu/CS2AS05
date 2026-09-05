<script setup lang="ts">
import { defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow, type CloseRequestedEvent } from '@tauri-apps/api/window'
import AppTitlebar from '@/components/AppTitlebar.vue'
import AppShell from '@/components/AppShell.vue'
import AnnouncementCenter from '@/components/AnnouncementCenter.vue'
import GlobalToast from '@/components/GlobalToast.vue'
import PendingUpdateExitModal from '@/components/PendingUpdateExitModal.vue'
import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { closeSoftwareUpdate, deferSoftwareUpdateInstall, installSoftwareUpdate, openSoftwareUpdateDownload, openSoftwareUpdateReleasePage, showPendingSoftwareUpdate, softwareUpdateCoordinatorState, startSoftwareUpdateCoordinator, startSoftwareUpdateDownload } from '@/features/software-updates/coordinator'
import { hasPendingDownloadedUpdate, prepareDeferredUpdateForExit, softwareUpdaterState } from '@/features/software-updates/updater-state'
import { announcementState, loadAnnouncements } from '@/features/announcements/state'
import { initializeAppearancePreferences } from '@/composables/useAppearancePreferences'

const exitConfirmOpen = ref(false)
const exiting = ref(false)
const appearancePreferences = initializeAppearancePreferences()
const introOpen = ref(!appearancePreferences.skipIntro)
const easterEggOpen = ref(false)
let easterEggTrigger: HTMLButtonElement | null = null
let unlistenClose: (() => void) | undefined
const StartupIntro = defineAsyncComponent(() => import('@/components/intro/StartupIntro.vue'))
const EasterEggGame = defineAsyncComponent(() => import('@/components/easter-egg/EasterEggGame.vue'))

function onCloseRequested(event: CloseRequestedEvent) {
  if (!hasPendingDownloadedUpdate()) {
    // Let the native close proceed immediately; scoreboard cleanup is best effort
    // and must not hold the window event loop on an IPC round trip.
    void invoke('destroy_scoreboard').catch(() => undefined)
    return
  }
  event.preventDefault()
  void invoke('hide_scoreboard').catch(() => undefined)
  exitConfirmOpen.value = true
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
  window.addEventListener('cs2as:show-pending-update', showPendingSoftwareUpdate)
  void startSoftwareUpdateCoordinator()
  void loadAnnouncements()
  if (isTauri()) unlistenClose = await getCurrentWindow().onCloseRequested(onCloseRequested)
})
onBeforeUnmount(() => { unlistenClose?.(); window.removeEventListener('cs2as:show-pending-update', showPendingSoftwareUpdate) })
</script>

<template>
  <div class="app-frame">
    <AppTitlebar />
    <AppShell @open-easter-egg="openEasterEgg" />
  </div>
  <GlobalToast />
  <AnnouncementCenter v-if="announcementState.open" />
  <SoftwareUpdateModal
    v-if="softwareUpdateCoordinatorState.activeRelease"
    :release="softwareUpdateCoordinatorState.activeRelease"
    :download-error="softwareUpdateCoordinatorState.downloadError"
    :phase="softwareUpdaterState.phase"
    :downloaded-bytes="softwareUpdaterState.downloadedBytes"
    :total-bytes="softwareUpdaterState.totalBytes"
    :updater-error="softwareUpdaterState.error"
    @close="closeSoftwareUpdate" @self-update="startSoftwareUpdateDownload" @quark="openSoftwareUpdateDownload"
    @defer-install="deferSoftwareUpdateInstall" @install="installSoftwareUpdate" @open-release-page="openSoftwareUpdateReleasePage"
  />
  <Suspense><StartupIntro v-if="introOpen && !easterEggOpen" @close="introOpen = false" /></Suspense>
  <Suspense><EasterEggGame v-if="easterEggOpen" @close="closeEasterEgg" /></Suspense>
  <PendingUpdateExitModal
    v-if="exitConfirmOpen"
    :version="softwareUpdaterState.version"
    :exiting="exiting"
    @return-install="returnToInstall"
    @exit-anyway="exitAnyway"
  />
</template>
