<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { AlertTriangle, LoaderCircle } from 'lucide-vue-next'
import { getMatchPerformanceRadar } from '@/services/tauri/demo'
import type { MatchPerformanceRadar, PerformanceRadarPlayer } from '@/types/demo'
import { formatRadarRaw, formatRadarScore, radarStroke, stableRadarCandidates } from '@/features/demo/radar/performance-radar'
import PerformanceRadarChart from './PerformanceRadarChart.vue'

const props = defineProps<{ demoId: number; initialPlayerKey?: string | null }>()
const report = ref<MatchPerformanceRadar | null>(null)
const roster = ref<PerformanceRadarPlayer[]>([])
const selectedKeys = ref<string[]>([])
const loading = ref(false)
const showSkeleton = ref(false)
const error = ref('')
let requestGeneration = 0
let skeletonTimer: ReturnType<typeof setTimeout> | undefined

const selectedPlayers = computed(() => selectedKeys.value.map(key => report.value?.players.find(player => player.stableKey === key)).filter((player): player is PerformanceRadarPlayer => Boolean(player)))
const selectableRoster = computed(() => roster.value.filter(player => player.teamNumber === 2 || player.teamNumber === 3))
const unavailableRoster = computed(() => roster.value.filter(player => player.teamNumber !== 2 && player.teamNumber !== 3))
const qualityLabel: Record<string, string> = { complete: '完整', partial: '部分可用', unavailable: '不可用' }
const benchmarkLabel = (unit: string, benchmark: number) => unit === '百分比' ? `${(benchmark * 100).toFixed(0)}%` : benchmark.toFixed(2)
const playerName = (player: PerformanceRadarPlayer) => player.name || player.stableKey
const rankValue = (player: PerformanceRadarPlayer) => player.rankingRatingModel === 'lb-rating-2.0' && Number.isFinite(player.rankingRating) ? player.rankingRating! : null
const rankedRoster = computed(() => [...roster.value].filter(player => player.teamNumber === 2 || player.teamNumber === 3).sort((a, b) => {
  const ar = rankValue(a); const br = rankValue(b)
  if (ar != null && br != null) return br - ar || a.stableKey.localeCompare(b.stableKey)
  if (ar != null) return -1
  if (br != null) return 1
  return a.stableKey.localeCompare(b.stableKey)
}))
const playerRank = (player: PerformanceRadarPlayer) => {
  const index = rankedRoster.value.findIndex(item => item.stableKey === player.stableKey)
  return index < 0 ? '--' : `${index + 1} / ${rankedRoster.value.length}`
}
const rankBadge = (player: PerformanceRadarPlayer) => ({ 1: '冠军', 2: '亚军', 3: '季军' } as Record<number, string>)[Number(playerRank(player).split(' / ')[0])] || ''

function beginLoading() {
  loading.value = true
  showSkeleton.value = false
  if (skeletonTimer) clearTimeout(skeletonTimer)
  skeletonTimer = setTimeout(() => { showSkeleton.value = loading.value }, 300)
}

async function load(keys?: string[], resetRoster = false) {
  const generation = ++requestGeneration
  beginLoading(); error.value = ''
  try {
    const result = await getMatchPerformanceRadar(props.demoId, keys?.length ? keys : undefined)
    if (generation !== requestGeneration) return
    report.value = result
    if (resetRoster) {
      roster.value = result.players
      const ordered = stableRadarCandidates(result.players.filter(player => player.teamNumber === 2 || player.teamNumber === 3))
      const focused = props.initialPlayerKey ? ordered.find(player => player.stableKey === props.initialPlayerKey) : undefined
      const initialKey = focused?.stableKey || ordered[0]?.stableKey
      selectedKeys.value = initialKey ? [initialKey] : []
    }
  } catch (reason) {
    if (generation === requestGeneration) error.value = reason instanceof Error ? reason.message : String(reason)
  } finally {
    if (generation === requestGeneration) {
      loading.value = false; showSkeleton.value = false
      if (skeletonTimer) clearTimeout(skeletonTimer)
    }
  }
}

async function togglePlayer(key: string) {
  const next = selectedKeys.value.includes(key) ? selectedKeys.value.filter(item => item !== key) : [...selectedKeys.value, key]
  if (!next.length || next.length > 3) return
  selectedKeys.value = next
  await load(next)
}

