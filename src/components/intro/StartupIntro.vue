<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import ThreeStage from '@/components/three/ThreeStage.vue'
import type { ThreeStageFactory } from '@/components/three/stage'
import { appConfig } from '@/config/app'
import type { IntroData } from '@/features/intro/types'
import { FALLBACK_UPSTREAM, loadIntroData } from '@/services/intro-data'

const emit = defineEmits<{ close: [] }>()
const skipButton = ref<HTMLButtonElement | null>(null)
const elapsed = ref(0)
const failed = ref(false)
const loaded = ref(false)
const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
const data = ref<IntroData>({ supporters: [], upstream: FALLBACK_UPSTREAM, fetchedAt: null, sources: { supporters: 'fallback', upstream: 'fallback' } })
let timer = 0
let startedAt = 0
let closed = false

const supporters = computed(() => data.value.supporters.slice(0, 8))
const duration = computed(() => reducedMotion ? 1_200 : Math.min(9_500, Math.max(8_000, supporters.value.length * 2_000 + 1_500)))
const supporterDuration = computed(() => duration.value - (reducedMotion ? 560 : 1_500))
const supporterSlot = computed(() => supporters.value.length ? supporterDuration.value / supporters.value.length : supporterDuration.value)
const phase = computed<'supporters' | 'upstream'>(() => elapsed.value < supporterDuration.value ? 'supporters' : 'upstream')
const currentSupporter = computed(() => supporters.value[Math.min(supporters.value.length - 1, Math.floor(elapsed.value / Math.max(1, supporterSlot.value)))])
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
    supporterSlotMs: supporterSlot.value,
    phase: phase.value,
    loading: !loaded.value,
    supporters: supporters.value,
    upstream: data.value.upstream,
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
  void loadIntroData().then((value) => { if (!closed) data.value = value }).finally(() => { if (!closed) loaded.value = true })
})
onBeforeUnmount(() => { cancelAnimationFrame(timer); window.removeEventListener('keydown', onKeydown) })
</script>

<template>
  <section class="cinema-overlay intro-overlay" role="dialog" aria-modal="true" aria-label="启动鸣谢">
    <ThreeStage v-if="!failed && !reducedMotion" :factory="sceneFactory" @error="failed = true" @context-lost="failed = true" />
    <div v-else class="cinema-static" aria-hidden="true" />
    <header class="cinema-topbar">
      <div><span>CS2AS</span><strong>{{ appConfig.appVersion }}</strong></div>
      <button ref="skipButton" type="button" class="cinema-text-button" @click="close">跳过</button>
    </header>
    <div class="intro-copy" :data-phase="phase">
      <Transition name="cinema-copy" mode="out-in">
        <div v-if="phase === 'supporters'" key="supporters">
          <p class="cinema-kicker">鸣谢长廊</p><h1>幸与诸君同路</h1>
          <div v-if="currentSupporter" class="supporter-spotlight">
            <strong>{{ currentSupporter.nickname || '青锋无名客' }}</strong>
            <span>{{ currentSupporter.message || '长夜执剑，幸与诸君同路。' }}</span>
            <small>¥{{ (currentSupporter.amountCents / 100).toFixed(2) }}</small>
          </div>
          <p v-else class="cinema-note">{{ supporterStatus }}<br>感谢每一位同路人</p>
          <small v-if="data.supporters.length > 8">以及 {{ data.supporters.length - 8 }} 位同路人</small>
        </div>
        <div v-else key="upstream" class="upstream-monument">
          <p class="cinema-kicker">源流所自</p><h1>{{ data.upstream.fullName }}</h1>
          <p>{{ data.upstream.description }}</p>
          <dl><div><dt>LICENSE</dt><dd>{{ data.upstream.license }}</dd></div><div><dt>STARS</dt><dd>{{ data.upstream.stars ?? '--' }}</dd></div><div><dt>FORKS</dt><dd>{{ data.upstream.forks ?? '--' }}</dd></div></dl>
        </div>
      </Transition>
    </div>
    <div class="cinema-progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
    <div class="sr-only" aria-live="polite">{{ phase === 'supporters' ? '正在展示鸣谢长廊' : `上游项目 ${data.upstream.fullName}` }}</div>
  </section>
</template>
