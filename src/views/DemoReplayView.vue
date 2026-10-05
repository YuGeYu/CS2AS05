<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { AlertTriangle, CheckCircle2, ExternalLink, Film, FolderOpen, HardDrive, LoaderCircle, PlaySquare, RefreshCw, ServerCog, ShieldCheck, Trash2, X } from 'lucide-vue-next'

import { useCs2Store } from '@/stores/cs2'
import { dispatchToast } from '@/services/toast'
import { openReferenceProject } from '@/services/tauri/support'
import { getDemoTracerStatus, installDemoTracerPlayback, openDemoTracerGui, uninstallDemoTracerPlayback } from '@/services/tauri/demotracer'
import type { DemoTracerStatus } from '@/types/demotracer'

const cs2 = useCs2Store()
const status = ref<DemoTracerStatus | null>(null)
const statusError = ref('')
const activeAction = ref<'refresh' | 'install' | 'open' | 'uninstall' | null>(null)
const uninstallConfirmOpen = ref(false)
let rootWatchStop: (() => void) | undefined

const rootPath = computed(() => cs2.selectedRoot || '')
const cs2RootLabel = computed(() => rootPath.value || '尚未选择 CS2 游戏目录')
const hostReady = computed(() => Boolean(status.value?.metamodReady && status.value?.counterStrikeSharpReady) || Boolean(cs2.environment?.metamodExists && cs2.environment?.counterstrikeSharpExists))
const resourcesReady = computed(() => status.value?.guiResourceReady !== false && status.value?.playbackResourceReady !== false)
const playbackReady = computed(() => Boolean(status.value?.playbackInstalled && hostReady.value && resourcesReady.value && !status.value?.missingFiles.length && !status.value?.hashMismatches.length))
const installDisabled = computed(() => Boolean(activeAction.value) || cs2.cs2Running || !rootPath.value || !resourcesReady.value || !hostReady.value)
const openDisabled = computed(() => Boolean(activeAction.value) || !resourcesReady.value || !status.value?.guiResourceReady)
const heroState = computed(() => {
  if (statusError.value) return { tone: 'warning', title: '正在等待桌面环境回应', detail: '网页预览无法直接访问本机 CS2；启动桌面助手后会自动读取状态。' }
  if (!rootPath.value) return { tone: 'neutral', title: '先选择 CS2 游戏目录', detail: 'DemoTracer Playback 只会写入当前目录的受管文件。' }
  if (cs2.cs2Running) return { tone: 'danger', title: 'CS2 正在运行', detail: '请先退出游戏，再安装、卸载或启动重玩组件。' }
  if (!hostReady.value) return { tone: 'warning', title: '需要先准备运行主机', detail: '请在“安装与诊断”完成 Metamod 与 CounterStrikeSharp 的基础安装。' }
  if (!resourcesReady.value) return { tone: 'danger', title: '安装包资源校验未通过', detail: '请重新安装助手，恢复 DemoTracer 官方资源。' }
  if (!playbackReady.value) return { tone: 'warning', title: 'Playback 尚未就绪', detail: '安装 Playback 后，才能在本机服务器中重演 BOT 行为。' }
  return { tone: 'ready', title: 'Demo 重玩环境已就绪', detail: '现在可以打开 DemoTracer GUI 分析录像，并导出可重演的回合。' }
})
const operationLabel = computed(() => {
  if (!activeAction.value) return ''
  return ({ refresh: '正在读取 DemoTracer 状态…', install: '正在安装并校验 Playback…', open: '正在启动 DemoTracer GUI…', uninstall: '正在安全卸载 Playback…' } as Record<NonNullable<typeof activeAction.value>, string>)[activeAction.value]
})

async function refreshStatus() {
  if (activeAction.value && activeAction.value !== 'refresh') return
  activeAction.value = 'refresh'
  statusError.value = ''
  try {
    status.value = await getDemoTracerStatus(rootPath.value || undefined)
  } catch (error) {
    statusError.value = error instanceof Error ? error.message : String(error)
  } finally {
    if (activeAction.value === 'refresh') activeAction.value = null
  }
}

