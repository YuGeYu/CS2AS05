<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Bot, Boxes, ChartNoAxesCombined, Gauge, PackageCheck, ScrollText, Settings, Sword } from 'lucide-vue-next'
import StatusStrip from '@/components/StatusStrip.vue'
import { useCs2ProcessPolling } from '@/composables/useCs2ProcessPolling'
import { appConfig } from '@/config/app'
import { registerVersionClick, type VersionTriggerState } from '@/features/easter-egg/version-trigger'
import { dispatchToast } from '@/services/toast'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import { useDemoStore } from '@/stores/demo'
import OverviewView from '@/views/OverviewView.vue'
import PresetsView from '@/views/PresetsView.vue'
import BotItemsView from '@/views/BotItemsView.vue'
import KnivesView from '@/views/KnivesView.vue'
import CommandsView from '@/views/CommandsView.vue'
import DemoReviewView from '@/views/DemoReviewView.vue'
import InstallView from '@/views/InstallView.vue'

type ViewKey = 'overview' | 'presets' | 'items' | 'knives' | 'commands' | 'demoReview' | 'install'
const emit = defineEmits<{ openEasterEgg: [trigger: HTMLButtonElement] }>()
const current = ref<ViewKey>('overview'); const cs2 = useCs2Store(); const panel = usePanelStore(); const demo = useDemoStore(); let timer: ReturnType<typeof setInterval> | undefined; let filesystemTimer: ReturnType<typeof setTimeout> | undefined; let unlistenReport: UnlistenFn | undefined; let unlistenFilesystem: UnlistenFn | undefined
let versionTrigger: VersionTriggerState = { count: 0, firstClickAt: null, lastClickAt: null }
const nav = [
  { key: 'overview', label: '概览', icon: Gauge }, { key: 'presets', label: '人机预设', icon: Bot },
  { key: 'items', label: 'Bot 物品', icon: Boxes }, { key: 'knives', label: '刀具', icon: Sword },
  { key: 'commands', label: '命令', icon: ScrollText }, { key: 'demoReview', label: '对局复盘', icon: ChartNoAxesCombined }, { key: 'install', label: '安装与诊断', icon: Settings },
] as const
const view = computed(() => ({ overview: OverviewView, presets: PresetsView, items: BotItemsView, knives: KnivesView, commands: CommandsView, demoReview: DemoReviewView, install: InstallView })[current.value])
const activeNavIndex = computed(() => nav.findIndex(item => item.key === current.value))
function selectView(key: ViewKey) { if (key !== current.value) current.value = key }
function onVersionClick(event: MouseEvent) {
  if (event.button !== 0) return
  const result = registerVersionClick(versionTrigger, appConfig.appVersion, performance.now())
  versionTrigger = { count: result.count, firstClickAt: result.firstClickAt, lastClickAt: result.lastClickAt }
  dispatchToast({ tone: 'info', title: '提示', message: '别点我', durationMs: 1_400 })
  if (result.unlocked) emit('openEasterEgg', event.currentTarget as HTMLButtonElement)
}
function visibleRefresh() { if (!document.hidden && current.value !== 'commands' && current.value !== 'install') void panel.refresh(cs2.selectedRoot, true) }
function visibilityChanged() { if (!document.hidden) visibleRefresh() }
watch(() => cs2.selectedRoot, root => { panel.resetRoot(root); if (root) void panel.refresh(root) })
onMounted(async () => { await cs2.scanRoots(); if (cs2.selectedRoot) { await cs2.selectRoot(cs2.selectedRoot); await panel.refresh(cs2.selectedRoot) } if (isTauri()) { unlistenReport = await listen<number>('demo://report-ready', async event => { await demo.openReport(event.payload); if (demo.report) current.value = 'demoReview' }); unlistenFilesystem = await listen('demo://filesystem-changed', () => { if (filesystemTimer) clearTimeout(filesystemTimer); filesystemTimer = setTimeout(() => void demo.scan(), 1_000) }) } timer = setInterval(visibleRefresh, 2000); document.addEventListener('visibilitychange', visibilityChanged) })
onBeforeUnmount(() => { if (timer) clearInterval(timer); if (filesystemTimer) clearTimeout(filesystemTimer); unlistenReport?.(); unlistenFilesystem?.(); document.removeEventListener('visibilitychange', visibilityChanged) })
useCs2ProcessPolling(cs2.refreshProcessStatus)
</script>

<template><div class="workspace-shell"><aside class="sidebar"><div class="sidebar-brand"><span><PackageCheck :size="20" /></span><div><strong>人机增强</strong><small>玄铁机括</small></div><button type="button" class="version-easter-egg" :title="`版本 ${appConfig.appVersion}`" :aria-label="`版本 ${appConfig.appVersion}`" @click="onVersionClick">{{ appConfig.appVersion }}</button></div><nav aria-label="主导航" :style="{ '--nav-index': activeNavIndex }"><span class="nav-cursor" aria-hidden="true" /><button v-for="item in nav" :key="item.key" type="button" :title="item.label" :aria-label="item.label" :aria-current="current === item.key ? 'page' : undefined" @click="selectView(item.key)"><component :is="item.icon" :size="18" /><span>{{ item.label }}</span></button></nav></aside><div class="workspace-main"><StatusStrip /><main class="view-container" :data-current-view="current" tabindex="-1"><Transition name="view-swap" mode="out-in"><div :key="current" class="view-swap-frame"><component :is="view" /></div></Transition></main></div></div></template>
