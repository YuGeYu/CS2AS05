<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { ChevronLeft, ChevronRight, PackageCheck, Sparkles } from 'lucide-vue-next'
import StatusStrip from '@/components/StatusStrip.vue'
import { useCs2ProcessPolling } from '@/composables/useCs2ProcessPolling'
import { appConfig } from '@/config/app'
import { registerVersionClick, type VersionTriggerState } from '@/features/easter-egg/version-trigger'
import { dispatchToast } from '@/services/toast'
import { initializePreflight, recordInteractiveReady } from '@/services/preflight'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import { useDemoStore } from '@/stores/demo'
import { quickSupportState } from '@/services/quick-support-state'
import OverviewView from '@/views/OverviewView.vue'
import PresetsView from '@/views/PresetsView.vue'
import BotItemsView from '@/views/BotItemsView.vue'
import KnivesView from '@/views/KnivesView.vue'
import CommandsView from '@/views/CommandsView.vue'
import DemoReviewView from '@/views/DemoReviewView.vue'
import InstallView from '@/views/InstallView.vue'
import InventorySimulatorView from '@/views/InventorySimulatorView.vue'
import QuickSupportView from '@/views/QuickSupportView.vue'
import type { PostMatchReportFailed, PostMatchReportReady } from '@/types/demo'
import { NAVIGATION_GROUPS, NAV_ITEMS, type ViewKey } from '@/config/navigation'
import { useAppearancePreferences } from '@/composables/useAppearancePreferences'

const emit = defineEmits<{ openEasterEgg: [trigger: HTMLButtonElement] }>()
const current = ref<ViewKey>('overview'); const cs2 = useCs2Store(); const panel = usePanelStore(); const demo = useDemoStore(); let timer: ReturnType<typeof setInterval> | undefined; let filesystemTimer: ReturnType<typeof setTimeout> | undefined; let unlistenReport: UnlistenFn | undefined; let unlistenReportFailed: UnlistenFn | undefined; let unlistenFilesystem: UnlistenFn | undefined; let unlistenScoreboardError: UnlistenFn | undefined
const { preferences } = useAppearancePreferences()
const sidebarCollapsed = ref(false)
const handledPostMatchSessions = new Set<string>()
let versionTrigger: VersionTriggerState = { count: 0, firstClickAt: null, lastClickAt: null }
const nav = computed(() => NAV_ITEMS.filter(item => item.required === true || !preferences.hiddenSidebarItems.includes(item.key)).sort((a, b) => preferences.sidebarOrder.indexOf(a.key) - preferences.sidebarOrder.indexOf(b.key)))
const view = computed(() => ({ overview: OverviewView, presets: PresetsView, items: BotItemsView, knives: KnivesView, inventory: InventorySimulatorView, commands: CommandsView, demoReview: DemoReviewView, quickSupport: QuickSupportView, install: InstallView })[current.value])
const groupedNav = computed(() => NAVIGATION_GROUPS.map(group => ({ ...group, items: nav.value.filter(item => item.group === group.key) })).filter(group => group.items.length))
function selectView(key: ViewKey) {
  if (key === current.value) return
  current.value = key
  window.dispatchEvent(new CustomEvent('cs2as:view-changed', { detail: key }))
}
function onVersionClick(event: MouseEvent) {
  if (event.button !== 0) return
  const result = registerVersionClick(versionTrigger, appConfig.appVersion, performance.now())
  versionTrigger = { count: result.count, firstClickAt: result.firstClickAt, lastClickAt: result.lastClickAt }
  dispatchToast({ tone: 'info', title: '提示', message: '别点我', durationMs: 1_400 })
  if (result.unlocked) emit('openEasterEgg', event.currentTarget as HTMLButtonElement)
}
function visibleRefresh() { if (!document.hidden && current.value !== 'commands' && current.value !== 'install') void panel.refresh(cs2.selectedRoot, true, cs2.environment?.baseEnvironmentReady ?? false) }
function visibilityChanged() { if (!document.hidden) visibleRefresh() }
function navigate(event: Event) { const key = (event as CustomEvent<ViewKey>).detail; if (nav.value.some(item => item.key === key)) selectView(key) }
async function openScoreboard(reportId: number) { try { await invoke('open_scoreboard', { reportId }) } catch (error) { dispatchToast({ tone: 'danger', title: '战报打开失败', message: String(error) }) } }
function handlePostMatchReady(payload: PostMatchReportReady) {
  if (handledPostMatchSessions.has(payload.sessionId)) return
  handledPostMatchSessions.add(payload.sessionId)
  void openScoreboard(payload.reportId)
}
function handlePostMatchFailed(payload: PostMatchReportFailed) {
  if (handledPostMatchSessions.has(payload.sessionId)) return
  handledPostMatchSessions.add(payload.sessionId)
  dispatchToast({ tone: 'warn', title: '最新战报未显示', message: payload.message })
}
watch(() => cs2.selectedRoot, root => { panel.resetRoot(root); if (root) void panel.refresh(root, false, cs2.environment?.baseEnvironmentReady ?? false) })
onMounted(async () => { window.dispatchEvent(new CustomEvent('cs2as:view-changed', { detail: current.value })); window.addEventListener('cs2as:navigate', navigate); await cs2.scanRoots(); if (cs2.selectedRoot) { await cs2.selectRoot(cs2.selectedRoot); await panel.refresh(cs2.selectedRoot, false, cs2.environment?.baseEnvironmentReady ?? false) } if (isTauri()) { unlistenReport = await listen<PostMatchReportReady>('demo://report-ready', event => handlePostMatchReady(event.payload)); unlistenReportFailed = await listen<PostMatchReportFailed>('demo://report-failed', event => handlePostMatchFailed(event.payload)); unlistenScoreboardError = await listen<string>('scoreboard://boot-error', event => dispatchToast({ tone: 'danger', title: '战报加载失败', message: event.payload })); unlistenFilesystem = await listen('demo://filesystem-changed', () => { if (filesystemTimer) clearTimeout(filesystemTimer); filesystemTimer = setTimeout(() => void demo.scan(), 1_000) }); if (await initializePreflight()) { await demo.refresh(); await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))); await recordInteractiveReady({ libraryItems: demo.items.length, libraryTotal: demo.total, busy: demo.busy, mainWorkbenchRendered: Boolean(document.querySelector('.workspace-shell')) }) } } timer = setInterval(visibleRefresh, 2000); document.addEventListener('visibilitychange', visibilityChanged) })
onBeforeUnmount(() => { if (timer) clearInterval(timer); if (filesystemTimer) clearTimeout(filesystemTimer); unlistenReport?.(); unlistenReportFailed?.(); unlistenFilesystem?.(); unlistenScoreboardError?.(); window.removeEventListener('cs2as:navigate', navigate); document.removeEventListener('visibilitychange', visibilityChanged) })
useCs2ProcessPolling(cs2.refreshProcessStatus)
</script>