async function installPlayback() {
  if (!rootPath.value || installDisabled.value) return
  activeAction.value = 'install'
  try {
    const result = await installDemoTracerPlayback(rootPath.value)
    status.value = result.status
    dispatchToast({ tone: 'ready', title: 'Playback 安装完成', message: `已安装 ${result.installedFiles.length} 项受管文件。${result.backupPath ? '原始文件已保存在独立备份目录。' : ''}` })
  } catch (error) {
    dispatchToast({ tone: 'danger', title: 'Playback 安装失败', message: error instanceof Error ? error.message : String(error) })
  } finally {
    activeAction.value = null
    await refreshStatus()
  }
}

async function openGui() {
  if (openDisabled.value) return
  activeAction.value = 'open'
  try {
    const result = await openDemoTracerGui()
    dispatchToast({ tone: 'ready', title: 'DemoTracer 已启动', message: result.message })
  } catch (error) {
    dispatchToast({ tone: 'danger', title: 'DemoTracer 启动失败', message: error instanceof Error ? error.message : String(error) })
  } finally {
    activeAction.value = null
  }
}

async function uninstallPlayback() {
  if (!rootPath.value) return
  uninstallConfirmOpen.value = false
  activeAction.value = 'uninstall'
  try {
    const result = await uninstallDemoTracerPlayback(rootPath.value)
    dispatchToast({ tone: 'ready', title: 'Playback 已安全卸载', message: result.message })
  } catch (error) {
    dispatchToast({ tone: 'danger', title: 'Playback 卸载失败', message: error instanceof Error ? error.message : String(error) })
  } finally {
    activeAction.value = null
    await refreshStatus()
  }
}

async function openUpstream() {
  try {
    await openReferenceProject('demotracer')
  } catch (error) {
    dispatchToast({ tone: 'danger', title: '上游项目打开失败', message: error instanceof Error ? error.message : String(error) })
  }
}

function goToInstall() {
  window.dispatchEvent(new CustomEvent('cs2as:navigate', { detail: 'install' }))
}

onMounted(async () => {
  await refreshStatus()
  rootWatchStop = watch(() => cs2.selectedRoot, () => void refreshStatus())
})
onBeforeUnmount(() => rootWatchStop?.())
</script>

