<script setup lang="ts">
import { computed, onMounted, ref, toRef, watch } from 'vue'
import { AlertCircle, CheckCircle2, FolderOpen, Play, RefreshCw, ScanSearch, Wrench } from 'lucide-vue-next'

import Cs2RootSuggestionsDialog from '@/components/Cs2RootSuggestionsDialog.vue'
import BotDifficultyWorkbench from '@/components/BotDifficultyWorkbench.vue'
import LaunchExperience from '@/components/LaunchExperience.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import { useCs2LaunchExperience } from '@/composables/useCs2LaunchExperience'
import type { Difficulty, PanelMode } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import { useDemoStore } from '@/stores/demo'

const cs2 = useCs2Store()
const panel = usePanelStore()
const demo = useDemoStore()
const suggestionsOpen = ref(false)
const botWorkbenchOpen = ref(false)
const launchExperience = useCs2LaunchExperience(toRef(cs2, 'selectedRoot'))
const modeOptions = [{ value: 'online', label: '在线模式' }, { value: 'bots', label: 'BOT 模式' }, { value: 'skin_only', label: '只开换肤' }] as const
const difficultyOptions = [{ value: 'Low', label: '低' }, { value: 'Medium', label: '中' }, { value: 'High', label: '高' }] as const
const blocked = computed(() => !cs2.selectedRoot || !panel.snapshot?.ready)
const environmentState = computed(() => {
  if (!cs2.selectedRoot) return { key: 'missing', label: '还没有选择 CS2 目录', detail: '选择游戏根目录后，助手会检查插件和受管资源。', tone: 'neutral' }
  if (cs2.cs2ProcessState === 'checking') return { key: 'checking', label: '正在检查 CS2 状态', detail: '稍等片刻，助手正在读取当前环境。', tone: 'warning' }
  if (cs2.cs2Running && !cs2.writeUnlocked) return { key: 'running', label: 'CS2 正在运行', detail: '可确认游戏已关闭后继续使用配置功能。', tone: 'warning' }
  if (cs2.environment && !cs2.environment.baseEnvironmentReady) return { key: 'install', label: '需要安装定制插件', detail: '目录已识别，但 BOT 资源尚未完整安装。', tone: 'warning' }
  if (!panel.snapshot?.ready) return { key: 'checking', label: '正在准备运行环境', detail: '资源检查完成后即可继续配置。', tone: 'warning' }
  return { key: 'ready', label: '环境已就绪，可以启动', detail: 'CS2 已退出，当前配置可以用于下一次本地对局。', tone: 'ready' }
})
const modeSummary = computed(() => panel.snapshot?.mode.current === 'bots'
  ? { label: 'BOT 模式', detail: '本地对局 · -insecure' }
  : panel.snapshot?.mode.current === 'skin_only'
    ? { label: '只开换肤', detail: '地图原生 BOT · -insecure' }
  : panel.snapshot?.mode.current === 'online'
    ? { label: '在线模式', detail: '官方启动边界' }
    : { label: '等待选择', detail: '请选择启动模式' })
const launchDisabledReason = computed(() => {
  if (!cs2.selectedRoot) return '请先选择 CS2 游戏目录'
  if (cs2.cs2Running && !cs2.writeUnlocked) return '请先退出 CS2，或确认已关闭后解锁配置功能'
  if (!panel.snapshot?.ready) return '运行环境尚未准备完成'
  return ''
})

async function browse() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const result = await open({ directory: true, multiple: false, title: '选择 CS2 游戏目录' })
  if (typeof result === 'string') {
    await cs2.selectRoot(result)
    await panel.refresh(cs2.selectedRoot, false, cs2.environment?.baseEnvironmentReady ?? false)
  }
}

function launch() {
  void launchExperience.start(panel.snapshot?.mode.current ?? 'bots')
}

const changeMode = (value: PanelMode) => panel.setMode(cs2.selectedRoot, value).catch(() => undefined)
const changeDifficulty = (value: Difficulty) => panel.setDifficulty(cs2.selectedRoot, value).catch(() => undefined)
const changeRecording = (enabled: boolean) => demo.setRecording(cs2.selectedRoot, enabled).catch(() => undefined)
const openInstall = () => window.dispatchEvent(new CustomEvent('cs2as:navigate', { detail: 'install' }))
watch(() => cs2.selectedRoot, root => void demo.loadSettings(root))
onMounted(() => void demo.loadSettings(cs2.selectedRoot))
</script>

