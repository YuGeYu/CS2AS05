<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import { ExternalLink, X } from 'lucide-vue-next'
import { appConfig } from '@/config/app'
import ThreeStage from '@/components/three/ThreeStage.vue'
import { createAcknowledgementScene } from '@/features/acknowledgement/scene'
import { STATIC_REFERENCE_PROJECTS } from '@/features/intro/static-data'
import type { IntroAcknowledgementCard } from '@/features/intro/types'
import { openReferenceProject, type ReferenceProjectId } from '@/services/tauri/support'
const emit = defineEmits<{ close: [] }>()
const selectedIndex = ref(0); const cards = ref<IntroAcknowledgementCard[]>([]); const errorMessage = ref(''); const closeButton = ref<HTMLButtonElement | null>(null)
const selected = computed(() => cards.value[selectedIndex.value])
const fallbackCards = () => STATIC_REFERENCE_PROJECTS.map(item => ({ id: `upstream:${item.id}`, kind: 'upstream' as const, eyebrow: '上游项目', title: item.repository, message: item.description, detail: item.license || item.group, updatedAt: '' }))
function close() { emit('close') }
function select(index: number) { selectedIndex.value = Math.max(0, Math.min(Math.max(0, cards.value.length - 1), index)) }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape') close(); if (event.key === 'ArrowRight' || event.key === 'ArrowDown') { event.preventDefault(); select(selectedIndex.value + 1) }; if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') { event.preventDefault(); select(selectedIndex.value - 1) } }
async function openProject(id: ReferenceProjectId) { errorMessage.value = ''; try { await openReferenceProject(id) } catch { errorMessage.value = '暂时无法打开上游项目。' } }
onMounted(() => { window.addEventListener('keydown', onKeydown); closeButton.value?.focus(); cards.value = fallbackCards() })
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>
<template>
  <section class="cinema-overlay intro-overlay ink-ack-scene easter-ink-scene" role="dialog" aria-modal="true" aria-label="水墨江南鸣谢长廊">
    <header class="cinema-topbar"><div><span>CS2AS · 水墨江南</span><strong>{{ appConfig.appVersion }}</strong></div><button ref="closeButton" type="button" class="cinema-text-button" @click="close">返回助手</button></header>
    <ThreeStage :factory="(canvas, size) => createAcknowledgementScene(canvas, size, () => ({ elapsedMs: 0, durationMs: 1, supporterLockMs: 0, loading: false, cards, mode: 'free-roam', selectedIndex }))" />
    <aside class="ink-ack-copy easter-copy"><p class="cinema-kicker">同一条水墨长廊 · 自由游览</p><h1>上游项目鸣谢</h1><p aria-live="polite">当前牌子：{{ selected?.title || '正在载入鸣谢' }}</p><small>因部分赞助者不愿公开昵称，我们不再展示赞助者或 B 站充电记录。这里只保留构建时固定的上游项目资料。</small></aside>
    <nav class="easter-plaque-nav" aria-label="选择牌子"><button v-for="(card, index) in cards" :key="card.id" type="button" :class="{ active: index === selectedIndex }" @click="select(index)"><span>{{ card.eyebrow }}</span><strong>{{ card.title }}</strong><ExternalLink v-if="card.kind === 'upstream'" :size="14" @click.stop="openProject(card.id.replace('upstream:', '') as ReferenceProjectId)" /></button></nav>
    <p v-if="errorMessage" class="inline-error" role="alert">{{ errorMessage }}</p><span class="sr-only">贡献陈列馆</span><button class="cinema-icon-button easter-close" type="button" title="关闭" aria-label="关闭贡献陈列馆" @click="close"><X :size="20" /></button>
  </section>
</template>
