<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { Bot, Boxes, ChartNoAxesCombined, Gauge, LoaderCircle, PackageCheck, Palette, Save, ScrollText, Settings, Sword } from 'lucide-vue-next'
import StatusStrip from '@/components/StatusStrip.vue'
import { useCs2ProcessPolling } from '@/composables/useCs2ProcessPolling'
import { appConfig } from '@/config/app'
import { registerVersionClick, type VersionTriggerState } from '@/features/easter-egg/version-trigger'
import { dispatchToast } from '@/services/toast'
import { initializePreflight, recordInteractiveReady } from '@/services/preflight'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import { useDemoStore } from '@/stores/demo'
import { useSkinForgeStore } from '@/stores/skinForge'
import OverviewView from '@/views/OverviewView.vue'
import PresetsView from '@/views/PresetsView.vue'
import BotItemsView from '@/views/BotItemsView.vue'
import KnivesView from '@/views/KnivesView.vue'
import CommandsView from '@/views/CommandsView.vue'
import DemoReviewView from '@/views/DemoReviewView.vue'
import InstallView from '@/views/InstallView.vue'
import SkinForgeView from '@/views/SkinForgeView.vue'
import type { PostMatchReportFailed, PostMatchReportReady } from '@/types/demo'

type ViewKey = 'overview' | 'presets' | 'items' | 'knives' | 'skinForge' | 'commands' | 'demoReview' | 'install'
const emit = defineEmits<{ openEasterEgg: [trigger: HTMLButtonElement] }>()
const current = ref<ViewKey>('overview'); const cs2 = useCs2Store(); const panel = usePanelStore(); const demo = useDemoStore(); const skinForge = useSkinForgeStore(); let timer: ReturnType<typeof setInterval> | undefined; let filesystemTimer: ReturnType<typeof setTimeout> | undefined; let unlistenReport: UnlistenFn | undefined; let unlistenReportFailed: UnlistenFn | undefined; let unlistenFilesystem: UnlistenFn | undefined; let unlistenScoreboardError: UnlistenFn | undefined
const pendingView = ref<ViewKey | null>(null)
const leaveBusy = ref<'save' | 'discard' | null>(null)
const handledPostMatchSessions = new Set<string>()
let versionTrigger: VersionTriggerState = { count: 0, firstClickAt: null, lastClickAt: null }
const nav = [
  { key: 'overview', label: '概览', icon: Gauge }, { key: 'presets', label: '人机预设', icon: Bot },
  { key: 'items', label: 'Bot 物品', icon: Boxes }, { key: 'knives', label: '刀具', icon: Sword },
  { key: 'skinForge', label: '皮肤工坊', icon: Palette }, { key: 'commands', label: '命令', icon: ScrollText }, { key: 'demoReview', label: '对局复盘', icon: ChartNoAxesCombined }, { key: 'install', label: '安装与诊断', icon: Settings },
] as const
const view = computed(() => ({ overview: OverviewView, presets: PresetsView, items: BotItemsView, knives: KnivesView, skinForge: SkinForgeView, commands: CommandsView, demoReview: DemoReviewView, install: InstallView })[current.value])
const activeNavIndex = computed(() => nav.findIndex(item => item.key === current.value))
function selectView(key: ViewKey) {
  if (key === current.value) return
  if (current.value === 'skinForge' && skinForge.dirty) { pendingView.value = key; return }
  current.value = key
}
function cancelLeave() { if (!leaveBusy.value) pendingView.value = null }
async function confirmLeave(action: 'save' | 'discard') {
  const target = pendingView.value
  if (!target || leaveBusy.value) return
  leaveBusy.value = action
  try {
    if (action === 'save') await skinForge.save()
    else await skinForge.load()
    if (action === 'discard' && skinForge.error) return
    pendingView.value = null
    current.value = target
  } catch { /* The store keeps the actionable error visible; remain in the workshop. */ }
  finally { leaveBusy.value = null }
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
function navigate(event: Event) { const key = (event as CustomEvent<ViewKey>).detail; if (nav.some(item => item.key === key)) selectView(key) }
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
onMounted(async () => { window.addEventListener('cs2as:navigate', navigate); await cs2.scanRoots(); if (cs2.selectedRoot) { await cs2.selectRoot(cs2.selectedRoot); await panel.refresh(cs2.selectedRoot, false, cs2.environment?.baseEnvironmentReady ?? false) } if (isTauri()) { unlistenReport = await listen<PostMatchReportReady>('demo://report-ready', event => handlePostMatchReady(event.payload)); unlistenReportFailed = await listen<PostMatchReportFailed>('demo://report-failed', event => handlePostMatchFailed(event.payload)); unlistenScoreboardError = await listen<string>('scoreboard://boot-error', event => dispatchToast({ tone: 'danger', title: '战报加载失败', message: event.payload })); unlistenFilesystem = await listen('demo://filesystem-changed', () => { if (filesystemTimer) clearTimeout(filesystemTimer); filesystemTimer = setTimeout(() => void demo.scan(), 1_000) }); if (await initializePreflight()) { await demo.refresh(); await new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve()))); await recordInteractiveReady({ libraryItems: demo.items.length, libraryTotal: demo.total, busy: demo.busy, mainWorkbenchRendered: Boolean(document.querySelector('.workspace-shell')) }) } } timer = setInterval(visibleRefresh, 2000); document.addEventListener('visibilitychange', visibilityChanged) })
onBeforeUnmount(() => { if (timer) clearInterval(timer); if (filesystemTimer) clearTimeout(filesystemTimer); unlistenReport?.(); unlistenReportFailed?.(); unlistenFilesystem?.(); unlistenScoreboardError?.(); window.removeEventListener('cs2as:navigate', navigate); document.removeEventListener('visibilitychange', visibilityChanged) })
useCs2ProcessPolling(cs2.refreshProcessStatus)
</script>

