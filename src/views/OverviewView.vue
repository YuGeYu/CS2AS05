<script setup lang="ts">
import { computed, ref, toRef } from 'vue'
import { FolderOpen, Play, RefreshCw, ScanSearch } from 'lucide-vue-next'

import Cs2RootSuggestionsDialog from '@/components/Cs2RootSuggestionsDialog.vue'
import LaunchExperience from '@/components/LaunchExperience.vue'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import { useCs2LaunchExperience } from '@/composables/useCs2LaunchExperience'
import type { Difficulty, PanelMode } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store()
const panel = usePanelStore()
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
    await panel.refresh(cs2.selectedRoot)
  }
}

function launch() {
  void launchExperience.start(panel.snapshot?.mode.current ?? 'bots')
}

const changeMode = (value: PanelMode) => panel.setMode(cs2.selectedRoot, value).catch(() => undefined)
const changeDifficulty = (value: Difficulty) => panel.setDifficulty(cs2.selectedRoot, value).catch(() => undefined)
</script>

<template>
  <section class="tool-view" aria-labelledby="overview-title">
    <header class="view-heading"><div><p class="overline">运行控制</p><h1 id="overview-title">概览</h1></div><button class="icon-button" title="刷新状态" aria-label="刷新状态" @click="panel.refresh(cs2.selectedRoot)"><RefreshCw :size="18" /></button></header>
    <section class="control-band">
      <div class="field-heading"><div><h2>CS2 游戏目录</h2><p :title="cs2.selectedRoot">{{ cs2.selectedRoot || '选择游戏根目录、game 或 game/csgo 目录。' }}</p></div><div class="directory-actions"><button class="secondary-button" type="button" @click="suggestionsOpen = true"><ScanSearch :size="18" />猜你想选</button><button class="secondary-button" type="button" @click="browse"><FolderOpen :size="18" />选择目录</button></div></div>
    </section>
    <section class="control-grid">
      <div class="control-group"><div><h2>启动模式</h2><p>BOT 模式会加载 Metamod 并使用 -insecure。</p></div><SegmentedControl :model-value="panel.snapshot?.mode.current ?? null" :options="modeOptions" label="启动模式" :disabled="blocked || cs2.cs2Running" @update:model-value="changeMode" /></div>
      <div class="control-group"><div><h2>BOT 难度</h2><p>切换当前使用的 botprofile.vpk。</p></div><SegmentedControl :model-value="panel.snapshot?.difficulty.current ?? null" :options="difficultyOptions" label="BOT 难度" :disabled="blocked" @update:model-value="changeDifficulty" /></div>
    </section>
    <section class="launch-band"><div><p class="overline">主操作</p><h2>启动 Counter-Strike 2</h2><p>{{ panel.snapshot?.mode.current === 'bots' ? '将以 -insecure -console -condebug 启动。' : '将以正常在线模式启动。' }}</p></div><button class="launch-button" :disabled="blocked || cs2.cs2Running || panel.mutationKey === 'launch'" @click="launch"><Play :size="21" fill="currentColor" />启动 CS2</button></section>
    <p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p>
  </section>
  <Cs2RootSuggestionsDialog :open="suggestionsOpen" @close="suggestionsOpen = false" @browse="browse" />
  <LaunchExperience :active="launchExperience.active.value" :elapsed-ms="launchExperience.elapsedMs.value" :mode="launchExperience.mode.value" @dismiss="launchExperience.dismiss" />
</template>
