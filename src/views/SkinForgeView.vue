<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { BookOpen, Check, CircleAlert, Download, LoaderCircle, RotateCcw, Save, ShieldCheck, TriangleAlert } from 'lucide-vue-next'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import ForgeHelpDialog from '@/features/skin-forge/components/ForgeHelpDialog.vue'
import ForgeSafetyDialog from '@/features/skin-forge/components/ForgeSafetyDialog.vue'
import ForgeWorkbench from '@/features/skin-forge/components/ForgeWorkbench.vue'
import { consumeQuickStartCue, shouldShowQuickStartCue } from '@/features/skin-forge/quick-start-cue'
import { confirmSafetyToday, hasConfirmedSafetyToday } from '@/features/skin-forge/safety-confirmation'
import { useCs2Store } from '@/stores/cs2'
import { useSkinForgeStore } from '@/stores/skinForge'
import type { ForgeMode, Team } from '@/types/skin-forge'

const cs2 = useCs2Store()
const forge = useSkinForgeStore()
const category = ref<'weapons' | 'knives' | 'gloves' | 'agents' | 'music'>('weapons')
const resetOpen = ref(false)
const help = ref<'tutorial' | 'safety' | null>(null)
const safetyConfirmed = ref(hasConfirmedSafetyToday())
const quickStartHighlighted = ref(false)
let quickStartTimer: ReturnType<typeof setTimeout> | undefined
const tabs = [
  { value: 'weapons', label: '武器' }, { value: 'knives', label: '刀具' }, { value: 'gloves', label: '手套' },
  { value: 'agents', label: '角色' }, { value: 'music', label: '音乐盒' },
] as const
const teamOptions = [{ value: 'ct', label: 'CT 阵营' }, { value: 't', label: 'T 阵营' }] as const
const pluginReady = computed(() => forge.plugin?.allPresent === true && forge.plugin.hashMismatches.length === 0)
const workshopUnlocked = computed(() => safetyConfirmed.value && pluginReady.value)
const pluginLabel = computed(() => {
  if (cs2.cs2Running) return 'CS2 运行中，部署已锁定'
  if (!cs2.selectedRoot) return '等待选择 CS2 目录'
  if (pluginReady.value) return `PlayerSkinMod ${forge.plugin?.deployedVersion ?? '1.8.2'} 已就绪`
  if (forge.plugin?.deployedVersion) return `检测到 ${forge.plugin.deployedVersion}，可更新`
  return '插件尚未部署'
})
// The CS2 root is restored before this view is mounted, so the store's root
// watcher may not run during a subsequent app launch. Always re-read the
// on-disk PlayerSkinMod state when entering the workshop.
onMounted(() => { void forge.load() })
async function confirmReset() { await forge.reset(); resetOpen.value = false }
function cancelSafety() { window.dispatchEvent(new CustomEvent('cs2as:navigate', { detail: 'overview' })) }
function confirmSafety() {
  confirmSafetyToday()
  safetyConfirmed.value = true
  void forge.load()
}
function openQuickStart() {
  consumeQuickStartCue()
  quickStartHighlighted.value = false
  if (quickStartTimer) clearTimeout(quickStartTimer)
  help.value = 'tutorial'
}
watch(workshopUnlocked, unlocked => {
  if (!unlocked || quickStartHighlighted.value || !shouldShowQuickStartCue()) return
  quickStartHighlighted.value = true
  quickStartTimer = setTimeout(() => {
    consumeQuickStartCue()
    quickStartHighlighted.value = false
  }, 3_000)
}, { immediate: true })
onBeforeUnmount(() => {
  if (quickStartTimer) clearTimeout(quickStartTimer)
  if (quickStartHighlighted.value) consumeQuickStartCue()
})
</script>

