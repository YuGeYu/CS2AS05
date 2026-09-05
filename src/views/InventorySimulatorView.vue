<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue'
import { AlertTriangle, Check, CheckCircle2, ChevronRight, CircleHelp, Cloud, Copy, ExternalLink, Gamepad2, LoaderCircle, PackageCheck, Play, RefreshCw, ShieldCheck, Sparkles, Trash2, Unplug, Wrench, X } from 'lucide-vue-next'

import LaunchExperience from '@/components/LaunchExperience.vue'
import { useCs2LaunchExperience } from '@/composables/useCs2LaunchExperience'
import { checkInventorySimulatorService, getInventorySimulatorStatus, installInventorySimulator, openInventoryWorkshop, removeInventorySimulator } from '@/services/tauri/inventorySimulator'
import { dispatchToast } from '@/services/toast'
import { useCs2Store } from '@/stores/cs2'
import type { InventorySimulatorServiceStatus, InventorySimulatorStatus } from '@/types/inventory-simulator'

const cs2 = useCs2Store()
const status = ref<InventorySimulatorStatus | null>(null)
const service = ref<InventorySimulatorServiceStatus | null>(null)
const busy = ref<'status' | 'install' | 'remove' | 'service' | 'open' | 'copy' | null>(null)
const error = ref('')
const tutorialOpen = ref(false)
const removeDialogOpen = ref(false)
const launchExperience = useCs2LaunchExperience(toRef(cs2, 'selectedRoot'))
let requestGeneration = 0
let alive = true

const localState = computed(() => {
  if (!cs2.selectedRoot) return { tone: 'muted', label: '还没选择 CS2' }
  if (busy.value === 'install') return { tone: 'warning', label: '正在安稳地准备新插件' }
  if (status.value?.cs2Running && !status.value.ready) return { tone: 'warning', label: '请先退出 CS2' }
  if (status.value?.ready) return { tone: 'ready', label: '已经准备好啦' }
  if (status.value?.legacyPlayerSkinModPresent || status.value?.legacyAppDataPresent) return { tone: 'warning', label: '旧换肤待移除' }
  if (status.value?.blockedCode === 'COUNTERSTRIKESHARP_MISSING' || status.value?.blockedCode === 'CORE_GUIDELINE_ENABLED') return { tone: 'danger', label: '先补齐基础环境' }
  if (status.value?.hashMismatches.length) return { tone: 'danger', label: '文件需要修复' }
  return { tone: 'muted', label: '等待一键启用' }
})

const primaryAction = computed(() => {
  if (!cs2.selectedRoot) return { kind: 'root' as const, label: '先选择 CS2' }
  if (status.value?.blockedCode === 'COUNTERSTRIKESHARP_MISSING' || status.value?.blockedCode === 'CORE_GUIDELINE_ENABLED') return { kind: 'environment' as const, label: '前往安装与诊断' }
  if (!status.value?.ready) return { kind: 'install' as const, label: '一键启用库存换肤' }
  return { kind: 'workshop' as const, label: '打开饰品工坊' }
})

const canLaunch = computed(() => Boolean(status.value?.ready && !status.value.cs2Running && !busy.value))
const canRemove = computed(() => Boolean(cs2.selectedRoot && status.value && (status.value.inventorySimulatorPresent || status.value.gamedataPresent)))
const pluginDetail = computed(() => {
  if (!status.value) return '等待检测'
  if (status.value.ready) return `Inventory Simulator ${status.value.deployedVersion}`
  if (status.value.legacyPlayerSkinModPresent) return '检测到旧 PlayerSkinMod，将在启用时精确移除'
  if (status.value.hashMismatches.length) return `${status.value.hashMismatches.length} 个文件需要修复`
  return status.value.blockedMessage ?? '尚未安装'
})

function normalizeError(value: unknown) {
  return typeof value === 'string' ? value : value instanceof Error ? value.message : '操作没有完成，请重试。'
}

async function refresh(silent = false) {
  const root = cs2.selectedRoot
  const generation = ++requestGeneration
  if (!root) {
    status.value = null
    error.value = ''
    return
  }
  if (!silent) busy.value = 'status'
  try {
    const result = await getInventorySimulatorStatus(root)
    if (!alive || generation !== requestGeneration || root !== cs2.selectedRoot) return
    status.value = result
    error.value = ''
  } catch (reason) {
    if (!alive || generation !== requestGeneration || root !== cs2.selectedRoot) return
    status.value = null
    error.value = normalizeError(reason)
  } finally {
    if (alive && generation === requestGeneration && busy.value === 'status') busy.value = null
  }
}

