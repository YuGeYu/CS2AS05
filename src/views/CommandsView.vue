<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'
import { ChevronDown, ChevronUp, Copy, Search } from 'lucide-vue-next'
import { parseCommands } from '@/data/panel/commands'

const entries = parseCommands(); const query = ref(''); const activeIndex = ref(0); const copied = ref(''); const commandList = ref<HTMLElement>()
const matches = computed(() => { const q = query.value.trim().toLowerCase(); return q ? entries.filter(entry => entry.copyable && entry.display.toLowerCase().includes(q)) : [] })
function move(delta: number) { if (!matches.value.length) return; activeIndex.value = (activeIndex.value + delta + matches.value.length) % matches.value.length; void nextTick(() => { const list = commandList.value; const element = list?.querySelector<HTMLElement>(`[data-command-id="${matches.value[activeIndex.value]?.id}"]`); if (!list || !element) return; const listBox = list.getBoundingClientRect(); const elementBox = element.getBoundingClientRect(); list.scrollTop += elementBox.top - listBox.top - (list.clientHeight - element.clientHeight) / 2 }) }
function isMatch(id: number) { return matches.value.some(item => item.id === id) }
function isActive(id: number) { return matches.value[activeIndex.value]?.id === id }
async function copy(value: string) { const { writeText } = await import('@tauri-apps/plugin-clipboard-manager'); await writeText(value); copied.value = value.endsWith(' ') ? '命令已复制，并保留尾随空格。' : '命令已复制。' }
</script>

<template><section class="tool-view commands-view" aria-labelledby="commands-title"><header class="view-heading"><div><p class="overline">CS2 控制台</p><h1 id="commands-title">命令</h1></div><span>{{ entries.filter(item => item.copyable).length }} 条可复制内容</span></header><div class="command-toolbar"><label><Search :size="17" /><input v-model="query" placeholder="搜索命令、队伍或参数" @input="activeIndex = 0" /></label><span>{{ matches.length ? activeIndex + 1 : 0 }} / {{ matches.length }}</span><button class="icon-button" title="上一个匹配" aria-label="上一个匹配" :disabled="!matches.length" @click="move(-1)"><ChevronUp :size="18" /></button><button class="icon-button" title="下一个匹配" aria-label="下一个匹配" :disabled="!matches.length" @click="move(1)"><ChevronDown :size="18" /></button></div><div ref="commandList" class="command-list"><template v-for="entry in entries" :key="entry.id"><h2 v-if="!entry.copyable">{{ entry.display }}</h2><button v-else type="button" :data-command-id="entry.id" :class="{ 'is-match': isMatch(entry.id), 'is-active': isActive(entry.id) }" @click="copy(entry.copy)"><code>{{ entry.display }}</code><Copy :size="16" /></button></template></div><p class="command-result" aria-live="polite">{{ copied }}</p></section></template>
