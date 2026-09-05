<script setup lang="ts">
import { computed, ref } from 'vue'
import type { PerformanceRadarPlayer } from '@/types/demo'
import { RADAR_DIMENSION_KEYS, formatRadarRaw, formatRadarScore, radarDash, radarGrid, radarLabelPoint, radarPoint, radarPolygon, radarScaleMax, radarSegments, radarStroke } from '@/features/demo/radar/performance-radar'

const props = defineProps<{ players: PerformanceRadarPlayer[] }>()
const displayScale = ref(1)
const setScale = (value: number) => { displayScale.value = Math.min(2, Math.max(.75, Number(value.toFixed(2)))) }
const radarRadius = computed(() => 222 * displayScale.value)
const scaleMax = computed(() => radarScaleMax(props.players.flatMap(player => player.dimensions.map(dimension => dimension.score ?? Number.NaN))))
const gridLevels = computed(() => [20, 40, 60, 80, 100, scaleMax.value].filter((level, index, levels) => levels.indexOf(level) === index))
const axes = computed(() => RADAR_DIMENSION_KEYS.map((key, index) => ({ key, index, point: radarPoint(index, scaleMax.value), label: radarLabelPoint(index) })))
const dimension = (player: PerformanceRadarPlayer | undefined, key: string) => player?.dimensions.find(item => item.key === key)
const playerLabel = (player: PerformanceRadarPlayer) => player.name || player.stableKey
</script>

<template>
  <figure class="performance-radar-chart">
    <div class="radar-zoom" role="group" aria-label="雷达图缩放"><button type="button" aria-label="缩小雷达图" @click="setScale(displayScale - .05)">-</button><input aria-label="雷达图缩放比例" type="range" min="0.75" max="2" step="0.05" :value="displayScale" @input="setScale(Number(($event.target as HTMLInputElement).value))" /><button type="button" aria-label="放大雷达图" @click="setScale(displayScale + .05)">+</button><button type="button" aria-label="重置雷达图缩放" @click="setScale(1)">{{ Math.round(displayScale * 100) }}%</button></div>
    <svg viewBox="0 0 640 640" role="img" aria-labelledby="performance-radar-title performance-radar-desc">
      <title id="performance-radar-title">玩家六维对局表现雷达</title>
      <desc id="performance-radar-desc">100 分为指标基准圈，外圈为当前对比显示上限。缺失轴不会按零补齐，精确数据见图表后的表格。</desc>
      <g class="radar-grid" aria-hidden="true">
        <polygon v-for="level in gridLevels" :key="level" :points="radarGrid(level, radarRadius, 320, scaleMax)" />
        <line v-for="axis in axes" :key="axis.key" x1="320" y1="320" :x2="axis.point.x" :y2="axis.point.y" />
      </g>
      <g v-for="(player, playerIndex) in players" :key="player.stableKey" class="radar-player">
        <polygon v-if="radarPolygon(player.dimensions, radarRadius, 320, scaleMax)" class="radar-player-fill" :points="radarPolygon(player.dimensions, radarRadius, 320, scaleMax) || ''" :fill="radarStroke(playerIndex)" />
        <polygon v-if="radarPolygon(player.dimensions, radarRadius, 320, scaleMax)" class="radar-player-line" :points="radarPolygon(player.dimensions, radarRadius, 320, scaleMax) || ''" fill="none" :stroke="radarStroke(playerIndex)" :stroke-dasharray="radarDash(playerIndex)" role="img" :aria-label="`${playerLabel(player)} 的六维表现`" />
        <polyline v-for="(segment, segmentIndex) in radarSegments(player.dimensions, radarRadius, 320, scaleMax)" v-else :key="segmentIndex" class="radar-player-line" :points="segment" fill="none" :stroke="radarStroke(playerIndex)" :stroke-dasharray="radarDash(playerIndex)" />
      </g>
      <g v-for="axis in axes" :key="`label-${axis.key}`" class="radar-axis-label">
        <text :x="axis.label.x" :y="axis.label.y - 8" text-anchor="middle">{{ dimension(players[0], axis.key)?.label || axis.key }}</text>
        <text :x="axis.label.x" :y="axis.label.y + 13" text-anchor="middle" class="radar-axis-value">
          {{ players.map(player => `${formatRadarScore(dimension(player, axis.key)?.score ?? null)} · ${dimension(player, axis.key) ? formatRadarRaw(dimension(player, axis.key)!) : '--'}`).join(' / ') }}
        </text>
      </g>
    </svg>
    <p class="radar-scale-note">100 分基准圈 · 当前显示上限：{{ scaleMax }}</p>
    <figcaption>
      <span v-for="(player, index) in players" :key="player.stableKey">
        <i :style="{ borderColor: radarStroke(index), borderStyle: index === 1 ? 'dashed' : index === 2 ? 'dotted' : 'solid' }" />
        {{ playerLabel(player) }}<small v-if="player.isBot">BOT</small>
      </span>
    </figcaption>
  </figure>
</template>

<style scoped>
.performance-radar-chart { min-width: 0; margin: 0; }
.radar-zoom { display:flex; justify-content:flex-end; align-items:center; gap:6px; margin-bottom:6px; }
.radar-zoom button { min-width:28px; min-height:28px; border:1px solid var(--border); background:var(--surface); color:var(--text); }
.radar-zoom input { width:120px; accent-color:var(--primary); }
svg { display: block; width: 100%; max-height: min(620px, 70vh); overflow: visible; }
.radar-grid polygon, .radar-grid line { fill: none; stroke: var(--border-strong); stroke-width: 1; vector-effect: non-scaling-stroke; }
.radar-grid polygon:nth-child(5) { stroke: color-mix(in srgb, var(--text-muted) 62%, transparent); }
.radar-player-fill { opacity: .13; }
.radar-player-line { stroke-width: 3; stroke-linejoin: round; vector-effect: non-scaling-stroke; animation: radar-enter 220ms cubic-bezier(.23, 1, .32, 1) both; }
.radar-axis-label { fill: var(--text); font-size: 14px; font-weight: 800; letter-spacing: 0; }
.radar-axis-value { fill: var(--text-muted); font-family: Consolas, "Cascadia Mono", monospace; font-size: 10px; font-weight: 500; }
figcaption { display: flex; flex-wrap: wrap; justify-content: center; gap: 10px 18px; color: var(--text-muted); font-size: 11px; }
.radar-scale-note { margin: 8px 0 0; color: var(--text-muted); font-size: 11px; text-align: center; }
figcaption span { display: inline-flex; align-items: center; gap: 7px; }
figcaption i { width: 24px; border-top-width: 3px; }
figcaption small { padding-left: 2px; color: var(--warning); font-size: 9px; }
@keyframes radar-enter { from { opacity: 0; transform: scale(.97); transform-origin: 320px 320px; } to { opacity: 1; transform: scale(1); } }
@media (prefers-reduced-motion: reduce) { .radar-player-line { animation: none; } }
@media (max-width: 700px) { .radar-axis-label { font-size: 16px; } .radar-axis-value { display: none; } }
</style>
