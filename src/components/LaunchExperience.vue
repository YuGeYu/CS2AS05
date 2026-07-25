<script setup lang="ts">
import { computed } from 'vue'
import { Play, X } from 'lucide-vue-next'

import type { PanelMode } from '@/features/panel/types'
import { LAUNCH_EXPERIENCE_DURATION_MS } from '@/composables/useCs2LaunchExperience'

const props = defineProps<{ active: boolean; elapsedMs: number; mode: PanelMode }>()
const emit = defineEmits<{ dismiss: [] }>()
const progress = computed(() => Math.min(100, (props.elapsedMs / LAUNCH_EXPERIENCE_DURATION_MS) * 100))
</script>

<template>
  <Teleport to="body">
    <section v-if="active" class="launch-experience" role="status" aria-live="polite" @click.stop="emit('dismiss')">
      <button class="icon-button launch-experience__close" type="button" aria-label="关闭启动状态" @click.stop="emit('dismiss')"><X :size="20" /></button>
      <div class="launch-experience__visual" aria-hidden="true">
        <span class="launch-experience__ring launch-experience__ring--outer" />
        <span class="launch-experience__ring launch-experience__ring--inner" />
        <span class="launch-experience__mark"><Play :size="34" fill="currentColor" /></span>
      </div>
      <div class="launch-experience__copy">
        <p class="overline">{{ mode === 'bots' ? 'BOT 模式' : '在线模式' }}</p>
        <h2>正在启动 Counter-Strike 2</h2>
      </div>
      <div class="launch-experience__progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
    </section>
  </Teleport>
</template>