watch(() => props.demoId, () => { report.value = null; roster.value = []; selectedKeys.value = []; void load(undefined, true) }, { immediate: true })
watch(() => props.initialPlayerKey, key => {
  if (!key || !roster.value.some(player => player.stableKey === key)) return
  selectedKeys.value = [key]
  void load([key])
})
onBeforeUnmount(() => { requestGeneration++; if (skeletonTimer) clearTimeout(skeletonTimer) })
</script>

<template>
  <div class="match-performance-radar">
    <header class="radar-heading">
      <div><p class="overline">六维对局画像</p><h2>表现雷达</h2><p>表现雷达 v1 · 不等于 Rating</p></div>
      <span class="radar-model">{{ report?.modelVersion || 'performance-radar-v1' }}</span>
    </header>
    <div v-if="error" class="radar-state radar-state--error" role="alert"><AlertTriangle :size="18" /><span><strong>表现数据读取失败</strong>{{ error }}</span></div>
    <div v-else-if="showSkeleton" class="radar-skeleton" role="status"><LoaderCircle class="spinning" :size="20" />正在整理六维表现…</div>
    <template v-else-if="roster.length">
      <div class="radar-player-picker" aria-label="选择对比玩家">
        <button v-for="player in selectableRoster" :key="player.stableKey" type="button" :class="{ 'is-selected': selectedKeys.includes(player.stableKey) }" :aria-pressed="selectedKeys.includes(player.stableKey)" :disabled="!selectedKeys.includes(player.stableKey) && selectedKeys.length >= 3" :title="!selectedKeys.includes(player.stableKey) && selectedKeys.length >= 3 ? '最多同时比较三名玩家' : undefined" @click="togglePlayer(player.stableKey)">
          <i :style="selectedKeys.includes(player.stableKey) ? { borderColor: radarStroke(selectedKeys.indexOf(player.stableKey)), borderStyle: selectedKeys.indexOf(player.stableKey) === 1 ? 'dashed' : selectedKeys.indexOf(player.stableKey) === 2 ? 'dotted' : 'solid' } : undefined" />
          <span>{{ playerName(player) }}<small>{{ player.teamName || `队伍 ${player.teamNumber}` }}<template v-if="player.isBot"> · BOT</template></small></span>
        </button>
      </div>
      <p v-if="selectedKeys.length >= 3" class="radar-limit" role="status">已选择三名玩家；移除一名后可继续选择。</p>
      <div class="radar-workspace">
        <PerformanceRadarChart v-if="selectedPlayers.length" :players="selectedPlayers" />
        <div class="radar-insight">
          <strong>本场表现排名</strong><p>排名按 LBRating 2.0 从高到低；Rating 不可用时按稳定顺序降级。</p>
          <div v-for="player in selectedPlayers" :key="`rank-${player.stableKey}`" class="radar-rank-item"><b>{{ rankBadge(player) }} {{ playerRank(player) }}</b><span>{{ playerName(player) }}<small>{{ player.teamName || `队伍 ${player.teamNumber}` }}<template v-if="player.isBot"> · BOT</template></small></span></div>
          <div v-for="player in selectedPlayers" :key="player.stableKey" class="radar-warning-list">
            <p v-for="warning in player.warnings" :key="warning"><AlertTriangle :size="14" />{{ playerName(player) }}：{{ warning }}</p>
          </div>
        </div>
      </div>
      <div class="demo-table-wrap radar-table-wrap">
        <table class="demo-table radar-table">
          <caption class="sr-only">表现雷达精确数据</caption>
          <thead><tr><th>维度</th><th>玩家</th><th>图形分数</th><th>原始值</th><th>基准</th><th>数据质量</th></tr></thead>
          <tbody>
            <template v-for="player in selectedPlayers" :key="player.stableKey">
              <tr v-for="dimension in player.dimensions" :key="`${player.stableKey}-${dimension.key}`">
                <td>{{ dimension.label }}</td><td>{{ playerName(player) }}</td><td>{{ formatRadarScore(dimension.score) }}</td>
                <td :title="dimension.rawLabel">{{ formatRadarRaw(dimension) }} {{ dimension.raw == null ? '' : dimension.unit }}</td>
                <td>{{ benchmarkLabel(dimension.unit, dimension.benchmark) }} {{ dimension.unit }}</td><td><span class="radar-quality" :data-quality="dimension.quality">{{ qualityLabel[dimension.quality] }}</span></td>
              </tr>
            </template>
          </tbody>
        </table>
      </div>
      <p v-if="unavailableRoster.length" class="radar-unavailable"><AlertTriangle :size="15" />{{ unavailableRoster.length }} 名观战者或无正式回合身份已保留，但不进入默认选择器。</p>
    </template>
    <div v-else-if="!loading" class="radar-state" role="status">当前 Demo 没有可用于表现雷达的玩家数据。</div>
  </div>