<template>
  <section class="tool-view overview-launchpad" aria-labelledby="overview-title">
    <header class="overview-heading"><div><p class="overline">运行控制 / LOCAL SESSION</p><h1 id="overview-title">本地对局启动台</h1><p>把环境检查、启动模式和常用配置收在同一个清晰入口。</p></div><button class="icon-button" title="刷新状态" aria-label="刷新状态" @click="panel.refresh(cs2.selectedRoot)"><RefreshCw :size="18" /></button></header>
    <section class="overview-environment" :data-tone="environmentState.tone" role="status" aria-live="polite"><div class="overview-environment-icon"><CheckCircle2 v-if="environmentState.tone === 'ready'" :size="20" /><AlertCircle v-else :size="20" /></div><div class="overview-environment-copy"><span class="overview-kicker">环境状态</span><strong>{{ environmentState.label }}</strong><p>{{ environmentState.detail }}</p></div><div class="overview-environment-meta"><span v-if="cs2.selectedRoot" :title="cs2.selectedRoot">{{ cs2.selectedRoot }}</span><span v-else>等待目录</span><span v-if="cs2.closeOverride" class="overview-override-badge">玩家已确认 · 本次会话已解锁</span><button v-if="environmentState.key === 'install'" class="primary-button" type="button" @click="openInstall"><Wrench :size="16" />前往安装与诊断</button></div></section>
    <section class="overview-layout">
      <div class="overview-primary-column">
        <section class="overview-hero"><div class="overview-hero-copy"><p class="overline">主操作</p><h2>启动 Counter-Strike 2</h2><p>{{ panel.snapshot?.mode.current === 'bots' ? '将启动助手支持的本地 BOT 对局，使用 -insecure。' : panel.snapshot?.mode.current === 'skin_only' ? '只保留库存换肤，BOT 行为交还给地图和 CS2 原生逻辑。' : '将以正常在线模式启动，不修改在线模式文件。' }}</p><div class="overview-launch-summary"><span><b>模式</b>{{ modeSummary.label }} · {{ modeSummary.detail }}</span><span v-if="panel.snapshot?.mode.current === 'bots'"><b>难度</b>{{ panel.snapshot?.difficulty.current || '--' }}</span><span><b>录制</b>{{ demo.settings?.desiredEnabled ? '自动录制' : '未开启' }}</span></div></div><button class="launch-button" :disabled="blocked || cs2.cs2Running || panel.mutationKey === 'launch'" :aria-describedby="launchDisabledReason ? 'launch-disabled-reason' : undefined" @click="launch"><Play :size="21" fill="currentColor" />启动 CS2</button><p v-if="launchDisabledReason" id="launch-disabled-reason" class="launch-disabled-reason">{{ launchDisabledReason }}</p></section>
        <section class="overview-config-grid" aria-label="快速配置">
          <div class="overview-config-card overview-mode-card"><div class="overview-card-heading"><span class="overview-card-icon">01</span><div><h2>启动模式</h2><p>决定本次对局的运行边界。</p></div></div><SegmentedControl :model-value="panel.snapshot?.mode.current ?? null" :options="modeOptions" label="启动模式" :disabled="blocked || (cs2.cs2Running && !cs2.writeUnlocked)" :pending="panel.mutationKey === 'mode'" @update:model-value="changeMode" /></div>
          <div class="overview-config-card overview-difficulty-card"><div class="overview-card-heading"><span class="overview-card-icon">02</span><div><h2>BOT 难度</h2><p>{{ panel.snapshot?.mode.current === 'skin_only' ? '只开换肤时不接管 BOT 难度。' : '选择内置强度，或进入自定义工坊。' }}</p></div></div><div class="difficulty-actions"><SegmentedControl :model-value="panel.snapshot?.difficulty.current ?? null" :options="difficultyOptions" label="BOT 难度" :disabled="blocked || panel.snapshot?.mode.current === 'skin_only'" :pending="panel.mutationKey === 'difficulty'" @update:model-value="changeDifficulty" /><button class="secondary-button" type="button" :disabled="!cs2.selectedRoot || panel.snapshot?.mode.current === 'skin_only'" @click="botWorkbenchOpen = true">自定义强度</button></div></div>
          <div class="overview-config-card overview-recording-card"><div class="overview-card-heading"><span class="overview-card-icon">03</span><div><h2>本地对局记录</h2><p>只记录助手启动的本地对局。</p></div></div><ToggleSwitch :model-value="demo.settings?.desiredEnabled ?? false" label="自动录制本地对局 Demo" description="退出 CS2 后可在对局复盘中扫描战报。" :disabled="blocked || (cs2.cs2Running && !cs2.writeUnlocked) || demo.busy === 'recording'" @update:model-value="changeRecording" /><p v-if="demo.settings?.drifted" class="warning-note">配置已漂移，切换开关可重新应用。</p></div>
        </section>
      </div>
      <aside class="overview-side-panel" aria-label="环境操作"><div class="overview-side-heading"><span class="overview-kicker">ENVIRONMENT</span><h2>游戏目录</h2><p>选择根目录、`game` 或 `game/csgo` 目录。</p></div><div class="overview-path" :title="cs2.selectedRoot || undefined"><FolderOpen :size="17" /><span>{{ cs2.selectedRoot || '尚未选择 CS2 目录' }}</span></div><div class="overview-side-actions"><button class="secondary-button" type="button" @click="suggestionsOpen = true"><ScanSearch :size="17" />猜你想选</button><button class="secondary-button" type="button" @click="browse"><FolderOpen :size="17" />选择目录</button></div><div class="overview-side-facts"><div><span>游戏进程</span><strong>{{ cs2.cs2Running ? '运行中' : cs2.cs2ProcessState === 'checking' ? '检查中' : '已退出' }}</strong></div><div><span>定制资源</span><strong>{{ cs2.environment?.baseEnvironmentReady ? '已就绪' : cs2.selectedRoot ? '待安装' : '--' }}</strong></div><div><span>当前配置</span><strong>{{ panel.snapshot?.mode.current === 'bots' ? 'BOT' : panel.snapshot?.mode.current === 'skin_only' ? '只开换肤' : panel.snapshot?.mode.current === 'online' ? '在线' : '--' }}</strong></div></div></aside>
    </section>
    <p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p>
  </section>
  <Cs2RootSuggestionsDialog :open="suggestionsOpen" @close="suggestionsOpen = false" @browse="browse" />
  <BotDifficultyWorkbench :open="botWorkbenchOpen" :root-path="cs2.selectedRoot" @close="botWorkbenchOpen = false" />
  <LaunchExperience :active="launchExperience.active.value" :elapsed-ms="launchExperience.elapsedMs.value" :mode="launchExperience.mode.value" @dismiss="launchExperience.dismiss" />
</template>
