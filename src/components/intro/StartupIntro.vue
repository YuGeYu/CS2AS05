<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import ThreeStage from '@/components/three/ThreeStage.vue'
import type { ThreeStageFactory } from '@/components/three/stage'
import { appConfig } from '@/config/app'
import { REFERENCE_PROJECTS } from '@/features/support/reference-projects'
import type { IntroAcknowledgementCard, IntroData } from '@/features/intro/types'
import { loadCachedIntroData, loadIntroData } from '@/services/intro-data'

const INTRO_DURATION_MS = 9_600
const SUPPORTER_LOCK_MS = 2_800

const emit = defineEmits<{ close: [] }>()
const skipButton = ref<HTMLButtonElement | null>(null)
const elapsed = ref(0)
const failed = ref(false)
const loaded = ref(false)
const locked = ref(false)
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
const data = ref<IntroData>(loadCachedIntroData())
const lockedCards = ref<IntroAcknowledgementCard[]>([])
let timer = 0
let startedAt = 0
let closed = false

const supporters = computed(() => data.value.supporters.slice(0, 8))
const candidateCards = computed<IntroAcknowledgementCard[]>(() => [
  ...REFERENCE_PROJECTS.map(item => ({ id: `upstream:${item.id}`, kind: 'upstream' as const, eyebrow: '感谢上游项目', title: item.repository, message: item.description, detail: item.license || item.group, updatedAt: item.repository })),
  ...supporters.value.map(item => ({ id: `supporter:${item.id}`, kind: 'supporter' as const, eyebrow: '鸣谢同路人', title: item.nickname || '青锋无名客', message: item.message || '长夜执剑，幸与诸君同路。', detail: `¥${(item.amountCents / 100).toFixed(2)}`, updatedAt: item.updatedAt })),
])
const cards = computed(() => locked.value ? lockedCards.value : candidateCards.value)
const duration = computed(() => reducedMotion ? 1_400 : INTRO_DURATION_MS)
const currentCard = computed(() => {
  if (elapsed.value < 650) return undefined
  const upstreamIndex = Math.min(4, Math.floor((elapsed.value - 650) / 470))
  if (elapsed.value < SUPPORTER_LOCK_MS) return cards.value[upstreamIndex]
  const supporterCards = cards.value.filter(card => card.kind === 'supporter')
  if (!supporterCards.length) return undefined
  const index = Math.min(supporterCards.length - 1, Math.floor((elapsed.value - SUPPORTER_LOCK_MS) / Math.max(1, 4_800 / supporterCards.length)))
  return supporterCards[index]
})
const progress = computed(() => Math.min(100, elapsed.value / duration.value * 100))
const supporterStatus = computed(() => {
  if (!loaded.value) return '正在连接鸣谢长廊'
  if (supporters.value.length) return ''
  return data.value.sources.supporters === 'network' ? '鸣谢长廊静候同路人' : '鸣谢长廊暂未连接'
})

const sceneFactory: ThreeStageFactory = async (canvas, size) => {
  const { createIntroScene } = await import('@/features/intro/scene')
  return createIntroScene(canvas, size, () => ({
    elapsedMs: elapsed.value,
    durationMs: duration.value,
    supporterLockMs: SUPPORTER_LOCK_MS,
    loading: !loaded.value,
    cards: cards.value,
  }))
}

function close() {
  if (closed) return
  closed = true
  cancelAnimationFrame(timer)
  emit('close')
}
function tick(now: number) {
  if (!startedAt) startedAt = now
  elapsed.value = now - startedAt
  if (!locked.value && elapsed.value >= SUPPORTER_LOCK_MS) {
    lockedCards.value = structuredClone(candidateCards.value)
    locked.value = true
  }
  if (elapsed.value >= duration.value) close()
  else timer = requestAnimationFrame(tick)
}
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close()
  if (event.key === 'Tab') { event.preventDefault(); skipButton.value?.focus() }
}

onMounted(async () => {
  window.addEventListener('keydown', onKeydown)
  await nextTick(); skipButton.value?.focus()
  timer = requestAnimationFrame(tick)
  void loadIntroData().then((value) => {
    if (!closed && !locked.value) data.value = value
  }).finally(() => { if (!closed) loaded.value = true })
})
onBeforeUnmount(() => { cancelAnimationFrame(timer); window.removeEventListener('keydown', onKeydown) })
</script>

<template>
  <section class="cinema-overlay intro-overlay" role="dialog" aria-modal="true" aria-label="贡献陈列馆启动鸣谢">
    <ThreeStage v-if="!failed && !reducedMotion" :factory="sceneFactory" @error="failed = true" @context-lost="failed = true" />
    <div v-else class="cinema-static" aria-hidden="true" />
    <header class="cinema-topbar">
      <div><span>CS2AS</span><strong>{{ appConfig.appVersion }}</strong></div>
      <button ref="skipButton" type="button" class="cinema-text-button" @click="close">跳过</button>
    </header>
    <div class="intro-copy">
      <p class="cinema-kicker">贡献陈列馆</p><h1>幸与诸君同路</h1>
      <Transition name="cinema-copy" mode="out-in">
        <div v-if="currentCard" :key="currentCard.id" class="supporter-spotlight">
          <small>{{ currentCard.eyebrow }}</small>
          <strong>{{ currentCard.title }}</strong>
          <span>{{ currentCard.message }}</span>
          <small>{{ currentCard.detail }}</small>
        </div>
        <p v-else class="cinema-note">{{ supporterStatus }}<br>感谢每一位同路人与上游作者</p>
      </Transition>
      <small v-if="data.supporters.length > 8">以及 {{ data.supporters.length - 8 }} 位同路人</small>
    </div>
    <div class="cinema-progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
    <div class="sr-only" aria-live="polite">{{ currentCard ? `${currentCard.eyebrow} ${currentCard.title}` : '正在展示鸣谢长廊' }}</div>
  </section>
</template>
