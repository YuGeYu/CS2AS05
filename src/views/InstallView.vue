<script setup lang="ts">
import { computed, ref } from 'vue'
import { FolderOpen, RefreshCw, ShieldCheck, TerminalSquare, Trash2, Wrench } from 'lucide-vue-next'

import SupportActions from '@/components/SupportActions.vue'
import { appConfig } from '@/config/app'
import { useCs2Store } from '@/stores/cs2'

const store = useCs2Store()
const uninstallConfirmOpen = ref(false)
const diagnosticsOpen = ref(false)

const installBlocked = computed(() => !store.selectedRoot || store.cs2Running || store.busy)
const installLabel = computed(() => {
  if (store.busy) return '处理中...'
  if (store.environment?.baseEnvironmentReady) return '覆盖更新定制插件包'
  return '安装定制插件包'
})
const installHint = computed(() => {
  if (!store.selectedRoot) return '选择 Counter-Strike Global Offensive 目录后才能安装。'
  if (store.cs2Running) return '检测到 CS2 正在运行，请退出游戏后再继续。'
  if (store.environment?.baseEnvironmentReady) return `已检测到插件，可直接覆盖更新到 ${appConfig.appVersion} 定制包。`
  return '将安装基于 CS2-Bot-Improver v1.4.3 的最小定制资源包。'
})
const packageState = computed(() => store.environment?.baseEnvironmentReady ? '已安装' : '待安装')
const processLabel = computed(() => ({
  checking: '检测中',
  running: '运行中',
  stopped: '未运行',
  unknown: '检测失败',
}[store.cs2ProcessState]))
const processVisualState = computed(() => ({
  checking: 'warn',
  running: 'danger',
  stopped: 'ready',
  unknown: 'warn',
}[store.cs2ProcessState]))

async function browse() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const result = await open({ directory: true, multiple: false, title: '选择 CS2 游戏目录' })
  if (typeof result === 'string') await store.selectRoot(result)
}

async function install() {
  try {
    await store.install()
  } catch {
    // The store exposes the actionable failure in the toast and diagnostics.
  }
}

async function openPanel() {
  try {
    await store.openPanel()
  } catch {
    // The store exposes the actionable failure in the toast and diagnostics.
  }
}

async function confirmUninstall() {
  uninstallConfirmOpen.value = false
  try {
    await store.uninstall()
  } catch {
    // The store exposes the actionable failure in the toast and diagnostics.
  }
}

async function toggleDiagnostics() {
  diagnosticsOpen.value = !diagnosticsOpen.value
  if (diagnosticsOpen.value) await store.refreshDiagnostics()
}

</script>

<template>
  <main class="installer-page">
    <section class="installer-shell" aria-labelledby="page-title">
      <header class="installer-header">
        <div class="brand-mark" aria-hidden="true"><Wrench :size="24" /></div>
        <div>
          <p class="overline">CS2-BOT-IMPROVER</p>
          <h1 id="page-title">CS2 人机增强助手</h1>
          <p class="subtitle">插件安装、覆盖更新、卸载、诊断与兼容工具</p>
        </div>
        <span class="version-label">{{ appConfig.appVersion }}</span>
      </header>

      <SupportActions />

      <section class="directory-section" aria-labelledby="directory-title">
        <div class="section-heading">
          <div>
            <p class="overline">安装位置</p>
            <h2 id="directory-title">CS2 游戏目录</h2>
          </div>
          <button class="icon-button" type="button" title="重新扫描 CS2 目录" aria-label="重新扫描 CS2 目录" :disabled="store.busy" @click="store.scanRoots">
            <RefreshCw :size="18" :class="{ spinning: store.busy }" />
          </button>
        </div>

        <div class="directory-control">
          <label class="sr-only" for="selected-directory">CS2 游戏目录</label>
          <input id="selected-directory" :value="store.selectedRoot" readonly placeholder="尚未选择目录" />
          <button class="secondary-button" type="button" :disabled="store.busy" @click="browse">
            <FolderOpen :size="18" />
            <span>选择目录</span>
          </button>
        </div>
        <p v-if="store.candidates.length" class="candidate-note">已找到 {{ store.candidates.length }} 个候选目录。</p>
      </section>

      <section class="status-grid" aria-label="当前状态">
        <div class="status-item" :data-state="store.selectedRoot ? 'ready' : 'warn'">
          <span>目录</span><strong>{{ store.selectedRoot ? '已选择' : '未选择' }}</strong>
        </div>
        <div class="status-item" :data-state="processVisualState">
          <span>CS2</span><strong>{{ processLabel }}</strong>
        </div>
        <div class="status-item" :data-state="store.environment?.baseEnvironmentReady ? 'ready' : 'warn'">
          <span>插件</span><strong>{{ packageState }}</strong>
        </div>
      </section>

      <section class="install-section" aria-labelledby="install-title">
        <div>
          <p class="overline">定制资源包</p>
          <h2 id="install-title">{{ appConfig.appVersion }} 定制资源包</h2>
          <p>基于上游 CS2-Bot-Improver v1.4.3，仅包含本项目说明的最小定制。</p>
          <p>{{ installHint }}</p>
        </div>
        <button class="primary-button" type="button" :disabled="installBlocked" @click="install">
          <ShieldCheck :size="20" />
          <span>{{ installLabel }}</span>
        </button>
      </section>

      <section class="panel-section" aria-labelledby="panel-title">
        <div>
          <p class="overline">高级兼容入口</p>
          <h2 id="panel-title">原版 Panel v1.4.3</h2>
          <p>融合功能异常时可临时打开原版工具；请勿让两个面板同时写入同一目录。</p>
        </div>
        <button class="secondary-button panel-button" type="button" :disabled="store.busy" @click="openPanel">
          <TerminalSquare :size="19" />
          <span>打开原版 Panel</span>
        </button>
      </section>

      <section class="utility-section">
        <button class="text-button" type="button" :aria-expanded="diagnosticsOpen" @click="toggleDiagnostics">
          {{ diagnosticsOpen ? '收起诊断信息' : '展开诊断信息' }}
        </button>
        <div v-if="diagnosticsOpen" class="diagnostics" aria-live="polite">
          <p>{{ store.diagnostics?.summary ?? '正在读取诊断信息...' }}</p>
          <details>
            <summary>运行日志</summary>
            <pre>{{ store.diagnostics?.fullLog }}</pre>
          </details>
        </div>
        <button class="danger-button" type="button" :disabled="!store.selectedRoot || store.busy" @click="uninstallConfirmOpen = true">
          <Trash2 :size="17" />
          <span>卸载插件</span>
        </button>
      </section>
    </section>

    <Teleport to="body">
      <div v-if="uninstallConfirmOpen" class="modal-backdrop" role="presentation">
        <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="uninstall-title">
          <h2 id="uninstall-title">卸载定制插件包？</h2>
          <p>将删除由定制资源包安装的插件目录和配置文件。CS2 核心文件与 gameinfo.gi 不会删除。</p>
          <div class="dialog-actions">
            <button class="secondary-button" type="button" @click="uninstallConfirmOpen = false">取消</button>
            <button class="danger-button" type="button" @click="confirmUninstall"><Trash2 :size="17" /><span>确认卸载</span></button>
          </div>
        </section>
      </div>
    </Teleport>
  </main>
</template>
