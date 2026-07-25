<script setup lang="ts">
import AppTitlebar from '@/components/AppTitlebar.vue'
import AppShell from '@/components/AppShell.vue'
import GlobalToast from '@/components/GlobalToast.vue'
</script>

<template>
  <div class="app-frame">
    <AppTitlebar />
    <AppShell />
  </div>
  <GlobalToast />
</template>