<template>
  <div class="workspace-shell"><aside class="sidebar"><div class="sidebar-brand"><span><PackageCheck :size="20" /></span><button type="button" class="version-easter-egg" :title="`版本 ${appConfig.appVersion}`" :aria-label="`版本 ${appConfig.appVersion}`" @click="onVersionClick">{{ appConfig.appVersion }}</button></div><nav aria-label="主导航" :style="{ '--nav-index': activeNavIndex }"><span class="nav-cursor" aria-hidden="true" /><button v-for="item in nav" :key="item.key" type="button" :title="item.label" :aria-label="item.label" :aria-current="current === item.key ? 'page' : undefined" @click="selectView(item.key)"><component :is="item.icon" :size="18" /><span>{{ item.label }}</span></button></nav></aside><div class="workspace-main"><StatusStrip /><main class="view-container" :data-current-view="current" tabindex="-1"><Transition name="view-swap" mode="out-in"><div :key="current" class="view-swap-frame"><component :is="view" /></div></Transition></main></div></div>
  <Teleport to="body">
    <div v-if="pendingView" class="modal-backdrop forge-leave-backdrop">
      <section class="confirm-dialog forge-leave-dialog" role="dialog" aria-modal="true" aria-labelledby="forge-leave-title">
        <div class="modal-heading"><Palette :size="22" /><div><p class="overline">皮肤工坊</p><h2 id="forge-leave-title">还有搭配没有应用</h2><p>应用后再离开，或者放弃本次修改并恢复游戏目录中的配置。</p></div></div>
        <div class="dialog-actions forge-leave-actions">
          <button class="secondary-button" type="button" :disabled="Boolean(leaveBusy)" @click="cancelLeave">继续编辑</button>
          <button class="secondary-button" type="button" :disabled="Boolean(leaveBusy)" @click="confirmLeave('discard')"><LoaderCircle v-if="leaveBusy === 'discard'" :size="17" class="spin" />不应用并离开</button>
          <button class="primary-button" type="button" :disabled="Boolean(leaveBusy)" autofocus @click="confirmLeave('save')"><LoaderCircle v-if="leaveBusy === 'save'" :size="17" class="spin" /><Save v-else :size="17" />应用并离开</button>
        </div>
      </section>
    </div>
  </Teleport>
</template>
