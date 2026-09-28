<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { appConfig } from '@/config/app'
import ThreeStage from '@/components/three/ThreeStage.vue'
import { createAcknowledgementScene } from '@/features/acknowledgement/scene'
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
onMounted(() => {
  window.addEventListener('keydown', onKeydown)
  skipButton.value?.focus()
  frame = requestAnimationFrame(tick)
})
onBeforeUnmount(() => { cancelAnimationFrame(frame); window.removeEventListener('keydown', onKeydown) })
</script>
<template>
  <section class="cinema-overlay intro-overlay ink-ack-scene" role="dialog" aria-modal="true" aria-label="水墨江南鸣谢长廊">
    <header class="cinema-topbar"><div><span>CS2AS · 水墨江南</span><strong>{{ appConfig.appVersion }}</strong></div><button ref="skipButton" type="button" class="cinema-text-button" @click="close">跳过</button></header>
    <ThreeStage :factory="(canvas, size) => createAcknowledgementScene(canvas, size, () => ({ elapsedMs: elapsed, durationMs: durationMs, supporterLockMs: 4200, loading: false, cards, mode: 'cinematic' }))" />
    <div class="ink-ack-copy"><p class="cinema-kicker">幸与诸君同路</p><h1>上游项目鸣谢</h1><p aria-live="polite">镜头正在经过：{{ activeProject }}</p><small>因部分赞助者不愿公开昵称，我们不再展示赞助者或 B 站充电记录。感谢每一位支持与同行的人。</small></div>
    <div class="cinema-progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
  </section>
</template>
