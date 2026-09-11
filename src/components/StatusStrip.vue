<script setup lang="ts">
import { computed, ref } from 'vue'
import { AlertTriangle, CheckCircle2, CircleStop, HardDrive, Power } from 'lucide-vue-next'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store()
const panel = usePanelStore()
const missing = computed(() => panel.snapshot?.missingFiles.length ?? 0)
const processLabel = computed(() => ({ checking: '检测中', running: '运行中', stopped: '未运行', unknown: '检测失败' })[cs2.cs2ProcessState])
const forceConfirm = ref(false)
const manualConfirm = ref(false)
async function shutdown() {
  if (cs2.cs2ProcessState !== 'running' || cs2.closing) return
  const result = await cs2.shutdown(false)
  if (!result?.success && cs2.cs2ProcessState === 'running') forceConfirm.value = true
}
async function forceShutdown() {
  forceConfirm.value = false
  await cs2.shutdown(true)
}
async function confirmClosed() {
  manualConfirm.value = false
  await cs2.confirmClosedByPlayer()
}
</script>

<template>
  <div class="status-strip" aria-label="全局状态">
    <span class="status-path" :title="cs2.selectedRoot"><HardDrive :size="14" />{{ cs2.selectedRoot || '尚未选择 CS2 目录' }}</span>
    <span :data-tone="cs2.cs2ProcessState === 'running' ? 'danger' : cs2.cs2ProcessState === 'unknown' ? 'warning' : 'success'">
      <CircleStop :size="14" /> CS2 {{ processLabel }}
    </span>
    <button type="button" class="status-strip-close" :disabled="cs2.cs2ProcessState !== 'running' || cs2.closing" title="关闭 CS2" aria-label="关闭 CS2" @click="shutdown"><Power :size="14" />{{ cs2.closing ? '正在关闭' : '关闭 CS2' }}</button>
    <button v-if="cs2.cs2ProcessState === 'running' && !cs2.closeOverride" type="button" class="status-strip-confirm-trigger" @click="manualConfirm = true">确认已关闭并解锁</button>
    <span v-if="forceConfirm" class="status-strip-confirm" role="alert"><strong>CS2 未响应</strong><span>强制关闭可能丢失未保存内容。</span><button type="button" @click="forceShutdown">确认强制关闭</button><button type="button" @click="forceConfirm = false">取消</button></span>
    <span :data-tone="panel.snapshot?.ready ? 'success' : 'warning'">
      <CheckCircle2 v-if="panel.snapshot?.ready" :size="14" /><AlertTriangle v-else :size="14" />
      {{ panel.snapshot?.ready ? 'Panel 可用' : missing ? `缺少 ${missing} 项` : '等待检查' }}
    </span>
    <span v-if="panel.pendingRestart.size" data-tone="warning"><AlertTriangle :size="14" /> 待重启</span>
    <span v-if="cs2.closeOverride" data-tone="warning">已确认关闭 · 本次会话已解锁 <button type="button" class="status-strip-revoke" @click="cs2.revokeClosedConfirmation()">撤销</button></span>
  </div>
  <div v-if="manualConfirm" class="manual-close-confirm" role="dialog" aria-modal="true" aria-labelledby="manual-close-title">
    <div class="manual-close-confirm__panel"><h2 id="manual-close-title">确认 CS2 已关闭</h2><p>系统仍发现一个无法结束的 CS2 进程。如果游戏窗口、对局、服务器连接和 Steam 游戏会话都已结束，确认后将解锁本次会话内所有因 CS2 状态造成的功能锁定。</p><p>助手不会结束进程，也不会直接启动新的 CS2。若游戏实际仍在运行，配置可能要到下次启动才生效。</p><div><button type="button" class="secondary-button" @click="manualConfirm = false">返回检查</button><button type="button" class="primary-button" @click="confirmClosed">我确认 CS2 已关闭，解锁本次会话</button></div></div>
  </div>
</template>