<template>
  <div class="workspace-shell" :data-sidebar-collapsed="sidebarCollapsed"><aside class="sidebar"><div class="sidebar-brand"><span class="sidebar-brand-mark"><PackageCheck :size="20" /></span><span class="sidebar-brand-copy">工作区导航</span><button type="button" class="version-easter-egg" :title="`版本 ${appConfig.appVersion}`" :aria-label="`版本 ${appConfig.appVersion}`" @click="onVersionClick">{{ appConfig.appVersion }}</button><button type="button" class="sidebar-collapse-button" :title="sidebarCollapsed ? '展开导航' : '收起导航'" :aria-label="sidebarCollapsed ? '展开导航' : '收起导航'" :aria-expanded="!sidebarCollapsed" @click="sidebarCollapsed = !sidebarCollapsed"><ChevronRight v-if="sidebarCollapsed" :size="16" /><ChevronLeft v-else :size="16" /></button></div><nav aria-label="主导航"><section v-for="group in groupedNav" :key="group.key" class="nav-group"><h2>{{ group.label }}</h2><button v-for="item in group.items" :key="item.key" type="button" :title="item.label" :aria-label="item.label" :aria-current="current === item.key ? 'page' : undefined" :class="{ 'is-active': current === item.key }" @click="selectView(item.key)"><component :is="item.icon" :size="18" /><span>{{ item.label }}</span></button></section></nav></aside><div class="workspace-main"><StatusStrip /><main class="view-container" id="main-content" :data-current-view="current" tabindex="-1"><Transition name="view-swap" mode="out-in"><div :key="current" class="view-swap-frame"><component :is="view" /></div></Transition></main></div></div>
  <button v-if="quickSupportState.busy" class="quick-floating-orb is-busy" type="button" title="停止快快客服" aria-label="停止快快客服" @click="quickSupportState.stop?.()"><Sparkles :size="21" /><span class="quick-floating-status">{{ quickSupportState.status }}</span></button>
</template>
