<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Bot, Boxes, Gauge, PackageCheck, ScrollText, Settings, Sword } from 'lucide-vue-next'
import StatusStrip from '@/components/StatusStrip.vue'
import { useCs2ProcessPolling } from '@/composables/useCs2ProcessPolling'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import OverviewView from '@/views/OverviewView.vue'
import PresetsView from '@/views/PresetsView.vue'
import BotItemsView from '@/views/BotItemsView.vue'
import KnivesView from '@/views/KnivesView.vue'
import CommandsView from '@/views/CommandsView.vue'
import InstallView from '@/views/InstallView.vue'

type ViewKey = 'overview' | 'presets' | 'items' | 'knives' | 'commands' | 'install'
const current = ref<ViewKey>('overview'); const cs2 = useCs2Store(); const panel = usePanelStore(); let timer: ReturnType<typeof setInterval> | undefined
const nav = [
  { key: 'overview', label: '概览', icon: Gauge }, { key: 'presets', label: '人机预设', icon: Bot },
  { key: 'items', label: 'Bot 物品', icon: Boxes }, { key: 'knives', label: '刀具', icon: Sword },
  { key: 'commands', label: '命令', icon: ScrollText }, { key: 'install', label: '安装与诊断', icon: Settings },
] as const
const view = computed(() => ({ overview: OverviewView, presets: PresetsView, items: BotItemsView, knives: KnivesView, commands: CommandsView, install: InstallView })[current.value])
const activeNavIndex = computed(() => nav.findIndex(item => item.key === current.value))
function selectView(key: ViewKey) { if (key !== current.value) current.value = key }
function visibleRefresh() { if (!document.hidden && current.value !== 'commands' && current.value !== 'install') void panel.refresh(cs2.selectedRoot, true) }
function visibilityChanged() { if (!document.hidden) visibleRefresh() }
watch(() => cs2.selectedRoot, root => { panel.resetRoot(root); if (root) void panel.refresh(root) })
onMounted(async () => { await cs2.scanRoots(); if (cs2.selectedRoot) { await cs2.selectRoot(cs2.selectedRoot); await panel.refresh(cs2.selectedRoot) } timer = setInterval(visibleRefresh, 2000); document.addEventListener('visibilitychange', visibilityChanged) })
onBeforeUnmount(() => { if (timer) clearInterval(timer); document.removeEventListener('visibilitychange', visibilityChanged) })
useCs2ProcessPolling(cs2.refreshProcessStatus)
</script>

<template><div class="workspace-shell"><aside class="sidebar"><div class="sidebar-brand"><span><PackageCheck :size="20" /></span><div><strong>CS2 助手</strong><small>0.5.4</small></div></div><nav aria-label="主导航" :style="{ '--nav-index': activeNavIndex }"><span class="nav-cursor" aria-hidden="true" /><button v-for="item in nav" :key="item.key" type="button" :title="item.label" :aria-label="item.label" :aria-current="current === item.key ? 'page' : undefined" @click="selectView(item.key)"><component :is="item.icon" :size="18" /><span>{{ item.label }}</span></button></nav></aside><div class="workspace-main"><StatusStrip /><main class="view-container" :data-current-view="current"><Transition name="view-swap" mode="out-in"><div :key="current" class="view-swap-frame"><component :is="view" /></div></Transition></main></div></div></template>
