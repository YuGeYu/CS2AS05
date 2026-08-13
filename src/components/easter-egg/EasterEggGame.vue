<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { Footprints, X } from 'lucide-vue-next'
import ThreeStage from '@/components/three/ThreeStage.vue'
import type { StageInteraction, ThreeStageFactory } from '@/components/three/stage'
import { REFERENCE_PROJECTS } from '@/features/support/reference-projects'
import type { IntroAcknowledgementCard, IntroData } from '@/features/intro/types'
import { loadCachedIntroData, loadIntroData } from '@/services/intro-data'

const emit = defineEmits<{ close: [] }>()
const dialog = ref<HTMLElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)
const inputLayer = ref<HTMLElement | null>(null)
const stage = ref<InstanceType<typeof ThreeStage> | null>(null)
const failed = ref(false)
const data = ref<IntroData>(loadCachedIntroData())
let activePointerId: number | null = null
let closing = false

const cards = computed<IntroAcknowledgementCard[]>(() => [
  ...REFERENCE_PROJECTS.map(item => ({ id: `upstream:${item.id}`, kind: 'upstream' as const, eyebrow: '感谢上游项目', title: item.repository, message: item.description, detail: item.license || item.group, updatedAt: item.repository })),
  ...data.value.supporters.slice(0, 8).map(item => ({ id: `supporter:${item.id}`, kind: 'supporter' as const, eyebrow: '鸣谢同路人', title: item.nickname || '青锋无名客', message: item.message || '长夜执剑，幸与诸君同路。', detail: `¥${(item.amountCents / 100).toFixed(2)}`, updatedAt: item.updatedAt })),
])
const sceneFactory: ThreeStageFactory = async (canvas, size) => {
  const { createContributionGalleryScene } = await import('@/features/easter-egg/game-scene')
  return createContributionGalleryScene(canvas, size, () => cards.value)
}

function releasePointer() {
  if (activePointerId !== null && inputLayer.value?.hasPointerCapture?.(activePointerId)) inputLayer.value.releasePointerCapture(activePointerId)
  activePointerId = null
}
function close() { if (closing) return; closing = true; releasePointer(); emit('close') }
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') { close(); return }
  if (event.key === 'Tab') {
    event.preventDefault(); closeButton.value?.focus(); return
  }
  if (['w', 'a', 's', 'd', 'W', 'A', 'S', 'D', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'].includes(event.key)) {
    event.preventDefault(); stage.value?.interact({ type: 'key-down', key: event.key.toLowerCase() })
  }
}
function onKeyup(event: KeyboardEvent) { stage.value?.interact({ type: 'key-up', key: event.key.toLowerCase() }) }
function pointer(event: PointerEvent, type: StageInteraction['type']) {
  const target = inputLayer.value
  if (!target || !['pointer-down', 'pointer-move', 'pointer-up'].includes(type)) return
  if (type === 'pointer-down') { activePointerId = event.pointerId; target.setPointerCapture?.(event.pointerId) }
  else if (activePointerId !== event.pointerId) return
  const rect = target.getBoundingClientRect()
  stage.value?.interact({ type, x: (event.clientX - rect.left) / rect.width, y: (event.clientY - rect.top) / rect.height })
  if (type === 'pointer-up') releasePointer()
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  window.addEventListener('keyup', onKeyup)
  await nextTick(); closeButton.value?.focus()
  void loadIntroData().then(value => { data.value = value })
})
onBeforeUnmount(() => { releasePointer(); window.removeEventListener('keydown', onKeydown); window.removeEventListener('keyup', onKeyup) })
</script>

<template>
  <section ref="dialog" class="cinema-overlay game-overlay gallery-overlay" role="dialog" aria-modal="true" aria-label="贡献陈列馆">
    <ThreeStage v-if="!failed" ref="stage" :factory="sceneFactory" @error="failed = true" @context-lost="failed = true" />
    <div v-else class="cinema-static" aria-hidden="true" />
    <div ref="inputLayer" class="game-input-layer gallery-input-layer" aria-hidden="true" @pointerdown="pointer($event, 'pointer-down')" @pointermove="pointer($event, 'pointer-move')" @pointerup="pointer($event, 'pointer-up')" @pointercancel="pointer($event, 'pointer-up')" />
    <header class="gallery-heading"><p>贡献陈列馆</p><h1>众行者，共铸此间</h1><span><Footprints :size="16" />WASD / 方向键行走 · 拖动鼠标环顾</span></header>
    <aside class="gallery-artifact"><small>中央珍藏</small><strong>唐代彩绘仕女俑</strong><span>盛唐风华 · 高髻宽袖与彩绘余晖</span></aside>
    <button ref="closeButton" type="button" class="cinema-icon-button" title="关闭" aria-label="关闭贡献陈列馆" @pointerdown.stop="close" @pointerup.stop @pointercancel.stop @click.stop="close"><X :size="20" /></button>
    <div v-if="failed" class="game-center game-result"><p>贡献陈列馆</p><strong>当前设备无法启动三维展厅</strong><button type="button" class="cinema-primary-button" @click.stop="close">关闭</button></div>
    <footer class="game-credit">中央展品来源：cultural-relics-museum · MulanPSL-2.0</footer>
  </section>
</template>
