<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { appConfig } from '@/config/app'
import ThreeStage from '@/components/three/ThreeStage.vue'
import { createAcknowledgementScene } from '@/features/acknowledgement/scene'
import { loadIntroData } from '@/services/intro-data'
import { STATIC_REFERENCE_PROJECTS } from '@/features/intro/static-data'
import type { IntroAcknowledgementCard } from '@/features/intro/types'
const durationMs = computed(() => Math.max(9600, Math.min(42000, Math.max(1, cards.value.length) * 1800)))
const emit = defineEmits<{ close: [] }>()
const skipButton = ref<HTMLButtonElement | null>(null)
const elapsed = ref(0)
let frame = 0; let started = 0; let closed = false
const projects = STATIC_REFERENCE_PROJECTS
const cards = ref<IntroAcknowledgementCard[]>(projects.map(item => ({ id: `upstream:${item.id}`, kind: 'upstream', eyebrow: '上游项目', title: item.repository, message: item.description, detail: item.license || item.group, updatedAt: '' })))
const focusRows = computed(() => cards.value.map(item => item.title))
const progress = computed(() => Math.min(100, elapsed.value / durationMs.value * 100))
const activeProject = computed(() => focusRows.value[Math.min(focusRows.value.length - 1, Math.floor(Math.max(0, elapsed.value - 600) / Math.max(1, (durationMs.value - 600) / Math.max(1, focusRows.value.length))))] || '水墨长廊')
function close() { if (closed) return; closed = true; cancelAnimationFrame(frame); emit('close') }
function tick(now: number) { if (!started) started = now; elapsed.value = now - started; if (elapsed.value >= durationMs.value) close(); else frame = requestAnimationFrame(tick) }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape' || event.key === 'Tab') { event.preventDefault(); close() } }
onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  skipButton.value?.focus()
  const data = await loadIntroData()
  cards.value = [
    ...projects.map(item => ({ id: `upstream:${item.id}`, kind: 'upstream' as const, eyebrow: '上游项目', title: item.repository, message: item.description, detail: item.license || item.group, updatedAt: '' })),
    ...data.supporters.map(item => ({
      id: `supporter:${item.id}`,
      kind: 'supporter' as const,
      eyebrow: item.platform === 'bilibili' ? 'B站充电鸣谢' : '公开赞助',
      title: item.nickname || '匿名同路人',
      message: item.message || '感谢你的支持与同行。',
      detail: item.unit === 'beike' ? `后台可见 ${item.visibleAmount ?? 0} 贝壳 · 暂定约 ¥${(item.visibleAmount ?? 0).toFixed(2)}` : `公开支持 · ¥${((item.amountCents ?? 0) / 100).toFixed(2)}`,
      updatedAt: item.updatedAt,
    })),
  ]
    frame = requestAnimationFrame(tick)
})
onBeforeUnmount(() => { cancelAnimationFrame(frame); window.removeEventListener('keydown', onKeydown) })
</script>
<template>
  <section class="cinema-overlay intro-overlay ink-ack-scene" role="dialog" aria-modal="true" aria-label="水墨江南鸣谢长廊">
    <header class="cinema-topbar"><div><span>CS2AS · 水墨江南</span><strong>{{ appConfig.appVersion }}</strong></div><button ref="skipButton" type="button" class="cinema-text-button" @click="close">跳过</button></header>
    <ThreeStage :factory="(canvas, size) => createAcknowledgementScene(canvas, size, () => ({ elapsedMs: elapsed, durationMs: durationMs, supporterLockMs: 4200, loading: false, cards, mode: 'cinematic' }))" />
    <div class="ink-ack-copy"><p class="cinema-kicker">幸与诸君同路</p><h1>上游项目、赞助与公开鸣谢</h1><p aria-live="polite">镜头正在经过：{{ activeProject }}</p><small>金额以当前可核实记录为准；B站贝壳暂按 1 贝壳 = ¥1 展示，不代表完整充电总额。</small></div>
    <div class="cinema-progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
  </section>
</template>
