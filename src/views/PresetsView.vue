<script setup lang="ts">
import { computed, ref } from 'vue'
import { Copy } from 'lucide-vue-next'
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
const setAim = (value: AimValue) => panel.setAim(cs2.selectedRoot, value).catch(() => undefined)
const setNades = (value: NadesValue) => panel.setNades(cs2.selectedRoot, value).catch(() => undefined)
async function copyTeam() { const team = selectedTeam.value; if (!team) return; const { writeText } = await import('@tauri-apps/plugin-clipboard-manager'); await writeText(side.value === 'ct' ? team.ct : team.t); copied.value = `${team.name} ${side.value.toUpperCase()} 命令已复制` }
</script>

<template>
  <section class="tool-view" aria-labelledby="presets-title">
    <header class="view-heading"><div><p class="overline">游戏内行为</p><h1 id="presets-title">人机预设</h1></div></header>
    <section class="control-grid"><div class="control-group"><div><h2>Aim</h2><p>设置 Bot 的瞄准区域。</p></div><SegmentedControl :model-value="panel.snapshot?.presets.aim ?? null" :options="aimOptions" label="Aim" :disabled="blocked" :pending="panel.mutationKey === 'aim'" @update:model-value="setAim" /></div><div class="control-group"><div><h2>Nades</h2><p>设置 Bot 使用投掷物的频率。</p></div><SegmentedControl :model-value="panel.snapshot?.presets.nades ?? null" :options="nadeOptions" label="Nades" :disabled="blocked" :pending="panel.mutationKey === 'nades'" @update:model-value="setNades" /></div></section>
    <section class="control-band"><div class="field-heading"><div><h2>队伍预设</h2><p>共 {{ TEAMS.length }} 支队伍，复制后粘贴到 CS2 控制台。</p></div></div><div class="team-controls"><select v-model="teamIndex" aria-label="队伍"><option value="">选择队伍</option><option v-for="team in TEAMS" :key="team.index" :value="String(team.index)">{{ team.index }}. {{ team.name }}</option></select><SegmentedControl v-model="side" :options="[{ value: 'ct', label: 'CT' }, { value: 't', label: 'T' }]" label="队伍阵营" /><button class="icon-button" title="复制完整队伍命令" aria-label="复制完整队伍命令" :disabled="!selectedTeam" @click="copyTeam"><Copy :size="18" /></button></div><p v-if="copied" class="success-note" aria-live="polite">{{ copied }}</p></section>
  </section>
</template>
