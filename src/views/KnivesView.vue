<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { Check, Keyboard } from 'lucide-vue-next'
import { KNIVES } from '@/data/panel/knives'
import { captureKeyName } from '@/features/panel/key-capture'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store(); const panel = usePanelStore(); const capturing = ref(false)
const recentlySelectedId = ref<number | null>(null); const selectionNotice = ref(''); const selectionTone = ref<'selected' | 'cleared' | null>(null)
let selectedTimer: ReturnType<typeof setTimeout> | undefined; let noticeTimer: ReturnType<typeof setTimeout> | undefined
function stopCapture() { capturing.value = false; window.removeEventListener('keydown', onKey, true) }
function startCapture() { capturing.value = true; window.addEventListener('keydown', onKey, true) }
function onKey(event: KeyboardEvent) { event.preventDefault(); event.stopPropagation(); if (event.code === 'Escape') return stopCapture(); const key = captureKeyName(event); if (!key) return; stopCapture(); void save(key, panel.snapshot?.dropKnives.selected ?? []) }
async function toggle(id: number) {
  const selected = new Set(panel.snapshot?.dropKnives.selected ?? []); const adding = !selected.has(id)
  if (adding) selected.add(id); else selected.delete(id)
  try {
    await panel.setDropKnives(cs2.selectedRoot, panel.snapshot?.dropKnives.bindKey || '\\', [...selected])
    if (!adding) return
    if (selectedTimer) clearTimeout(selectedTimer)
    recentlySelectedId.value = id
    selectedTimer = setTimeout(() => { recentlySelectedId.value = null; selectedTimer = undefined }, 450)
  } catch { /* Store retains the visible error and authoritative selection. */ }
}
function save(bindKey: string, selected: number[]) { return panel.setDropKnives(cs2.selectedRoot, bindKey, selected).catch(() => undefined) }
async function applySelection(selected: number[], tone: 'selected' | 'cleared') {
  try {
    await panel.setDropKnives(cs2.selectedRoot, panel.snapshot?.dropKnives.bindKey || '\\', selected)
    if (noticeTimer) clearTimeout(noticeTimer)
    selectionTone.value = tone
    selectionNotice.value = tone === 'selected' ? `已选择全部 ${KNIVES.length} 款刀具` : '已清空刀具选择'
    noticeTimer = setTimeout(() => { selectionNotice.value = ''; selectionTone.value = null; noticeTimer = undefined }, 1100)
  } catch { /* Store retains the visible error and authoritative selection. */ }
}
function hideBrokenImage(event: Event) { (event.currentTarget as HTMLImageElement).hidden = true }
onBeforeUnmount(() => { stopCapture(); if (selectedTimer) clearTimeout(selectedTimer); if (noticeTimer) clearTimeout(noticeTimer) })
</script>

<template><section class="tool-view" aria-labelledby="knives-title"><header class="view-heading"><div><p class="overline">subclass_create</p><h1 id="knives-title">丢刀</h1></div><button class="secondary-button" :class="{ 'is-capturing': capturing }" :disabled="!panel.snapshot?.ready" @click="capturing ? stopCapture() : startCapture()"><Keyboard :size="18" />{{ capturing ? '按下按键，Esc 取消' : `绑定：${panel.snapshot?.dropKnives.bindKey || '\\'}` }}</button></header><section class="knife-grid" aria-label="刀具多选"><button v-for="knife in KNIVES" :key="knife.id" type="button" :class="{ 'is-just-selected': recentlySelectedId === knife.id }" :aria-pressed="panel.snapshot?.dropKnives.selected.includes(knife.id)" :disabled="!panel.snapshot?.ready || !!panel.mutationKey" @click="toggle(knife.id)"><span class="knife-image-frame"><img class="knife-image" :src="knife.image" :alt="knife.name" draggable="false" @error="hideBrokenImage"></span><span class="knife-label"><strong>{{ knife.name }}</strong><small>subclass {{ knife.id }}</small></span><Check v-if="panel.snapshot?.dropKnives.selected.includes(knife.id)" :size="17" aria-hidden="true" /></button></section><div class="selection-actions" :class="{ 'is-confirmed': selectionTone === 'selected', 'is-cleared': selectionTone === 'cleared' }"><span>已选择 {{ panel.snapshot?.dropKnives.selected.length ?? 0 }} / {{ KNIVES.length }}</span><span class="selection-notice" aria-live="polite">{{ selectionNotice }}</span><button class="text-button" :disabled="!panel.snapshot?.ready || !!panel.mutationKey" @click="applySelection(KNIVES.map(item => item.id), 'selected')">全选</button><button class="text-button" :disabled="!panel.snapshot?.ready || !!panel.mutationKey" @click="applySelection([], 'cleared')">清空</button></div><p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p></section></template>