</template>

<style scoped>
.match-performance-radar { min-width: 0; }
.radar-heading { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; margin-bottom: 14px; }
.radar-heading h2 { margin: 2px 0 3px; font-size: 19px; letter-spacing: 0; }
.radar-heading p { margin: 0; color: var(--text-muted); font-size: 11px; }
.radar-model { padding: 6px 8px; border: 1px solid var(--border-strong); border-radius: 4px; color: var(--bronze); font: 10px Consolas, "Cascadia Mono", monospace; }
.radar-player-picker { display: flex; flex-wrap: wrap; gap: 7px; padding: 10px 0 12px; border-block: 1px solid var(--border); }
.radar-player-picker button { display: flex; min-width: 142px; min-height: 44px; align-items: center; gap: 9px; padding: 5px 10px; border: 1px solid var(--border); border-radius: 5px; background: var(--surface); color: var(--text); text-align: left; transition: border-color 160ms ease, background-color 160ms ease, transform 120ms ease-out; }
.radar-player-picker button:active:not(:disabled) { transform: scale(.98); }
.radar-player-picker button.is-selected { border-color: var(--border-strong); background: var(--surface-raised); box-shadow: inset 3px 0 var(--primary); }
.radar-player-picker button:disabled { cursor: not-allowed; opacity: .42; }
.radar-player-picker i { width: 22px; border-top: 3px solid var(--text-muted); }
.radar-player-picker span { display: grid; min-width: 0; gap: 2px; overflow: hidden; font-size: 11px; font-weight: 800; text-overflow: ellipsis; white-space: nowrap; }
.radar-player-picker small { color: var(--text-muted); font-size: 9px; font-weight: 500; }
.radar-limit, .radar-unavailable { display: flex; align-items: center; gap: 7px; margin: 9px 0; color: var(--text-muted); font-size: 10px; }
.radar-workspace { display: grid; grid-template-columns: minmax(480px, 1.5fr) minmax(210px, .5fr); gap: 18px; align-items: center; padding: 14px 0; }
.radar-insight { align-self: start; padding: 14px; border-left: 2px solid var(--border-strong); background: var(--surface-muted); }
.radar-insight > p { color: var(--text-muted); font-size: 11px; line-height: 1.7; }
.radar-warning-list p { display: flex; gap: 6px; margin: 8px 0; color: var(--warning); font-size: 10px; line-height: 1.45; }
.radar-warning-list svg { flex: 0 0 auto; }
.radar-table-wrap { max-height: 360px; }
.radar-table { min-width: 720px; }
.radar-quality { display: inline-flex; padding: 2px 6px; border-left: 3px solid var(--success); background: var(--surface-muted); }
.radar-quality[data-quality='partial'] { border-color: var(--warning); }
.radar-quality[data-quality='unavailable'] { border-color: var(--danger); }
.radar-state, .radar-skeleton { display: flex; min-height: 180px; align-items: center; justify-content: center; gap: 9px; border: 1px solid var(--border); color: var(--text-muted); }
.radar-state--error { border-color: color-mix(in srgb, var(--danger) 55%, var(--border)); color: var(--danger); }
.radar-state--error span { display: grid; gap: 3px; }
@media (max-width: 920px) { .radar-workspace { grid-template-columns: 1fr; } .radar-insight { border-top: 1px solid var(--border); border-left: 0; } }
@media (max-width: 560px) { .radar-heading { align-items: stretch; flex-direction: column; } .radar-model { align-self: flex-start; } .radar-player-picker button { min-width: min(100%, 170px); flex: 1 1 140px; } }
@media (prefers-reduced-motion: reduce) { .radar-player-picker button { transition: none; } }
</style>
