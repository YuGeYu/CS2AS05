<script setup lang="ts">
import { LogOut, RotateCw } from 'lucide-vue-next'

defineProps<{ version: string; exiting?: boolean }>()
const emit = defineEmits<{ returnInstall: []; exitAnyway: [] }>()
</script>

<template>
  <Teleport to="body">
    <div class="modal-backdrop" role="presentation">
      <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="pending-update-exit-title">
        <h2 id="pending-update-exit-title">已下载的更新尚未安装</h2>
        <p>v{{ version }} 仅保存在当前程序进程中。退出后安装包不会保留，下次需要重新下载。</p>
        <div class="dialog-actions">
          <button class="secondary-button" type="button" :disabled="exiting" @click="emit('exitAnyway')"><LogOut :size="18" /><span>{{ exiting ? '正在退出...' : '仍然退出' }}</span></button>
          <button class="primary-button" type="button" :disabled="exiting" @click="emit('returnInstall')"><RotateCw :size="18" /><span>返回安装</span></button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
