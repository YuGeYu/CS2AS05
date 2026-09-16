<script setup lang="ts">
import { computed, ref } from 'vue'
import { AlertTriangle, CheckCircle2, CircleStop, HardDrive, LifeBuoy, Power } from 'lucide-vue-next'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store()
const panel = usePanelStore()
const missing = computed(() => panel.snapshot?.missingFiles.length ?? 0)
const processLabel = computed(() => ({ checking: '检测中', running: '运行中', stopped: '未运行', unknown: '检测失败' })[cs2.cs2ProcessState])
const manualConfirm = ref(false)
async function shutdown() {
  if (cs2.cs2ProcessState !== 'running' || cs2.closing) return
  let result: Awaited<ReturnType<typeof cs2.shutdown>> | null = null
  try {
    result = await cs2.shutdown(false)
  } catch {
    // Keep the player path available even when the close command itself fails.
  }
  if (!result?.success && (cs2.cs2ProcessState === 'running' || cs2.cs2ProcessState === 'unknown')) {
    let forced: Awaited<ReturnType<typeof cs2.shutdown>> | null = null
    try {
      forced = await cs2.shutdown(true)
    } catch {
      // The manual confirmation entry is the final recovery path.
    }
    if (!forced?.success && ['running', 'unknown'].includes(cs2.cs2ProcessState)) manualConfirm.value = true
  }
}
async function confirmClosed() {
  try {
    await cs2.confirmClosedByPlayer()
    manualConfirm.value = false
  } catch {
    // The global toast carries the actionable error; keep the dialog open.
  }
}
</script>

<template>
  <div class="status-strip" aria-label="全局状态">
    <span class="status-path" :title="cs2.selectedRoot"><HardDrive :size="14" />{{ cs2.selectedRoot || '尚未选择 CS2 目录' }}</span>
    <span :data-tone="cs2.cs2ProcessState === 'running' ? 'danger' : cs2.cs2ProcessState === 'unknown' ? 'warning' : 'success'">
      <CircleStop :size="14" /> CS2 {{ processLabel }}
    </span>
    <button type="button" class="status-strip-close" :disabled="cs2.cs2ProcessState !== 'running' || cs2.closing" title="关闭 CS2" aria-label="关闭 CS2" @click="shutdown"><Power :size="14" />{{ cs2.closing ? '正在关闭' : '关闭 CS2' }}</button>
    <span v-if="cs2.closing" class="status-strip-confirm" role="status">正在尝试关闭 CS2；超过 2 秒会自动执行强制关闭。</span>
    <button type="button" class="status-strip-confirm-trigger" title="打开人工确认并解锁助手" aria-label="人工确认 CS2 已关闭" @click="manualConfirm = true"><LifeBuoy :size="14" />人工确认已关闭</button>
    <span :data-tone="panel.snapshot?.ready ? 'success' : 'warning'">
      <CheckCircle2 v-if="panel.snapshot?.ready" :size="14" /><AlertTriangle v-else :size="14" />
      {{ panel.snapshot?.ready ? 'Panel 可用' : missing ? `缺少 ${missing} 项` : '等待检查' }}
    </span>
    <span v-if="panel.pendingRestart.size" data-tone="warning"><AlertTriangle :size="14" /> 待重启</span>
    <span v-if="cs2.closeOverride" data-tone="warning">已确认关闭 · 本次会话已解锁 <button type="button" class="status-strip-revoke" @click="cs2.revokeClosedConfirmation()">撤销</button></span>
  </div>
  <div v-if="manualConfirm" class="manual-close-confirm" role="dialog" aria-modal="true" aria-labelledby="manual-close-title">
    <div class="manual-close-confirm__panel"><h2 id="manual-close-title">确认 CS2 已关闭</h2><p>这是给玩家保留的人工解锁入口。请确认 CS2 窗口、对局、服务器连接和 Steam 游戏会话都已结束；确认后会立即解锁助手内所有因 CS2 状态造成的功能锁定，包括“启动 CS2”。</p><p>如果系统仍报告 CS2 正在运行，助手不会替你结束未知状态的进程；请先确认游戏确实已经退出，再执行解锁。</p><div><button type="button" class="secondary-button" @click="manualConfirm = false">返回检查</button><button type="button" class="primary-button" @click="confirmClosed">我确认 CS2 已关闭，解锁全部功能</button></div></div>
  </div>
</template>