async function checkService() {
  busy.value = 'service'
  try {
    service.value = await checkInventorySimulatorService()
  } catch (reason) {
    service.value = { reachable: false, checkedAt: new Date().toISOString(), message: normalizeError(reason) }
  } finally {
    if (alive && busy.value === 'service') busy.value = null
  }
}

async function install() {
  if (!cs2.selectedRoot || busy.value) return
  busy.value = 'install'
  error.value = ''
  const root = cs2.selectedRoot
  const generation = ++requestGeneration
  try {
    const result = await installInventorySimulator(root)
    if (!alive || generation !== requestGeneration || root !== cs2.selectedRoot) return
    status.value = result.status
    dispatchToast({ tone: 'ready', title: '库存换肤准备好了', message: result.removedLegacyPlugin ? '旧换肤已移除，现在只使用稳定的 Inventory Simulator。' : '插件和游戏数据已经校验完成。' })
  } catch (reason) {
    if (!alive || generation !== requestGeneration || root !== cs2.selectedRoot) return
    error.value = normalizeError(reason)
  } finally {
    if (alive && generation === requestGeneration && busy.value === 'install') busy.value = null
  }
}

async function remove() {
  if (!cs2.selectedRoot || busy.value) return
  busy.value = 'remove'
  error.value = ''
  const root = cs2.selectedRoot
  const generation = ++requestGeneration
  try {
    const result = await removeInventorySimulator(root)
    if (!alive || generation !== requestGeneration || root !== cs2.selectedRoot) return
    status.value = result.status
    removeDialogOpen.value = false
    dispatchToast({ tone: 'ready', title: '库存换肤插件已移除', message: result.removedPaths.length ? '只清理了 Inventory Simulator 自身文件，其他插件和 CS2 保持不变。' : '本机没有发现需要清理的库存换肤文件。' })
  } catch (reason) {
    if (!alive || generation !== requestGeneration || root !== cs2.selectedRoot) return
    error.value = normalizeError(reason)
  } finally {
    if (alive && generation === requestGeneration && busy.value === 'remove') busy.value = null
  }
}

async function openWorkshop() {
  busy.value = 'open'
  try {
    await openInventoryWorkshop()
    dispatchToast({ tone: 'ready', title: '饰品工坊已打开', message: '在网站保存装备后，回到游戏输入 !ws。' })
  } catch (reason) {
    error.value = normalizeError(reason)
  } finally {
    if (alive && busy.value === 'open') busy.value = null
  }
}

async function copyWs() {
  busy.value = 'copy'
  try {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
    await writeText('!ws')
    dispatchToast({ tone: 'ready', title: '已复制 !ws', message: '进入本地 BOT 对局后，粘贴到聊天框发送即可。' })
  } catch (reason) {
    error.value = normalizeError(reason)
  } finally {
    if (alive && busy.value === 'copy') busy.value = null
  }
}

function goTo(view: 'overview' | 'install') {
  window.dispatchEvent(new CustomEvent('cs2as:navigate', { detail: view }))
}

function runPrimaryAction() {
  if (primaryAction.value.kind === 'root') goTo('overview')
  else if (primaryAction.value.kind === 'environment') goTo('install')
  else if (primaryAction.value.kind === 'install') void install()
  else void openWorkshop()
}

function launch() {
  if (canLaunch.value) void launchExperience.start('bots')
}

watch(() => cs2.selectedRoot, () => { status.value = null; void refresh() })
onMounted(() => { void refresh(); void checkService() })
onBeforeUnmount(() => { alive = false; requestGeneration += 1 })
</script>

