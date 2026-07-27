<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { getCurrentWindow, type CloseRequestedEvent } from '@tauri-apps/api/window'
import AppTitlebar from '@/components/AppTitlebar.vue'
import AppShell from '@/components/AppShell.vue'
import GlobalToast from '@/components/GlobalToast.vue'
import PendingUpdateExitModal from '@/components/PendingUpdateExitModal.vue'
import { hasPendingDownloadedUpdate, prepareDeferredUpdateForExit, softwareUpdaterState } from '@/features/software-updates/updater-state'

const exitConfirmOpen = ref(false)
const exiting = ref(false)
let unlistenClose: (() => void) | undefined

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

onMounted(async () => {
  if (isTauri()) unlistenClose = await getCurrentWindow().onCloseRequested(onCloseRequested)
})
onBeforeUnmount(() => unlistenClose?.())
</script>

<template>
  <div class="app-frame">
    <AppTitlebar />
    <AppShell />
  </div>
  <GlobalToast />
  <PendingUpdateExitModal
    v-if="exitConfirmOpen"
    :version="softwareUpdaterState.version"
    :exiting="exiting"
    @return-install="returnToInstall"
    @exit-anyway="exitAnyway"
  />
</template>
