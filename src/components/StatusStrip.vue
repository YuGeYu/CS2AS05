<script setup lang="ts">
import { computed } from 'vue'
import { AlertTriangle, CheckCircle2, CircleStop, HardDrive } from 'lucide-vue-next'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store()
const panel = usePanelStore()
const missing = computed(() => panel.snapshot?.missingFiles.length ?? 0)
const processLabel = computed(() => ({ checking: '检测中', running: '运行中', stopped: '未运行', unknown: '检测失败' })[cs2.cs2ProcessState])
</script>

<template>
  <div class="status-strip" aria-label="全局状态">
    <span class="status-path" :title="cs2.selectedRoot"><HardDrive :size="14" />{{ cs2.selectedRoot || '尚未选择 CS2 目录' }}</span>
    <span :data-tone="cs2.cs2ProcessState === 'running' ? 'danger' : cs2.cs2ProcessState === 'unknown' ? 'warning' : 'success'">
      <CircleStop :size="14" /> CS2 {{ processLabel }}
    </span>
    <span :data-tone="panel.snapshot?.ready ? 'success' : 'warning'">
      <CheckCircle2 v-if="panel.snapshot?.ready" :size="14" /><AlertTriangle v-else :size="14" />
      {{ panel.snapshot?.ready ? 'Panel 可用' : missing ? `缺少 ${missing} 项` : '等待检查' }}
    </span>
    <span v-if="panel.pendingRestart.size" data-tone="warning"><AlertTriangle :size="14" /> 待重启</span>
  </div>
</template>
