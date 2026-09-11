<script setup lang="ts">
import { computed, ref } from 'vue'
import { AlertTriangle, Check, CheckCircle2, ClipboardCopy, Crosshair, Info, Sparkles, Users } from 'lucide-vue-next'
import SegmentedControl from '@/components/ui/SegmentedControl.vue'
import { TEAMS } from '@/data/panel/commands'
import type { AimValue, NadesValue } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store(); const panel = usePanelStore()
const teamIndex = ref(''); const side = ref<'ct' | 't'>('ct'); const copied = ref('')
const aimOptions = [{ value: 'head', label: '头部' }, { value: 'mixed', label: '混合' }, { value: 'body', label: '身体' }] as const
const nadeOptions = [{ value: 'max', label: '最多' }, { value: 'more', label: '较多' }, { value: 'normal', label: '正常' }, { value: 'less', label: '较少' }, { value: 'off', label: '关闭' }] as const
const selectedTeam = computed(() => TEAMS.find(team => String(team.index) === teamIndex.value))
const blocked = computed(() => !panel.snapshot?.ready)
const environmentLabel = computed(() => {
  if (!panel.snapshot) return '正在读取配置'
  if (!panel.snapshot.ready) return '环境未就绪'
  if (panel.snapshot.cs2Running) return '退出 CS2 后应用'
  if (!panel.snapshot.presets.writable) return '配置不可写'
  return '配置可写'
})
const environmentTone = computed(() => !panel.snapshot?.ready || panel.snapshot.cs2Running || !panel.snapshot.presets.writable ? 'warn' : 'ready')
const aimLabel = computed(() => aimOptions.find(item => item.value === panel.snapshot?.presets.aim)?.label ?? '尚未读取')
const nadeLabel = computed(() => nadeOptions.find(item => item.value === panel.snapshot?.presets.nades)?.label ?? '尚未读取')
const commandPreview = computed(() => selectedTeam.value ? (side.value === 'ct' ? selectedTeam.value.ct : selectedTeam.value.t) : '选择队伍后预览完整命令')
const setAim = (value: AimValue) => panel.setAim(cs2.selectedRoot, value).catch(() => undefined)
const setNades = (value: NadesValue) => panel.setNades(cs2.selectedRoot, value).catch(() => undefined)
async function copyTeam() {
  const team = selectedTeam.value; if (!team) return
  const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
  await writeText(side.value === 'ct' ? team.ct : team.t)
  copied.value = `${team.name} ${side.value.toUpperCase()} 命令已复制`
  window.setTimeout(() => { copied.value = '' }, 1600)
}
</script>

