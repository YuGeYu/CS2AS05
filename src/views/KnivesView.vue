<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'
import { Check, Keyboard } from 'lucide-vue-next'
import { KNIVES } from '@/data/panel/knives'
import { captureKeyName } from '@/features/panel/key-capture'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

const cs2 = useCs2Store(); const panel = usePanelStore(); const capturing = ref(false)
function stopCapture() { capturing.value = false; window.removeEventListener('keydown', onKey, true) }
function startCapture() { capturing.value = true; window.addEventListener('keydown', onKey, true) }
function onKey(event: KeyboardEvent) { event.preventDefault(); event.stopPropagation(); if (event.code === 'Escape') return stopCapture(); const key = captureKeyName(event); if (!key) return; stopCapture(); void save(key, panel.snapshot?.dropKnives.selected ?? []) }
function toggle(id: number) { const selected = new Set(panel.snapshot?.dropKnives.selected ?? []); if (selected.has(id)) selected.delete(id); else selected.add(id); void save(panel.snapshot?.dropKnives.bindKey || '\\', [...selected]) }
function save(bindKey: string, selected: number[]) { return panel.setDropKnives(cs2.selectedRoot, bindKey, selected).catch(() => undefined) }
function hideBrokenImage(event: Event) { (event.currentTarget as HTMLImageElement).hidden = true }
onBeforeUnmount(stopCapture)
</script>

<template><section class="tool-view" aria-labelledby="knives-title"><header class="view-heading"><div><p class="overline">subclass_create</p><h1 id="knives-title">丢刀</h1></div><button class="secondary-button" :class="{ 'is-capturing': capturing }" :disabled="!panel.snapshot?.ready" @click="capturing ? stopCapture() : startCapture()"><Keyboard :size="18" />{{ capturing ? '按下按键，Esc 取消' : `绑定：${panel.snapshot?.dropKnives.bindKey || '\\'}` }}</button></header><section class="knife-grid" aria-label="刀具多选"><button v-for="knife in KNIVES" :key="knife.id" type="button" :aria-pressed="panel.snapshot?.dropKnives.selected.includes(knife.id)" :disabled="!panel.snapshot?.ready || !!panel.mutationKey" @click="toggle(knife.id)"><span class="knife-image-frame"><img class="knife-image" :src="knife.image" :alt="knife.name" draggable="false" @error="hideBrokenImage"></span><span class="knife-label"><strong>{{ knife.name }}</strong><small>subclass {{ knife.id }}</small></span><Check v-if="panel.snapshot?.dropKnives.selected.includes(knife.id)" :size="17" aria-hidden="true" /></button></section><div class="selection-actions"><span>已选择 {{ panel.snapshot?.dropKnives.selected.length ?? 0 }} / {{ KNIVES.length }}</span><button class="text-button" :disabled="!panel.snapshot?.ready" @click="save(panel.snapshot?.dropKnives.bindKey || '\\', KNIVES.map(item => item.id))">全选</button><button class="text-button" :disabled="!panel.snapshot?.ready" @click="save(panel.snapshot?.dropKnives.bindKey || '\\', [])">清空</button></div></section></template>
