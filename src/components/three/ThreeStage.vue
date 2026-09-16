<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import type { StageInteraction, StageSize, ThreeStageController, ThreeStageError, ThreeStageFactory } from './stage'

const props = withDefaults(defineProps<{ factory: ThreeStageFactory; animate?: boolean; active?: boolean }>(), {
  animate: true,
  active: true,
})
const emit = defineEmits<{ ready: []; error: [error: ThreeStageError]; contextLost: [] }>()

const host = ref<HTMLDivElement | null>(null)
const canvas = ref<HTMLCanvasElement | null>(null)
let controller: ThreeStageController | null = null
let observer: ResizeObserver | null = null
let frameId: number | null = null
let lastTime: number | null = null
let slowFrames = 0
let disposed = false
let running = false
let contextLost = false
let frameCount = 0
let lastFrameAt: number | null = null

function currentSize(): { size: StageSize; renderable: boolean } {
  const rect = host.value?.getBoundingClientRect()
  const width = Math.round(rect?.width ?? 0)
  const height = Math.round(rect?.height ?? 0)
  return { size: { width: Math.max(1, width), height: Math.max(1, height), dpr: Math.min(window.devicePixelRatio || 1, slowFrames > 45 ? 1 : 1.5) }, renderable: width >= 2 && height >= 2 }
}

function shouldRun() {
  return !disposed && !contextLost && controller !== null && props.animate !== false && !document.hidden && currentSize().renderable
}

function stopLoop() {
  running = false
  if (frameId !== null) cancelAnimationFrame(frameId)
  frameId = null
  lastTime = null
}

function ensureLoop() {
  if (running || !shouldRun()) return
  running = true
  lastTime = null
  frameId = requestAnimationFrame(onFrame)
}

function onFrame(time: number) {
  frameId = null
  if (!shouldRun()) { running = false; return }
  try {
    const delta = lastTime === null ? 0 : Math.min((time - lastTime) / 1000, 0.05)
    lastTime = time
    slowFrames = delta > 0.024 ? slowFrames + 1 : Math.max(0, slowFrames - 2)
    if (slowFrames === 46) {
      controller?.setLowPerformance?.(true)
      controller?.resize(currentSize().size)
    }
    controller?.frame(time / 1000, delta)
    frameCount += 1
    lastFrameAt = time
  } catch (cause) {
    running = false
    const message = cause instanceof Error ? cause.message : 'WebGL 动画运行失败'
    console.error('[ThreeStage] frame failed', cause)
    emit('error', { stage: 'frame', code: 'THREE_STAGE_FRAME_FAILED', message })
    return
  }
  if (shouldRun()) frameId = requestAnimationFrame(onFrame)
  else running = false
}

function onVisibilityChange() {
  if (document.hidden) stopLoop()
  else ensureLoop()
}

function onContextLost(event: Event) {
  event.preventDefault()
  contextLost = true
  stopLoop()
  emit('contextLost')
}

function interact(interaction: StageInteraction) {
  controller?.interact?.(interaction)
}

defineExpose({ interact, getDiagnostics: () => ({ frameCount, lastFrameAt, running, disposed, contextLost, hasController: controller !== null, animate: props.animate, hidden: document.hidden, renderable: currentSize().renderable }) })

watch(() => props.animate, (animate) => {
  if (animate === false) stopLoop()
  else ensureLoop()
})

watch(() => props.active, active => controller?.setActive?.(active !== false))

onMounted(async () => {
  if (!canvas.value || !host.value) return
  canvas.value.addEventListener('webglcontextlost', onContextLost)
  document.addEventListener('visibilitychange', onVisibilityChange)
  window.addEventListener('focus', ensureLoop)
  if (typeof ResizeObserver === 'undefined') {
    // jsdom and restricted WebViews may not expose ResizeObserver; the first
    // factory size is still valid and the render loop remains usable.
    observer = null
  } else observer = new ResizeObserver(() => {
    const current = currentSize()
    if (!current.renderable) { stopLoop(); return }
    controller?.resize(current.size)
    ensureLoop()
  })
  observer?.observe(host.value)
  try {
    controller = await props.factory(canvas.value, currentSize().size)
    if (disposed) {
      controller.dispose()
      controller = null
      return
    }
    controller.setActive?.(props.active !== false)
    const current = currentSize()
    if (current.renderable) controller.resize(current.size)
    emit('ready')
    ensureLoop()
  } catch (cause) {
    const message = cause instanceof Error ? cause.message : 'WebGL 初始化失败'
    console.error('[ThreeStage] factory failed', cause)
    emit('error', { stage: 'factory', code: 'THREE_STAGE_FACTORY_FAILED', message })
  }
})

onBeforeUnmount(() => {
  disposed = true
  stopLoop()
  observer?.disconnect()
  document.removeEventListener('visibilitychange', onVisibilityChange)
  window.removeEventListener('focus', ensureLoop)
  canvas.value?.removeEventListener('webglcontextlost', onContextLost)
  controller?.dispose()
  controller = null
})
</script>

<template>
  <div ref="host" class="three-stage"><canvas ref="canvas" aria-hidden="true" /></div>
</template>