<template>
  <section class="tool-view demo-replay-view" aria-labelledby="demo-replay-title">
    <header class="demo-replay-header">
      <div class="demo-replay-heading">
        <span class="demo-replay-mark" aria-hidden="true"><PlaySquare :size="24" /></span>
        <div>
          <p class="overline">核心工作台 / DEMO TRACER</p>
          <h1 id="demo-replay-title">Demo 重玩</h1>
          <p>把真实对局转成 BOT 可重演的本地训练场，逐回合复现移动、瞄准、投掷物与饰品状态。</p>
        </div>
      </div>
      <div class="demo-replay-header-actions">
        <span class="demo-version-chip"><span>GUI</span> v{{ status?.resourceVersion || '1.5.3' }}</span>
        <span class="demo-version-chip demo-version-chip--accent"><span>Playback</span> v{{ status?.playbackVersion || '1.5.2' }}</span>
        <button class="icon-button" type="button" title="刷新 DemoTracer 状态" aria-label="刷新 DemoTracer 状态" :disabled="Boolean(activeAction)" @click="refreshStatus"><RefreshCw :size="18" :class="{ spinning: activeAction === 'refresh' }" /></button>
      </div>
    </header>

    <section class="demo-replay-hero" :data-tone="heroState.tone" role="status" aria-live="polite">
      <div class="demo-replay-hero-icon" aria-hidden="true"><CheckCircle2 v-if="heroState.tone === 'ready'" :size="24" /><AlertTriangle v-else :size="24" /></div>
      <div class="demo-replay-hero-copy"><p class="overline">当前状态</p><h2>{{ heroState.title }}</h2><p>{{ heroState.detail }}</p></div>
      <span v-if="operationLabel" class="demo-replay-operation"><LoaderCircle :size="16" class="spinning" />{{ operationLabel }}</span>
      <span v-else class="demo-replay-hero-path" :title="cs2RootLabel"><FolderOpen :size="16" />{{ cs2RootLabel }}</span>
    </section>

    <div class="demo-replay-grid">
      <div class="demo-replay-main">
        <section class="demo-replay-card demo-replay-actions-card" aria-labelledby="demo-replay-actions-title">
          <div class="demo-replay-section-heading"><div><p class="overline">上游工作流</p><h2 id="demo-replay-actions-title">从 Demo 到 BOT 重演</h2><p>分析、回合筛选和 `.dtr` 文件生成由官方 DemoTracer GUI 完成；助手负责把 Playback 安全安装到当前 CS2。</p></div><Film :size="26" aria-hidden="true" /></div>
          <div class="demo-replay-action-row">
            <button class="primary-button demo-replay-primary-action" type="button" :disabled="openDisabled" @click="openGui"><Film :size="18" /><span>打开 DemoTracer GUI</span></button>
            <button class="secondary-button" type="button" :disabled="installDisabled" @click="installPlayback"><ShieldCheck :size="18" /><span>{{ status?.playbackInstalled ? '修复 / 覆盖 Playback' : '安装 Playback' }}</span></button>
            <button class="danger-button" type="button" :disabled="Boolean(activeAction) || !status?.playbackInstalled || !rootPath || cs2.cs2Running" @click="uninstallConfirmOpen = true"><Trash2 :size="17" /><span>安全卸载</span></button>
          </div>
          <p class="demo-replay-action-note"><HardDrive :size="15" />安装和卸载只处理 ownership 清单记录的 Playback 文件；被玩家或其他插件改动过的文件会保留。</p>
        </section>

        <section class="demo-replay-card" aria-labelledby="demo-replay-env-title">
          <div class="demo-replay-section-heading"><div><p class="overline">本机环境</p><h2 id="demo-replay-env-title">运行前检查</h2></div><ServerCog :size="24" aria-hidden="true" /></div>
          <div class="demo-replay-status-grid">
            <div class="demo-replay-status-item" :data-state="rootPath ? 'ready' : 'warn'"><span>CS2 目录</span><strong>{{ rootPath ? '已选择' : '待选择' }}</strong><small :title="cs2RootLabel">{{ cs2RootLabel }}</small></div>
            <div class="demo-replay-status-item" :data-state="cs2.cs2Running ? 'danger' : 'ready'"><span>游戏进程</span><strong>{{ cs2.cs2Running ? '运行中' : cs2.cs2ProcessState === 'checking' ? '检查中' : '已退出' }}</strong><small>安装 / 卸载期间必须退出 CS2</small></div>
            <div class="demo-replay-status-item" :data-state="hostReady ? 'ready' : 'warn'"><span>Metamod + CSS</span><strong>{{ hostReady ? '已检测' : '待准备' }}</strong><small>需要 KHook 与 CounterStrikeSharp 1.0.371+</small></div>
            <div class="demo-replay-status-item" :data-state="playbackReady ? 'ready' : 'warn'"><span>Playback</span><strong>{{ playbackReady ? '已就绪' : status?.playbackInstalled ? '需修复' : '未安装' }}</strong><small>{{ status?.installedFileCount || 0 }} 项受管文件</small></div>
          </div>
          <div v-if="status?.missingFiles.length || status?.hashMismatches.length" class="demo-replay-drift" role="alert"><AlertTriangle :size="17" /><span>检测到 {{ status.missingFiles.length + status.hashMismatches.length }} 项受管文件发生变化。重新安装会只修复 DemoTracer 清单中的文件。</span></div>
          <p v-if="statusError" class="demo-replay-error" role="alert">{{ statusError }}</p>
        </section>

        <section class="demo-replay-card demo-replay-guide-card" aria-labelledby="demo-replay-guide-title">
          <div class="demo-replay-section-heading"><div><p class="overline">重玩流程</p><h2 id="demo-replay-guide-title">四步开始</h2></div><span class="demo-replay-guide-badge">本地 Windows x64</span></div>
          <ol class="demo-replay-steps">
            <li><b>01</b><div><strong>安装 Playback</strong><span>确认 CS2 已退出，点击上方安装按钮。</span></div></li>
            <li><b>02</b><div><strong>导入并分析 Demo</strong><span>在 DemoTracer GUI 中选择 `.dem`，筛选要重演的回合。</span></div></li>
            <li><b>03</b><div><strong>生成重玩文件</strong><span>由官方 GUI 生成 `.dtr` 与 manifest，并复制它给本地服务器。</span></div></li>
            <li><b>04</b><div><strong>执行上游命令</strong><span><code>dtr_preset 0x15; dtr_go seq "&lt;manifest.json&gt;" 0</code></span></div></li>
          </ol>
        </section>
      </div>

      <aside class="demo-replay-aside" aria-label="DemoTracer 说明">
        <section class="demo-replay-card demo-replay-aside-card">
          <p class="overline">安装边界</p>
          <h2>只写 Playback 白名单</h2>
          <p>DemoTracer 与 CS2-Bot-Improver 可以并行存在。助手记录每个文件的写前摘要，卸载时按摘要恢复原文件。</p>
          <ul class="demo-replay-facts"><li><CheckCircle2 :size="15" />不会按文件名删除 Steam 官方文件</li><li><CheckCircle2 :size="15" />玩家改动过的文件会跳过并保留</li><li><CheckCircle2 :size="15" />原始备份位于 <code>cfg\cs2as05-demotracer-install-backups</code></li></ul>
        </section>
        <section class="demo-replay-card demo-replay-aside-card demo-replay-aside-card--accent">
          <p class="overline">需要的主机</p>
          <h2>Metamod / KHook / CSS</h2>
          <p>Playback 不是普通 `+playdemo` 播放器。它依赖 Windows x64 的 Metamod、KHook CounterStrikeSharp 和 ABI 22 的 dtr-controller。</p>
          <button class="secondary-button" type="button" @click="goToInstall"><FolderOpen :size="17" />前往安装与诊断</button>
        </section>
        <section class="demo-replay-card demo-replay-aside-card">
          <p class="overline">官方来源</p>
          <h2>DemoTracer 1.5.3</h2>
          <p>当前助手打包 GUI v1.5.3 与 Playback v1.5.2，并保留上游 AGPL-3.0-only 来源与第三方通知。</p>
          <button class="text-button demo-replay-link" type="button" @click="openUpstream"><ExternalLink :size="15" />打开 GitHub 上游项目</button>
        </section>
      </aside>
    </div>

    <Teleport to="body">
      <div v-if="uninstallConfirmOpen" class="modal-backdrop" role="presentation" @click.self="uninstallConfirmOpen = false">
        <section class="confirm-dialog demo-replay-confirm" role="dialog" aria-modal="true" aria-labelledby="demo-replay-uninstall-title">
          <button class="icon-button demo-replay-confirm-close" type="button" aria-label="关闭卸载确认" @click="uninstallConfirmOpen = false"><X :size="18" /></button>
          <div class="dialog-icon danger" aria-hidden="true"><Trash2 :size="22" /></div>
          <h2 id="demo-replay-uninstall-title">安全卸载 DemoTracer Playback？</h2>
          <p>只会处理本助手 ownership 清单中的 {{ status?.installedFileCount || 0 }} 项文件。改动过的文件会保留，CS2 官方文件不会按文件名删除。</p>
          <div class="dialog-actions"><button class="secondary-button" type="button" @click="uninstallConfirmOpen = false">取消</button><button class="danger-button" type="button" @click="uninstallPlayback"><Trash2 :size="17" />确认卸载</button></div>
        </section>
      </div>
    </Teleport>
  </section>