<template>
  <section class="inventory-view" aria-labelledby="inventory-title">
    <header class="inventory-heading">
      <div><p class="overline">IAN LUCAS · INVENTORY SIMULATOR</p><h1 id="inventory-title">库存换肤</h1><p>选喜欢的饰品，剩下的安装、校验和同步交给助手。</p></div>
      <div class="inventory-heading-actions"><button class="secondary-button" type="button" :disabled="Boolean(busy)" @click="refresh()"><RefreshCw :size="17" :class="{ spin: busy === 'status' }" />重新检测</button><button class="secondary-button" type="button" @click="tutorialOpen = !tutorialOpen"><CircleHelp :size="17" />{{ tutorialOpen ? '收起教程' : '查看完整教程' }}</button></div>
    </header>

    <section class="inventory-safety" role="note"><ShieldCheck :size="20" aria-hidden="true" /><div><strong>只在助手启动的本地 BOT 对局使用</strong><span>助手会自动使用 <code>-insecure</code>。不要用于官匹、5E、完美或其他受保护环境。</span></div></section>

    <section class="inventory-status-band" aria-label="库存换肤状态">
      <div :data-tone="localState.tone"><span>本机状态</span><strong><CheckCircle2 v-if="status?.ready" :size="16" /><AlertTriangle v-else-if="localState.tone === 'warning' || localState.tone === 'danger'" :size="16" />{{ localState.label }}</strong><small :title="status?.csgoRoot">{{ status?.csgoRoot || cs2.selectedRoot || '还没有选择游戏目录' }}</small></div>
      <div :data-tone="service?.reachable ? 'ready' : service ? 'warning' : 'muted'"><span>饰品服务</span><strong><Cloud v-if="service?.reachable" :size="16" /><Unplug v-else :size="16" />{{ busy === 'service' ? '正在轻轻敲门' : service?.reachable ? '连接正常' : '暂时未连接' }}</strong><small>{{ service?.message || '用于读取你在网站保存的库存' }}</small></div>
      <div :data-tone="status?.ready ? 'ready' : status?.hashMismatches.length ? 'danger' : 'muted'"><span>游戏插件</span><strong><PackageCheck :size="16" />{{ status?.ready ? '文件完整' : '等待准备' }}</strong><small>{{ pluginDetail }}</small></div>
    </section>

    <p v-if="error" class="inventory-error" role="alert"><AlertTriangle :size="18" />{{ error }}</p>

    <section class="inventory-workflow" aria-labelledby="inventory-flow-title">
      <div class="inventory-workflow-heading"><div><p class="overline">三步就好</p><h2 id="inventory-flow-title">从喜欢到游戏里</h2></div><span v-if="status?.resourceVersion">受管版本 {{ status.resourceVersion }}</span></div>
      <ol class="inventory-steps">
        <li :data-state="status?.ready ? 'done' : 'current'"><span class="inventory-step-index"><Check v-if="status?.ready" :size="18" /><Wrench v-else :size="18" /></span><div><strong>一键准备插件</strong><p>自动移除旧换肤，只安装 Ian Lucas 的库存模拟器。</p></div></li>
        <li :data-state="status?.ready ? 'current' : 'waiting'"><span class="inventory-step-index"><Sparkles :size="18" /></span><div><strong>在饰品工坊搭配</strong><p>Steam 登录后，为 CT/T 装备武器、刀、手套和角色。</p></div></li>
        <li data-state="waiting"><span class="inventory-step-index"><Gamepad2 :size="18" /></span><div><strong>启动并输入 !ws</strong><p>进入本地 BOT，发送一次 <code>!ws</code>，重生后就能看到。</p></div></li>
      </ol>
      <div class="inventory-primary-actions">
        <button class="primary-button" type="button" :disabled="Boolean(busy) || Boolean(status?.cs2Running && primaryAction.kind === 'install')" @click="runPrimaryAction"><LoaderCircle v-if="busy === 'install' || busy === 'open'" :size="18" class="spin" /><ExternalLink v-else-if="primaryAction.kind === 'workshop'" :size="18" /><Wrench v-else :size="18" />{{ busy === 'install' ? '正在准备，请稍等' : primaryAction.label }}</button>
        <button class="secondary-button" type="button" :disabled="!status?.ready || Boolean(busy)" @click="openWorkshop"><ExternalLink :size="17" />打开饰品工坊</button>
        <button class="secondary-button" type="button" :disabled="busy === 'copy'" @click="copyWs"><Copy :size="17" />复制 !ws</button>
        <button class="inventory-launch-button" type="button" :disabled="!canLaunch" @click="launch"><Play :size="18" fill="currentColor" />启动本地 BOT</button>
      </div>
      <div v-if="canRemove" class="inventory-removal-action">
        <button class="danger-button" type="button" :disabled="Boolean(busy) || Boolean(status?.cs2Running)" @click="removeDialogOpen = true"><Trash2 :size="17" />移除库存换肤插件</button>
        <span>仅移除 Inventory Simulator 自身文件，不影响其他插件。</span>
      </div>
      <p v-if="status?.cs2Running && !status.ready" class="inventory-inline-note"><AlertTriangle :size="16" />CS2 正在运行。请正常退出游戏，再点击一键启用。</p>
    </section>

    <Transition name="inventory-tutorial">
      <section v-if="tutorialOpen" class="inventory-tutorial" aria-labelledby="inventory-tutorial-title">
        <header><div><p class="overline">宝宝友好指南</p><h2 id="inventory-tutorial-title">第一次用，照着做就行</h2></div><span>大约 3 分钟</span></header>
        <div class="inventory-guide-list">
          <details open><summary><span>01</span><div><strong>先把 CS2 完全关掉</strong><small>插件安装时游戏不能占用文件。</small></div><ChevronRight :size="18" /></summary><p>确认 CS2 已经退出。如果按钮提示没有基础环境，先去“安装与诊断”完成安装，再回到这里。</p></details>
          <details><summary><span>02</span><div><strong>点击“一键启用库存换肤”</strong><small>不用找文件夹，也不用手工复制插件。</small></div><ChevronRight :size="18" /></summary><p>助手会校验内置文件，精确移除旧 PlayerSkinMod，再安装 Inventory Simulator。其他插件、Demo 和游戏文件都不会动。</p></details>
          <details><summary><span>03</span><div><strong>打开饰品工坊并登录 Steam</strong><small>网站用 SteamID 把库存认到你的玩家。</small></div><ChevronRight :size="18" /></summary><p>只在 <code>inventory.cstrike.app</code> 或 Steam 官方登录页输入信息。助手不会读取密码、Cookie，也不会在程序里嵌入登录页面。</p></details>
          <details><summary><span>04</span><div><strong>制作并“装备”饰品</strong><small>创建物品后，还要放进 CT/T 对应槽位。</small></div><ChevronRight :size="18" /></summary><p>网站支持武器、刀、手套、角色、音乐盒、贴纸、挂件、收藏品和涂鸦。CT 与 T 是两套装备；只创建但没有装备的物品不会出现在游戏里。</p></details>
          <details><summary><span>05</span><div><strong>启动本地 BOT，发送 !ws</strong><small>刷新后在重生或换图时稳定应用。</small></div><ChevronRight :size="18" /></summary><p>点击本页“启动本地 BOT”。进入地图后，在聊天框发送 <code>!ws</code>。刷新冷却为 30 秒，不需要连续发送；重生或换图后查看新外观。</p></details>
          <details><summary><span>06</span><div><strong>以后换搭配更简单</strong><small>不需要重复安装插件。</small></div><ChevronRight :size="18" /></summary><p>在网站修改并保存，回到游戏发送一次 <code>!ws</code>，然后重生即可。网站暂时打不开时不要反复重装，稍后再试。</p></details>
        </div>
        <div class="inventory-faq"><h3>没显示怎么办？</h3><p><strong>还是默认皮肤：</strong>确认网站物品已装备到当前 CT/T 槽位，等待 30 秒后发送一次 <code>!ws</code>，然后重生。</p><p><strong>网站暂时打不开：</strong>插件仍然在本机，不会影响助手其他功能；网络恢复后重新同步即可。</p><p><strong>StatTrak 或喷漆变化：</strong>这是上游完整库存能力，相关计数可能同步到公共饰品服务。</p></div>
      </section>
    </Transition>

    <footer class="inventory-provenance"><span><ShieldCheck :size="15" />上游 <strong>ianlucas/cs2-css-inventory-simulator</strong> · MIT</span><span>固定 tag {{ status?.upstreamTag || '3.1.0' }} · 不再使用自制换肤引擎</span></footer>
  </section>
  <Teleport to="body">
    <div v-if="removeDialogOpen" class="inventory-remove-backdrop" role="presentation" @click.self="removeDialogOpen = false">
      <section class="inventory-remove-dialog" role="dialog" aria-modal="true" aria-labelledby="inventory-remove-title">
        <button class="icon-button" type="button" aria-label="关闭" :disabled="busy === 'remove'" @click="removeDialogOpen = false"><X :size="18" /></button>
        <div class="inventory-remove-icon"><Trash2 :size="22" /></div>
        <p class="overline">谨慎操作</p>
        <h2 id="inventory-remove-title">移除库存换肤插件？</h2>
        <p>助手只会删除 Inventory Simulator 插件目录、它的专属配置和 gamedata 文件，不会删除 CS2、CounterStrikeSharp 或其他插件。</p>
        <p class="inventory-remove-note"><AlertTriangle :size="16" />请确认 CS2 已完全退出。移除后仍可随时点击“一键启用库存换肤”重新安装。</p>
        <div class="inventory-remove-actions">
          <button class="secondary-button" type="button" :disabled="busy === 'remove'" @click="removeDialogOpen = false">先不移除</button>
          <button class="danger-button" type="button" :disabled="busy === 'remove'" @click="remove"><LoaderCircle v-if="busy === 'remove'" :size="17" class="spin" /><Trash2 v-else :size="17" />{{ busy === 'remove' ? '正在移除' : '确认移除' }}</button>
        </div>
      </section>
    </div>
  </Teleport>
  <LaunchExperience :active="launchExperience.active.value" :elapsed-ms="launchExperience.elapsedMs.value" :mode="launchExperience.mode.value" @dismiss="launchExperience.dismiss" />
</template>
