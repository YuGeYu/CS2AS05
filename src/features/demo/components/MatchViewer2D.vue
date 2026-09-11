<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { LoaderCircle, Pause, Play, RefreshCw } from 'lucide-vue-next'
import * as api from '@/services/tauri/demo'
import { findCs2Map } from '@/features/demo/viewer/coordinates'
import { uniqueTicks } from '@/features/demo/viewer/frame-index'
import { drawViewer } from '@/features/demo/viewer/draw-viewer'
import { drawHeatmap } from '@/features/demo/viewer/draw-heatmap'
import { viewerFrame, viewerLoaded, viewerMounted, viewerPlaying, viewerUnmounted } from '@/services/preflight'
import type { MatchRoundSummary, PositionPoint } from '@/types/demo'

const props = defineProps<{ demoId: number; mapName: string | null; rounds: MatchRoundSummary[]; mode: 'viewer' | 'heatmap' }>()
const canvas = ref<HTMLCanvasElement | null>(null)
const shell = ref<HTMLElement | null>(null)
const roundNumber = ref(props.rounds[0]?.roundNumber ?? 1)
const samplingHz = ref(8)
const points = ref<PositionPoint[]>([])
const loading = ref(false)
const generating = ref(false)
const error = ref('')
const playing = ref(false)
const tickIndex = ref(0)
const layer = ref<'upper' | 'lower'>('upper')
const speed = ref(1)
const map = computed(() => props.mapName ? findCs2Map(props.mapName) : undefined)
const ticks = computed(() => uniqueTicks(points.value))
const currentTick = computed(() => ticks.value[tickIndex.value] ?? 0)
const modeLabel = computed(() => props.mode === 'viewer' ? '地图回放' : '位置热力图')
const dataState = computed(() => loading.value ? '正在读取空间数据' : error.value ? '需要检查数据' : points.value.length ? '空间数据已就绪' : '尚未生成空间数据')
const radarAssets = import.meta.glob('../../../assets/maps/cs2/radars/*.png', { eager: true, query: '?url', import: 'default' }) as Record<string, string>
let resizeObserver: ResizeObserver | undefined
let animationFrame = 0
let lastFrame = 0
let radar = new Image()
let loadGeneration = 0
let radarGeneration = 0

const radarUrl = computed(() => {
  if (!props.mapName) return ''
  const suffix = layer.value === 'lower' ? `${props.mapName}_lower.png` : `${props.mapName}.png`
  return Object.entries(radarAssets).find(([path]) => path.endsWith(suffix))?.[1] || ''
})

function resize() {
  const element = canvas.value
  const container = shell.value
  if (!element || !container) return
  const availableHeight = Math.max(280, window.innerHeight - 240)
  const size = Math.max(280, Math.floor(Math.min(container.clientWidth - 28, availableHeight, 640)))
  const ratio = window.devicePixelRatio || 1
  element.width = Math.floor(size * ratio)
  element.height = Math.floor(size * ratio)
  element.style.width = `${size}px`
  element.style.height = `${size}px`
  draw()
}

function draw() {
  const element = canvas.value
  const metadata = map.value
  if (!element || !metadata) return
  const context = element.getContext('2d')
  if (!context) return
  const ratio = window.devicePixelRatio || 1
  const size = element.width / ratio
  const image = radar.complete && radar.naturalWidth ? radar : undefined
  if (props.mode === 'heatmap') {
    drawHeatmap(context, size, ratio, metadata, image, points.value, layer.value)
    return
  }
  drawViewer({ context, size, ratio, metadata, radar: image, points: points.value, layer: layer.value, tick: currentTick.value })
}

function stopPlayback() {
  playing.value = false
  if (animationFrame) cancelAnimationFrame(animationFrame)
  animationFrame = 0
}

