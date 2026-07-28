<script setup lang="ts">
import { defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow, type CloseRequestedEvent } from '@tauri-apps/api/window'
import AppTitlebar from '@/components/AppTitlebar.vue'
import AppShell from '@/components/AppShell.vue'
import GlobalToast from '@/components/GlobalToast.vue'
import PendingUpdateExitModal from '@/components/PendingUpdateExitModal.vue'
import SoftwareUpdateModal from '@/components/SoftwareUpdateModal.vue'
import { closeSoftwareUpdate, deferSoftwareUpdateInstall, installSoftwareUpdate, openSoftwareUpdateDownload, openSoftwareUpdateReleasePage, showPendingSoftwareUpdate, softwareUpdateCoordinatorState, startSoftwareUpdateCoordinator, startSoftwareUpdateDownload } from '@/features/software-updates/coordinator'
import { hasPendingDownloadedUpdate, prepareDeferredUpdateForExit, softwareUpdaterState } from '@/features/software-updates/updater-state'

const exitConfirmOpen = ref(false)
const exiting = ref(false)
const introOpen = ref(true)
const easterEggOpen = ref(false)
let easterEggTrigger: HTMLButtonElement | null = null
let unlistenClose: (() => void) | undefined
const StartupIntro = defineAsyncComponent(() => import('@/components/intro/StartupIntro.vue'))
const EasterEggGame = defineAsyncComponent(() => import('@/components/easter-egg/EasterEggGame.vue'))

async function onCloseRequested(event: CloseRequestedEvent) {
  if (!hasPendingDownloadedUpdate()) return
  event.preventDefault()
  exitConfirmOpen.value = true
}

function returnToInstall() {
  exitConfirmOpen.value = false
  window.dispatchEvent(new CustomEvent('cs2as:show-pending-update'))
}

async function exitAnyway() {
  exiting.value = true
  await prepareDeferredUpdateForExit()
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
