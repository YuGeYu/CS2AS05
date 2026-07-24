<script setup lang="ts">
import AppTitlebar from '@/components/AppTitlebar.vue'
import GlobalToast from '@/components/GlobalToast.vue'
import InstallView from '@/views/InstallView.vue'
</script>

<template>
  <div class="app-frame">
    <AppTitlebar />
    <InstallView />
  </div>
  <GlobalToast />
</template>
