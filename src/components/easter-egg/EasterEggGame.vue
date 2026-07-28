<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { RotateCcw, X } from 'lucide-vue-next'
import ThreeStage from '@/components/three/ThreeStage.vue'
import type { ThreeStageFactory, StageInteraction } from '@/components/three/stage'
import type { GameSceneEvent } from '@/features/easter-egg/game-scene'

const emit = defineEmits<{ close: [] }>()
const dialog = ref<HTMLElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)
const inputLayer = ref<HTMLElement | null>(null)
const stage = ref<InstanceType<typeof ThreeStage> | null>(null)
const status = ref<'countdown' | 'playing' | 'finished' | 'failed'>('countdown')
const countdown = ref(3)
const seconds = ref(60)
const score = ref(0); const combo = ref(0); const bestCombo = ref(0); const lives = ref(3)
const session = ref(0)
let interval: ReturnType<typeof setInterval> | null = null
let activePointerId: number | null = null
let closing = false

const sceneFactory = computed<ThreeStageFactory>(() => async (canvas, size) => {
  const { createGameScene } = await import('@/features/easter-egg/game-scene')
  return createGameScene(canvas, size, updateScore)
})
function updateScore(event: GameSceneEvent) {
  score.value = event.score; combo.value = event.combo; bestCombo.value = event.bestCombo; lives.value = event.lives
  if (!event.lives && status.value === 'playing') finish()
}
function clearClock() { if (interval) clearInterval(interval); interval = null }
function startClock() {
  clearClock(); status.value = 'countdown'; countdown.value = 3; seconds.value = 60
  interval = setInterval(() => {
    if (status.value === 'countdown') {
      countdown.value -= 1
      if (countdown.value <= 0) status.value = 'playing'
    } else if (status.value === 'playing') {
      seconds.value -= 1
      if (seconds.value <= 0) finish()
    }
  }, 1_000)
}
function releaseActivePointerCapture() {
  if (activePointerId !== null && inputLayer.value?.hasPointerCapture?.(activePointerId)) inputLayer.value.releasePointerCapture(activePointerId)
  activePointerId = null
}
function finish() { status.value = 'finished'; clearClock(); releaseActivePointerCapture() }
function fail() { status.value = 'failed'; clearClock(); releaseActivePointerCapture() }
function retry() { releaseActivePointerCapture(); score.value = 0; combo.value = 0; bestCombo.value = 0; lives.value = 3; session.value += 1; startClock() }
function close() {
  if (closing) return
  closing = true
  clearClock()
  releaseActivePointerCapture()
  emit('close')
}
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close()
  if (event.key === 'Tab') {
    const buttons = [...(dialog.value?.querySelectorAll<HTMLButtonElement>('button:not(:disabled)') ?? [])]
    if (!buttons.length) return
    event.preventDefault()
    const current = buttons.indexOf(document.activeElement as HTMLButtonElement)
    const next = event.shiftKey ? (current <= 0 ? buttons.length - 1 : current - 1) : (current + 1) % buttons.length
    buttons[next]?.focus()
  }
}
function pointer(event: PointerEvent, type: StageInteraction['type']) {
  if (status.value !== 'playing') return
  const target = inputLayer.value
  if (!target) return
  if (type === 'pointer-down') {
    if (activePointerId !== null) return
    activePointerId = event.pointerId
    target.setPointerCapture?.(event.pointerId)
  } else if (event.pointerId !== activePointerId) return
  const rect = target.getBoundingClientRect()
  stage.value?.interact({ type, x: (event.clientX - rect.left) / rect.width, y: (event.clientY - rect.top) / rect.height })
  if (type === 'pointer-up') releaseActivePointerCapture()
}

onMounted(async () => { window.addEventListener('keydown', onKeydown); await nextTick(); closeButton.value?.focus(); startClock() })
onBeforeUnmount(() => { clearClock(); releaseActivePointerCapture(); window.removeEventListener('keydown', onKeydown) })
</script>

<template>
  <section ref="dialog" class="cinema-overlay game-overlay" role="dialog" aria-modal="true" aria-label="青冥试剑">
    <ThreeStage v-if="status !== 'failed'" :key="session" ref="stage" :factory="sceneFactory" :active="status === 'playing'" @error="fail" @context-lost="fail" />
    <div v-else class="cinema-static" aria-hidden="true" />
    <div ref="inputLayer" class="game-input-layer" aria-hidden="true" @pointerdown="pointer($event, 'pointer-down')" @pointermove="pointer($event, 'pointer-move')" @pointerup="pointer($event, 'pointer-up')" @pointercancel="pointer($event, 'pointer-up')" />
    <header class="game-hud">
      <div><span>得分</span><strong>{{ score }}</strong></div><div><span>连击</span><strong>{{ combo }}</strong></div><div><span>时间</span><strong>{{ seconds }}</strong></div><div><span>剑心</span><strong>{{ '◆'.repeat(lives) || '—' }}</strong></div>
    </header>
    <button ref="closeButton" type="button" class="cinema-icon-button" title="关闭" aria-label="关闭青冥试剑" @pointerdown.stop="close" @pointerup.stop @pointercancel.stop @click.stop="close"><X :size="20" /></button>
    <div v-if="status === 'countdown'" class="game-center"><p>青冥试剑</p><strong>{{ countdown }}</strong><span>挥动鼠标或触摸斩断青色符印，避开铜印</span></div>
    <div v-else-if="status === 'finished'" class="game-center game-result"><p>试剑已毕</p><strong>{{ score }}</strong><span>最高连击 {{ bestCombo }}</span><button type="button" class="cinema-primary-button" @click.stop="retry"><RotateCcw :size="18" /> 再试一局</button></div>
    <div v-else-if="status === 'failed'" class="game-center game-result"><p>青冥试剑</p><strong>当前设备无法启动彩蛋</strong><button type="button" class="cinema-primary-button" @click.stop="close">关闭</button></div>
    <footer class="game-credit">原创程序化场景 · Three.js MIT</footer>
    <div class="sr-only" aria-live="polite">{{ status === 'finished' ? `游戏结束，得分 ${score}，最高连击 ${bestCombo}` : '' }}</div>
  </section>
</template>