<template>
  <section class="tool-view presets-view" aria-labelledby="presets-title">
    <header class="view-heading presets-heading">
      <div><p class="overline">BOT 行为工作台</p><h1 id="presets-title">人机预设</h1><p class="presets-heading__description">调整瞄准与投掷物行为，下一次助手启动本地对局时生效。</p></div>
      <div class="presets-environment" :data-tone="environmentTone" aria-live="polite"><CheckCircle2 v-if="environmentTone === 'ready'" :size="17" aria-hidden="true" /><AlertTriangle v-else :size="17" aria-hidden="true" /><span>{{ environmentLabel }}</span></div>
    </header>
    <div class="presets-summary" aria-label="当前配置摘要"><div><span>瞄准策略</span><strong>{{ aimLabel }}</strong></div><div><span>投掷物频率</span><strong>{{ nadeLabel }}</strong></div><div><span>生效范围</span><strong>本地 BOT 对局</strong></div></div>
    <section class="presets-section" aria-labelledby="behavior-title">
      <div class="presets-section-heading"><div><p class="overline">BEHAVIOR PROFILE</p><h2 id="behavior-title">BOT 行为设置</h2></div><span><Sparkles :size="15" aria-hidden="true" />即时写入配置</span></div>
      <div class="presets-control-grid">
        <article class="preset-module" :data-pending="panel.mutationKey === 'aim' || undefined"><div class="preset-module__top"><div class="preset-module__icon"><Crosshair :size="18" aria-hidden="true" /></div><div><p class="preset-module__eyebrow">AIM PROFILE</p><h3>瞄准策略</h3></div><span class="preset-status" :data-tone="panel.mutationKey === 'aim' ? 'pending' : panel.snapshot?.presets.aim ? 'ready' : 'muted'"><CheckCircle2 v-if="panel.snapshot?.presets.aim && panel.mutationKey !== 'aim'" :size="13" aria-hidden="true" />{{ panel.mutationKey === 'aim' ? '正在应用' : panel.snapshot?.presets.aim ? '已生效' : '尚未读取' }}</span></div><p class="preset-module__description">决定 Bot 优先锁定身体的哪个区域。</p><SegmentedControl :model-value="panel.snapshot?.presets.aim ?? null" :options="aimOptions" label="瞄准策略" :disabled="blocked" :pending="panel.mutationKey === 'aim'" @update:model-value="setAim" /><p class="preset-module__hint">头部更强调爆头威胁 · 混合保持动态切换 · 身体形成持续压制</p></article>
        <article class="preset-module" :data-pending="panel.mutationKey === 'nades' || undefined"><div class="preset-module__top"><div class="preset-module__icon"><Sparkles :size="18" aria-hidden="true" /></div><div><p class="preset-module__eyebrow">NADE TEMPO</p><h3>投掷物频率</h3></div><span class="preset-status" :data-tone="panel.mutationKey === 'nades' ? 'pending' : panel.snapshot?.presets.nades ? 'ready' : 'muted'"><CheckCircle2 v-if="panel.snapshot?.presets.nades && panel.mutationKey !== 'nades'" :size="13" aria-hidden="true" />{{ panel.mutationKey === 'nades' ? '正在应用' : panel.snapshot?.presets.nades ? '已生效' : '尚未读取' }}</span></div><p class="preset-module__description">控制 Bot 使用手雷、闪光和烟雾的积极程度。</p><SegmentedControl :model-value="panel.snapshot?.presets.nades ?? null" :options="nadeOptions" label="投掷物频率" :disabled="blocked" :pending="panel.mutationKey === 'nades'" @update:model-value="setNades" /><p class="preset-module__hint">从积极压制到完全关闭，按当前对局节奏自由调整</p></article>
      </div>
    </section>
    <section class="presets-team-band" aria-labelledby="team-title"><div class="presets-section-heading"><div><p class="overline">TEAM LOADOUT</p><h2 id="team-title">队伍预设</h2><p>选择一支队伍，复制对应阵营命令到 CS2 控制台。</p></div><Users :size="20" aria-hidden="true" /></div><div class="team-controls presets-team-controls"><label class="presets-select-label" for="team-select">队伍选择<select id="team-select" v-model="teamIndex"><option value="">选择队伍</option><option v-for="team in TEAMS" :key="team.index" :value="String(team.index)">{{ team.index }}. {{ team.name }}</option></select></label><div class="presets-side-field"><span>队伍阵营</span><SegmentedControl v-model="side" :options="[{ value: 'ct', label: 'CT' }, { value: 't', label: 'T' }]" label="队伍阵营" /></div><button class="primary-button presets-copy-button" type="button" :disabled="!selectedTeam" :data-copied="Boolean(copied)" :aria-label="selectedTeam ? '复制完整队伍命令' : '请先选择队伍'" @click="copyTeam"><Check v-if="copied" :size="17" aria-hidden="true" /><ClipboardCopy v-else :size="17" aria-hidden="true" />{{ copied ? '已复制' : '复制命令' }}</button></div><div class="presets-command-preview"><Info :size="14" aria-hidden="true" /><code :title="commandPreview">{{ commandPreview }}</code></div><p v-if="copied" class="success-note" aria-live="polite">{{ copied }}</p><p v-else class="presets-scope-note"><Info :size="14" aria-hidden="true" />这些命令只在本地 BOT/控制台场景中使用，不会改变官方匹配服务器。</p></section>
  </section>
</template>