</template>

<style scoped>
.demo-replay-view { display: grid; gap: 18px; padding: clamp(18px, 3vw, 34px); }
.demo-replay-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 24px; }
.demo-replay-heading { display: flex; min-width: 0; align-items: flex-start; gap: 14px; }
.demo-replay-mark { display: grid; width: 48px; height: 48px; flex: 0 0 48px; place-items: center; border: 1px solid color-mix(in srgb, var(--primary) 35%, var(--border)); border-radius: 13px; color: var(--primary); background: color-mix(in srgb, var(--primary) 11%, var(--surface)); box-shadow: 0 12px 28px color-mix(in srgb, var(--primary) 12%, transparent); }
.demo-replay-heading h1 { margin: 4px 0 8px; font-size: clamp(26px, 3.1vw, 38px); letter-spacing: -.02em; }
.demo-replay-heading p:not(.overline) { max-width: 720px; margin: 0; color: var(--text-muted); line-height: 1.65; }
.demo-replay-header-actions { display: flex; flex-wrap: wrap; align-items: center; justify-content: flex-end; gap: 8px; }
.demo-version-chip { display: inline-flex; align-items: center; gap: 6px; min-height: 30px; padding: 0 10px; border: 1px solid var(--border-strong); border-radius: 999px; color: var(--primary); background: var(--surface); font-family: 'Cascadia Mono', Consolas, monospace; font-size: 11px; font-weight: 800; }
.demo-version-chip span { color: var(--text-muted); font-family: 'Microsoft YaHei UI', 'Segoe UI', sans-serif; font-weight: 700; }
.demo-version-chip--accent { border-color: color-mix(in srgb, var(--bronze) 40%, var(--border)); color: var(--bronze); }
.demo-replay-hero { display: flex; align-items: center; gap: 14px; padding: 18px 20px; border: 1px solid var(--border); border-radius: var(--radius-panel); background: linear-gradient(115deg, var(--surface), color-mix(in srgb, var(--primary) 6%, var(--surface))); box-shadow: var(--panel-shadow); }
.demo-replay-hero[data-tone='ready'] { border-color: color-mix(in srgb, var(--success) 38%, var(--border)); background: linear-gradient(115deg, var(--surface), color-mix(in srgb, var(--success) 8%, var(--surface))); }
.demo-replay-hero[data-tone='danger'] { border-color: color-mix(in srgb, var(--danger) 38%, var(--border)); background: linear-gradient(115deg, var(--surface), color-mix(in srgb, var(--danger) 8%, var(--surface))); }
.demo-replay-hero-icon { display: grid; width: 42px; height: 42px; flex: 0 0 42px; place-items: center; border-radius: 11px; color: var(--warning); background: color-mix(in srgb, var(--warning) 12%, transparent); }
.demo-replay-hero[data-tone='ready'] .demo-replay-hero-icon { color: var(--success); background: color-mix(in srgb, var(--success) 14%, transparent); }
.demo-replay-hero[data-tone='danger'] .demo-replay-hero-icon { color: var(--danger); background: color-mix(in srgb, var(--danger) 13%, transparent); }
.demo-replay-hero-copy { min-width: 0; flex: 1; }
.demo-replay-hero-copy h2 { margin: 2px 0 4px; font-size: 17px; }
.demo-replay-hero-copy p:not(.overline) { margin: 0; color: var(--text-muted); line-height: 1.55; }
.demo-replay-hero-path, .demo-replay-operation { display: inline-flex; max-width: 38%; align-items: center; gap: 7px; color: var(--text-muted); font-size: 12px; }
.demo-replay-hero-path { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.demo-replay-operation { color: var(--primary); white-space: nowrap; }
.demo-replay-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(260px, 330px); gap: 18px; align-items: start; }
.demo-replay-main, .demo-replay-aside { display: grid; gap: 18px; min-width: 0; }
.demo-replay-card { padding: 22px; border: 1px solid var(--border); border-radius: var(--radius-panel); background: var(--surface); box-shadow: var(--panel-shadow); }
.demo-replay-section-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 14px; }
.demo-replay-section-heading > svg { flex: 0 0 auto; color: var(--primary); }
.demo-replay-section-heading h2 { margin: 3px 0 6px; font-size: 18px; }
.demo-replay-section-heading p:not(.overline) { max-width: 720px; margin: 0; color: var(--text-muted); line-height: 1.6; }
.demo-replay-action-row { display: flex; flex-wrap: wrap; gap: 10px; margin-top: 20px; }
.demo-replay-action-row button { min-height: 44px; }
.demo-replay-primary-action { min-width: 218px; }
.demo-replay-action-note { display: flex; align-items: flex-start; gap: 7px; margin: 14px 0 0; color: var(--text-muted); font-size: 12px; line-height: 1.55; }
.demo-replay-action-note svg { flex: 0 0 auto; margin-top: 2px; color: var(--primary); }
.demo-replay-status-grid { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 10px; margin-top: 18px; }
.demo-replay-status-item { display: grid; min-width: 0; gap: 5px; padding: 13px 14px; border: 1px solid var(--border); border-radius: var(--radius-card); background: var(--surface-raised); }
.demo-replay-status-item > span { color: var(--text-muted); font-size: 11px; font-weight: 700; }
.demo-replay-status-item strong { color: var(--warning); font-size: 14px; }
.demo-replay-status-item[data-state='ready'] strong { color: var(--success); }
.demo-replay-status-item[data-state='danger'] strong { color: var(--danger); }
.demo-replay-status-item small { overflow: hidden; color: var(--text-muted); font-size: 11px; line-height: 1.4; text-overflow: ellipsis; white-space: nowrap; }
.demo-replay-drift, .demo-replay-error { display: flex; align-items: flex-start; gap: 8px; margin: 14px 0 0; padding: 10px 12px; border: 1px solid color-mix(in srgb, var(--warning) 35%, var(--border)); border-radius: var(--radius-control); color: var(--warning); background: color-mix(in srgb, var(--warning) 9%, var(--surface)); font-size: 12px; line-height: 1.5; }
.demo-replay-error { color: var(--danger); border-color: color-mix(in srgb, var(--danger) 34%, var(--border)); background: color-mix(in srgb, var(--danger) 8%, var(--surface)); }
.demo-replay-guide-badge { padding: 5px 9px; border: 1px solid color-mix(in srgb, var(--bronze) 30%, var(--border)); border-radius: 999px; color: var(--bronze); background: var(--bronze-soft); font-size: 11px; font-weight: 800; white-space: nowrap; }
.demo-replay-steps { display: grid; gap: 0; margin: 18px 0 0; padding: 0; list-style: none; }
.demo-replay-steps li { display: grid; grid-template-columns: 34px minmax(0, 1fr); gap: 12px; padding: 13px 0; border-top: 1px solid var(--border); }
.demo-replay-steps li:first-child { border-top: 0; padding-top: 0; }
.demo-replay-steps b { color: var(--primary); font-family: 'Cascadia Mono', Consolas, monospace; font-size: 12px; }
.demo-replay-steps div { display: grid; gap: 4px; }
.demo-replay-steps strong { font-size: 13px; }
.demo-replay-steps span { color: var(--text-muted); font-size: 12px; line-height: 1.55; }
.demo-replay-steps code, .demo-replay-aside-card code { padding: 2px 5px; border-radius: 4px; color: var(--code-text); background: var(--code-bg); font-family: 'Cascadia Mono', Consolas, monospace; font-size: 11px; }
.demo-replay-aside-card { display: grid; gap: 9px; align-content: start; }
.demo-replay-aside-card h2 { margin: 0; font-size: 16px; }
.demo-replay-aside-card p:not(.overline) { margin: 0; color: var(--text-muted); line-height: 1.65; font-size: 13px; }
.demo-replay-aside-card--accent { border-color: color-mix(in srgb, var(--bronze) 35%, var(--border)); background: linear-gradient(145deg, var(--surface), var(--bronze-soft)); }
.demo-replay-facts { display: grid; gap: 9px; margin: 4px 0 0; padding: 0; list-style: none; }
.demo-replay-facts li { display: flex; align-items: flex-start; gap: 7px; color: var(--text-muted); font-size: 12px; line-height: 1.5; }
.demo-replay-facts svg { flex: 0 0 auto; margin-top: 2px; color: var(--success); }
.demo-replay-link { display: inline-flex; align-items: center; gap: 6px; width: fit-content; padding: 0; }
.demo-replay-confirm { position: relative; width: min(440px, calc(100vw - 32px)); }
.demo-replay-confirm-close { position: absolute; top: 14px; right: 14px; }
.demo-replay-confirm h2 { padding-right: 32px; }
@media (max-width: 980px) { .demo-replay-grid { grid-template-columns: 1fr; }.demo-replay-aside { grid-template-columns: repeat(3, minmax(0, 1fr)); }.demo-replay-aside-card { min-width: 0; } }
@media (max-width: 760px) { .demo-replay-view { padding: 16px; }.demo-replay-header { flex-direction: column; }.demo-replay-header-actions { justify-content: flex-start; }.demo-replay-hero { align-items: flex-start; flex-wrap: wrap; }.demo-replay-hero-path, .demo-replay-operation { max-width: 100%; width: 100%; padding-left: 56px; }.demo-replay-status-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }.demo-replay-aside { grid-template-columns: 1fr; }.demo-replay-action-row { align-items: stretch; flex-direction: column; }.demo-replay-action-row button { width: 100%; justify-content: center; }.demo-replay-section-heading { gap: 10px; }.demo-replay-card { padding: 18px; } }
@media (prefers-reduced-motion: reduce) { .demo-replay-card, .demo-replay-hero { transition: none; } }
</style>
