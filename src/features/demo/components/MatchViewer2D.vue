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
onMounted(() => { viewerMounted(props.mode); resizeObserver = new ResizeObserver(resize); if (shell.value) resizeObserver.observe(shell.value); window.addEventListener('resize', resize); document.addEventListener('visibilitychange', onVisibilityChange); load() })
onBeforeUnmount(() => { loadGeneration++; radarGeneration++; stopPlayback(); viewerUnmounted(); resizeObserver?.disconnect(); window.removeEventListener('resize', resize); document.removeEventListener('visibilitychange', onVisibilityChange); radar.onload = null; radar.onerror = null; radar.src = '' })
</script>

<template>
  <div class="spatial-workbench">
    <aside class="spatial-controls">
      <label>回合<select v-model.number="roundNumber"><option v-for="round in rounds" :key="round.roundNumber" :value="round.roundNumber">第 {{ round.roundNumber }} 回合</option></select></label>
      <label>采样<select v-model.number="samplingHz"><option :value="4">4 Hz</option><option :value="8">8 Hz</option><option :value="16">16 Hz</option></select></label>
      <label v-if="map?.thresholdZ">楼层<select v-model="layer"><option value="upper">上层</option><option value="lower">下层</option></select></label>
      <label v-if="mode === 'viewer'">速度<select v-model.number="speed"><option :value="0.5">0.5x</option><option :value="1">1x</option><option :value="2">2x</option><option :value="4">4x</option></select></label>
      <div class="spatial-actions"><button class="icon-button" type="button" :disabled="loading" title="刷新空间数据" aria-label="刷新空间数据" @click="load"><RefreshCw :size="17" /></button><button v-if="mode === 'viewer'" class="icon-button" type="button" :disabled="!ticks.length" :title="playing ? '暂停' : '播放'" :aria-label="playing ? '暂停' : '播放'" @click="playing = !playing"><Pause v-if="playing" :size="17" /><Play v-else :size="17" /></button></div>
      <div v-if="mode === 'viewer'" class="team-legend" aria-label="阵营图例"><span><i data-team="ct" />CT 阵营</span><span><i data-team="t" />T 阵营</span></div>
      <button v-if="!points.length" class="primary-button" type="button" :disabled="generating" @click="generate"><LoaderCircle v-if="generating" :size="17" class="spinning" />生成空间数据</button>
      <p v-if="error" role="status">{{ error }}</p>
      <dl><div><dt>坐标点</dt><dd>{{ points.length || '--' }}</dd></div><div v-if="mode === 'viewer'"><dt>当前 Tick</dt><dd>{{ currentTick || '--' }}</dd></div></dl>
    </aside>
    <div ref="shell" class="spatial-canvas-shell"><canvas ref="canvas" :aria-label="mode === 'viewer' ? '二维地图回放' : '位置热力图'" /><input v-if="mode === 'viewer' && ticks.length" v-model.number="tickIndex" type="range" :max="Math.max(0, ticks.length - 1)" min="0" aria-label="回放进度" /></div>
  </div>
</template>