<template>
  <section class="tool-view skin-forge-view" aria-labelledby="skin-forge-title">
    <header class="view-heading skin-forge-heading">
      <div><p class="overline">SKIN FORGE / PLAYER SKIN MOD 1.8.2</p><h1 id="skin-forge-title">皮肤工坊</h1><p class="subtitle">用图片和名称搭配完整装备，CT 与 T 阵营各自保存。</p></div>
      <div class="skin-forge-actions">
        <button class="primary-button" :disabled="!workshopUnlocked || forge.busy || cs2.cs2Running || !cs2.selectedRoot" title="将当前装备写入游戏目录" @click="forge.save"><LoaderCircle v-if="forge.saving" :size="17" class="spin" /><Save v-else :size="17" />{{ forge.saving ? '正在应用' : '应用装备' }}</button>
        <button class="icon-button forge-quick-start" :class="{ 'is-highlighted': quickStartHighlighted }" type="button" title="快速上手" aria-label="快速上手" @click="openQuickStart"><BookOpen :size="18" /></button>
        <button class="icon-button" :disabled="!workshopUnlocked || forge.busy" type="button" title="重置为默认配置" aria-label="重置配置" @click="resetOpen = true"><RotateCcw :size="18" /></button>
      </div>
    </header>

    <section class="skin-forge-status" aria-label="部署状态">
      <div><span>游戏目录</span><strong :title="cs2.selectedRoot || ''">{{ cs2.selectedRoot || '尚未选择' }}</strong></div>
      <div :data-state="cs2.cs2Running ? 'warn' : pluginReady ? 'ready' : 'idle'" aria-live="polite"><span>运行环境</span><strong><ShieldCheck v-if="pluginReady" :size="15" /><TriangleAlert v-else :size="15" />{{ pluginLabel }}</strong></div>
      <div class="forge-deploy-anchor" :class="{ 'is-required': safetyConfirmed && !pluginReady }">
        <button class="secondary-button" :disabled="!safetyConfirmed || forge.deploying || cs2.cs2Running || !cs2.selectedRoot" type="button" title="检查、备份并部署固定版本插件" @click="forge.deploy"><LoaderCircle v-if="forge.deploying" :size="17" class="spin" /><Download v-else :size="17" />{{ forge.deploying ? '正在校验与部署' : '部署 / 更新插件' }}</button>
        <div v-if="safetyConfirmed && !pluginReady" class="forge-deploy-cue" role="status">先部署 PlayerSkinMod，工坊才会开放装备选择并写入配置。</div>
      </div>
    </section>

    <section v-if="forge.error" class="forge-error" role="alert"><CircleAlert :size="18" /><div><strong>工坊操作未完成</strong><span>{{ forge.error }}</span></div></section>
    <section class="skin-forge-toolbar">
      <SegmentedControl :model-value="forge.loadout.mode" :options="[{ value: 'custom', label: '自定义搭配' }, { value: 'random', label: '每次重生随机' }]" :disabled="!workshopUnlocked" label="装备模式" @update:model-value="value => forge.setMode(value as ForgeMode)" />
      <SegmentedControl :model-value="forge.loadout.activeTeam" :options="teamOptions" :disabled="!workshopUnlocked" label="当前阵营" @update:model-value="value => forge.setTeam(value as Team)" />
      <button class="forge-safety-link" type="button" @click="help = 'safety'"><TriangleAlert :size="15" />仅用于 <code>-insecure</code> 离线环境</button>
    </section>
    <nav class="skin-forge-tabs" aria-label="装备分类"><button v-for="item in tabs" :key="item.value" type="button" :aria-current="category === item.value ? 'page' : undefined" @click="category = item.value">{{ item.label }}</button></nav>
    <ForgeWorkbench :category="category" :disabled="!workshopUnlocked" />

    <div class="forge-save-state" aria-live="polite">
      <span v-if="forge.dirty"><CircleAlert :size="15" />有未应用的修改</span>
      <span v-else-if="forge.lastSave"><Check :size="15" />已写入 {{ forge.lastSave.loadoutPath }} · {{ forge.lastSave.sha256.slice(0, 12) }}…</span>
      <span v-else>选择图片即可开始搭配</span>
    </div>

    <ForgeHelpDialog v-if="help" :page="help" @close="help = null" />
    <ForgeSafetyDialog v-if="!safetyConfirmed" @confirm="confirmSafety" @cancel="cancelSafety" />
    <div v-if="resetOpen" class="modal-backdrop forge-modal-backdrop" @mousedown.self="resetOpen = false" @keydown.esc="resetOpen = false">
      <section class="confirm-dialog" role="dialog" aria-modal="true" aria-labelledby="skin-forge-reset-title">
        <div class="modal-heading"><TriangleAlert :size="22" /><div><h2 id="skin-forge-reset-title">重置整套装备？</h2><p>将清除 slot 0 的自定义配置并恢复随机模式。插件、框架和其他玩家文件不会受影响。</p></div></div>
        <div class="dialog-actions"><button class="secondary-button" type="button" autofocus @click="resetOpen = false">保留当前搭配</button><button class="danger-button" type="button" :disabled="forge.saving" @click="confirmReset">确认重置</button></div>
      </section>
    </div>
  </section>
</template>