async function load() {
  const generation = ++loadGeneration
  loading.value = true
  error.value = ''
  stopPlayback()
  try {
    const result = await api.getRoundPositions(props.demoId, roundNumber.value, samplingHz.value)
    if (generation !== loadGeneration) return
    points.value = result.points
    tickIndex.value = 0
    viewerLoaded(props.mode, result.points[0]?.tick ?? 0, result.points.length)
    await nextTick()
    resize()
  } catch (reason) {
    if (generation !== loadGeneration) return
    points.value = []
    error.value = String(reason)
    draw()
  } finally {
    if (generation === loadGeneration) loading.value = false
  }
}

async function generate() {
  generating.value = true
  error.value = ''
  try {
    await api.ensureSpatialAnalysis(props.demoId, samplingHz.value)
    error.value = '空间任务已入队，完成后点击刷新。'
  } catch (reason) {
    error.value = String(reason)
  } finally {
    generating.value = false
  }
}

function animate(time: number) {
  if (!playing.value) return
  if (time - lastFrame >= 1000 / (samplingHz.value * speed.value)) {
    if (tickIndex.value >= ticks.value.length - 1) { stopPlayback(); return }
    else tickIndex.value++
    lastFrame = time
  }
  viewerFrame(time, currentTick.value, points.value.length)
  if (playing.value) animationFrame = requestAnimationFrame(animate)
}

watch(tickIndex, draw)
watch([roundNumber, samplingHz, () => props.demoId], load)
watch(radarUrl, (url) => {
  const generation = ++radarGeneration
  radar.onload = null
  radar.onerror = null
  radar.src = ''
  radar = new Image()
  radar.onload = () => { if (generation === radarGeneration) { error.value = ''; draw() } }
  radar.onerror = () => { if (generation === radarGeneration) { error.value = url ? '雷达资源加载失败。' : '该地图没有可用雷达资源。'; draw() } }
  if (url) radar.src = url
  else error.value = map.value ? '该楼层没有可用雷达资源。' : '该地图没有固定坐标元数据。'
}, { immediate: true })
watch(playing, (value) => { viewerPlaying(value); if (!value) { if (animationFrame) cancelAnimationFrame(animationFrame); animationFrame = 0 } else { lastFrame = 0; animationFrame = requestAnimationFrame(animate) } })
watch(layer, draw)
function onVisibilityChange() { if (document.hidden) stopPlayback() }
function onTimelineKeydown(event: KeyboardEvent) {
  if (!ticks.value.length) return
  if (event.key === 'ArrowLeft') { event.preventDefault(); tickIndex.value = Math.max(0, tickIndex.value - 1) }
  if (event.key === 'ArrowRight') { event.preventDefault(); tickIndex.value = Math.min(ticks.value.length - 1, tickIndex.value + 1) }
  if (event.key === 'Home') { event.preventDefault(); tickIndex.value = 0 }
  if (event.key === 'End') { event.preventDefault(); tickIndex.value = ticks.value.length - 1 }
  if (event.key === ' ' && props.mode === 'viewer') { event.preventDefault(); playing.value = !playing.value }
}
onMounted(() => { viewerMounted(props.mode); resizeObserver = new ResizeObserver(resize); if (shell.value) resizeObserver.observe(shell.value); window.addEventListener('resize', resize); document.addEventListener('visibilitychange', onVisibilityChange); load() })
onBeforeUnmount(() => { loadGeneration++; radarGeneration++; stopPlayback(); viewerUnmounted(); resizeObserver?.disconnect(); window.removeEventListener('resize', resize); document.removeEventListener('visibilitychange', onVisibilityChange); radar.onload = null; radar.onerror = null; radar.src = '' })
</script>

