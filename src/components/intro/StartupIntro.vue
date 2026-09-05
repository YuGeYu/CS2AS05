<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { appConfig } from '@/config/app'
import { STATIC_INTRO_DATA, STATIC_REFERENCE_PROJECTS } from '@/features/intro/static-data'
const DURATION_MS = 6800
const emit = defineEmits<{ close: [] }>()
const skipButton = ref<HTMLButtonElement | null>(null)
const elapsed = ref(0)
let frame = 0; let started = 0; let closed = false
const projects = STATIC_REFERENCE_PROJECTS
const supporters = STATIC_INTRO_DATA.supporters
const focusRows = [...projects.map(item => item.repository), ...supporters.map(item => item.nickname || '匿名同路人')]
const progress = computed(() => Math.min(100, elapsed.value / DURATION_MS * 100))
const activeProject = computed(() => focusRows[Math.min(focusRows.length - 1, Math.floor(Math.max(0, elapsed.value - 600) / Math.max(1, (DURATION_MS - 600) / focusRows.length)))])
function close() { if (closed) return; closed = true; cancelAnimationFrame(frame); emit('close') }
function tick(now: number) { if (!started) started = now; elapsed.value = now - started; if (elapsed.value >= DURATION_MS) close(); else frame = requestAnimationFrame(tick) }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape' || event.key === 'Tab') { event.preventDefault(); close() } }
onMounted(() => { window.addEventListener('keydown', onKeydown); skipButton.value?.focus(); frame = requestAnimationFrame(tick) })
onBeforeUnmount(() => { cancelAnimationFrame(frame); window.removeEventListener('keydown', onKeydown) })
</script>
<template>
  <section class="cinema-overlay intro-overlay static-ack-overlay" role="dialog" aria-modal="true" aria-label="版本启动鸣谢">
    <div class="static-scanline" aria-hidden="true" /><header class="cinema-topbar"><div><span>CS2AS</span><strong>{{ appConfig.appVersion }}</strong></div><button ref="skipButton" type="button" class="cinema-text-button" @click="close">跳过</button></header>
    <div class="static-ack-shell"><div class="static-ack-heading"><p class="cinema-kicker">版本启动鸣谢 · 固化名单</p><h1>幸与诸君同路</h1><p>上游项目与公开鸣谢随本版本一同封存，启动时不访问官网赞助接口。</p></div><div class="static-ack-table-wrap"><table class="static-ack-table"><caption class="sr-only">上游项目与公开鸣谢名单</caption><thead><tr><th>类别</th><th>项目 / 名称</th><th>说明</th><th>许可 / 鸣谢</th></tr></thead><tbody><tr v-for="project in projects" :key="project.id"><td><span class="ack-badge">上游</span></td><td><strong>{{ project.repository }}</strong></td><td>{{ project.description }}</td><td>{{ project.license || project.group }}</td></tr><tr v-for="supporter in supporters" :key="supporter.id"><td><span class="ack-badge ack-badge--supporter">鸣谢</span></td><td><strong>{{ supporter.nickname || '匿名同路人' }}</strong></td><td>{{ supporter.message || '感谢你的支持与同行。' }}</td><td>公开鸣谢</td></tr></tbody></table></div><p class="static-ack-focus" aria-live="polite"><span>当前焦点</span>{{ activeProject }}</p></div>
    <div class="cinema-progress" aria-hidden="true"><span :style="{ width: `${progress}%` }" /></div>
  </section>
</template>
