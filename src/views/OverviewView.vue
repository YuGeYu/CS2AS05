<script setup lang="ts">
import { computed, onMounted, ref, toRef, watch } from 'vue'
import { FolderOpen, Play, RefreshCw, ScanSearch, Wrench } from 'lucide-vue-next'

import Cs2RootSuggestionsDialog from '@/components/Cs2RootSuggestionsDialog.vue'
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
const launchExperience = useCs2LaunchExperience(toRef(cs2, 'selectedRoot'))
const modeOptions = [{ value: 'online', label: '在线模式' }, { value: 'bots', label: 'BOT 模式' }] as const
const difficultyOptions = [{ value: 'Low', label: '低' }, { value: 'Medium', label: '中' }, { value: 'High', label: '高' }] as const
const blocked = computed(() => !cs2.selectedRoot || !panel.snapshot?.ready)

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
  <section class="tool-view" aria-labelledby="overview-title">
    <header class="view-heading"><div><p class="overline">运行控制</p><h1 id="overview-title">概览</h1></div><button class="icon-button" title="刷新状态" aria-label="刷新状态" @click="panel.refresh(cs2.selectedRoot)"><RefreshCw :size="18" /></button></header>
    <section class="control-band">
      <div class="field-heading"><div><h2>CS2 游戏目录</h2><p :title="cs2.selectedRoot">{{ cs2.selectedRoot || '选择游戏根目录、game 或 game/csgo 目录。' }}</p></div><div class="directory-actions"><button class="secondary-button" type="button" @click="suggestionsOpen = true"><ScanSearch :size="18" />猜你想选</button><button class="secondary-button" type="button" @click="browse"><FolderOpen :size="18" />选择目录</button></div></div>
    </section>
    <section v-if="cs2.selectedRoot && cs2.environment && !cs2.environment.baseEnvironmentReady" class="environment-recovery" role="status">
      <Wrench :size="20" aria-hidden="true" />
      <div><strong>插件尚未安装</strong><span :title="cs2.selectedRoot">已识别 CS2：{{ cs2.selectedRoot }}</span></div>
      <button class="primary-button" type="button" @click="openInstall">前往安装与诊断</button>
    </section>
    <section class="control-grid">
      <div class="control-group"><div><h2>启动模式</h2><p>BOT 模式会加载 Metamod 并使用 -insecure。</p></div><SegmentedControl :model-value="panel.snapshot?.mode.current ?? null" :options="modeOptions" label="启动模式" :disabled="blocked || cs2.cs2Running" :pending="panel.mutationKey === 'mode'" @update:model-value="changeMode" /></div>
      <div class="control-group"><div><h2>BOT 难度</h2><p>切换当前使用的 botprofile.vpk。</p></div><SegmentedControl :model-value="panel.snapshot?.difficulty.current ?? null" :options="difficultyOptions" label="BOT 难度" :disabled="blocked" :pending="panel.mutationKey === 'difficulty'" @update:model-value="changeDifficulty" /></div>
      <div class="control-group demo-recording-control"><div><h2>本地对局记录</h2><p>官方匹配是否提供 Demo 由服务器或平台决定。</p></div><ToggleSwitch :model-value="demo.settings?.desiredEnabled ?? false" label="自动录制本地对局 Demo" description="助手启动 BOT/本地对局前写入 CSTV 自动录制设置；CS2 退出后可在对局复盘中扫描战报。" :disabled="blocked || cs2.cs2Running || demo.busy === 'recording'" @update:model-value="changeRecording" /><p v-if="demo.settings?.drifted" class="warning-note">配置已漂移，切换开关可重新应用到两个受管 cfg。</p></div>
    </section>
    <section class="launch-band"><div><p class="overline">主操作</p><h2>启动 Counter-Strike 2</h2><p>{{ panel.snapshot?.mode.current === 'bots' ? '将以 -insecure -console -condebug 启动。' : '将以正常在线模式启动。' }}</p></div><button class="launch-button" :disabled="blocked || cs2.cs2Running || panel.mutationKey === 'launch'" @click="launch"><Play :size="21" fill="currentColor" />启动 CS2</button></section>
    <p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p>
  </section>
  <Cs2RootSuggestionsDialog :open="suggestionsOpen" @close="suggestionsOpen = false" @browse="browse" />
  <LaunchExperience :active="launchExperience.active.value" :elapsed-ms="launchExperience.elapsedMs.value" :mode="launchExperience.mode.value" @dismiss="launchExperience.dismiss" />
</template>