<template>
  <div class="tactical-workbench" :data-mode="mode">
    <header class="tactical-context">
      <div class="tactical-context-title"><span class="tactical-kicker">空间复盘 / {{ mode === 'viewer' ? 'REPLAY' : 'HEATMAP' }}</span><h3>{{ modeLabel }}</h3><p>{{ mapName || '未知地图' }} · 第 {{ roundNumber }} 回合</p></div>
      <div class="tactical-context-meta"><span class="tactical-status" :data-state="points.length ? 'ready' : loading ? 'loading' : 'idle'"><i />{{ dataState }}</span><span v-if="map?.thresholdZ" class="tactical-chip">{{ layer === 'upper' ? '上层' : '下层' }}</span></div>
    </header>
    <div class="tactical-main">
      <section class="tactical-stage" aria-label="战术地图观察区">
        <div class="tactical-stage-canvas" ref="shell"><canvas ref="canvas" :aria-label="mode === 'viewer' ? '二维地图回放' : '位置热力图'" /><div class="tactical-overlay tactical-overlay--top">第 {{ roundNumber }} 回合 · {{ layer === 'upper' ? '上层' : '下层' }}</div><div class="tactical-overlay tactical-overlay--bottom">{{ points.length ? `${points.length.toLocaleString()} 个坐标点` : '等待空间数据' }}</div></div>
        <div v-if="mode === 'viewer' && ticks.length" class="tactical-timeline" role="group" aria-label="回放时间轨" tabindex="0" @keydown="onTimelineKeydown">
          <button class="tactical-play" type="button" :disabled="loading" :title="playing ? '暂停回放' : '播放回放'" :aria-label="playing ? '暂停' : '播放'" @click="playing = !playing"><Pause v-if="playing" :size="18" /><Play v-else :size="18" /></button>
          <input v-model.number="tickIndex" type="range" :max="Math.max(0, ticks.length - 1)" min="0" aria-label="回放进度" :aria-valuetext="`第 ${tickIndex + 1} 帧，Tick ${currentTick}`" />
          <output class="tactical-tick">Tick {{ currentTick || '--' }}</output>
          <select v-model.number="speed" aria-label="回放速度"><option :value="0.5">0.5x</option><option :value="1">1x</option><option :value="2">2x</option><option :value="4">4x</option></select>
        </div>
      </section>
      <aside class="tactical-inspector" aria-label="预览检查器">
        <div class="tactical-inspector-heading"><div><span class="tactical-kicker">INSPECTOR</span><h4>观察参数</h4></div><button class="icon-button" type="button" :disabled="loading" title="刷新空间数据" aria-label="刷新空间数据" @click="load"><RefreshCw :size="17" /></button></div>
        <div class="tactical-fields"><label>回合<select v-model.number="roundNumber"><option v-for="round in rounds" :key="round.roundNumber" :value="round.roundNumber">第 {{ round.roundNumber }} 回合</option></select></label><label>采样率<select v-model.number="samplingHz"><option :value="4">4 Hz</option><option :value="8">8 Hz</option><option :value="16">16 Hz</option></select></label><label v-if="map?.thresholdZ">楼层<select v-model="layer"><option value="upper">上层</option><option value="lower">下层</option></select></label></div>
        <div v-if="mode === 'viewer'" class="tactical-legend" aria-label="阵营图例"><span><i data-team="ct" />CT 阵营</span><span><i data-team="t" />T 阵营</span><span><i data-team="observer" />观战者</span></div>
        <div class="tactical-metrics"><div><span>坐标点</span><strong>{{ points.length || '--' }}</strong></div><div><span>{{ mode === 'viewer' ? '当前 Tick' : '采样率' }}</span><strong>{{ mode === 'viewer' ? currentTick || '--' : `${samplingHz} Hz` }}</strong></div></div>
        <div v-if="!points.length" class="tactical-empty"><LoaderCircle v-if="generating" class="spinning" :size="18" /><p>{{ error || '当前回合还没有空间数据。' }}</p><button class="primary-button" type="button" :disabled="generating" @click="generate">生成空间数据</button></div>
        <p v-if="error && points.length" class="tactical-error" role="status">{{ error }}</p>
        <p class="tactical-hint">{{ mode === 'viewer' ? '方向键逐帧 · Home/End 跳转 · 空格播放' : '切换回合或楼层查看采样分布' }}</p>
      </aside>
    </div>
  </div>
</template>
