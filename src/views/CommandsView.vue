<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref } from 'vue'
import { Check, ChevronDown, ChevronUp, Copy, Search } from 'lucide-vue-next'
import { parseCommands } from '@/data/panel/commands'

const entries = parseCommands(); const query = ref(''); const activeIndex = ref(0); const commandList = ref<HTMLElement>()
const copiedId = ref<number | null>(null); const copyStatus = ref(''); const copyTone = ref<'success' | 'error' | null>(null)
let feedbackTimer: ReturnType<typeof setTimeout> | undefined; let copyRequest = 0
const matches = computed(() => { const q = query.value.trim().toLowerCase(); return q ? entries.filter(entry => entry.copyable && entry.display.toLowerCase().includes(q)) : [] })
function move(delta: number) { if (!matches.value.length) return; activeIndex.value = (activeIndex.value + delta + matches.value.length) % matches.value.length; void nextTick(() => { const list = commandList.value; const element = list?.querySelector<HTMLElement>(`[data-command-id="${matches.value[activeIndex.value]?.id}"]`); if (!list || !element) return; const listBox = list.getBoundingClientRect(); const elementBox = element.getBoundingClientRect(); list.scrollTop += elementBox.top - listBox.top - (list.clientHeight - element.clientHeight) / 2 }) }
function isMatch(id: number) { return matches.value.some(item => item.id === id) }
function isActive(id: number) { return matches.value[activeIndex.value]?.id === id }
function scheduleFeedbackClear() {
  if (feedbackTimer) clearTimeout(feedbackTimer)
  feedbackTimer = setTimeout(() => { copiedId.value = null; copyStatus.value = ''; copyTone.value = null; feedbackTimer = undefined }, 1200)
}
async function copy(id: number, value: string) {
  const request = ++copyRequest
  if (feedbackTimer) { clearTimeout(feedbackTimer); feedbackTimer = undefined }
  copiedId.value = null; copyStatus.value = ''; copyTone.value = null
  try {
    const { writeText } = await import('@tauri-apps/plugin-clipboard-manager')
    await writeText(value)
    if (request !== copyRequest) return
    copiedId.value = id; copyTone.value = 'success'; copyStatus.value = value.endsWith(' ') ? '命令已复制，并保留尾随空格。' : '命令已复制。'
  } catch {
    if (request !== copyRequest) return
    copiedId.value = null; copyTone.value = 'error'; copyStatus.value = '复制失败，请重试。'
  }
  scheduleFeedbackClear()
}
onBeforeUnmount(() => { copyRequest += 1; if (feedbackTimer) clearTimeout(feedbackTimer) })
</script>

<template><section class="tool-view commands-view" aria-labelledby="commands-title"><header class="view-heading"><div><p class="overline">CS2 控制台</p><h1 id="commands-title">命令</h1></div><span>{{ entries.filter(item => item.copyable).length }} 条可复制内容</span></header><div class="command-toolbar"><label><Search :size="17" /><input v-model="query" placeholder="搜索命令、队伍或参数" @input="activeIndex = 0" /></label><span>{{ matches.length ? activeIndex + 1 : 0 }} / {{ matches.length }}</span><button class="icon-button" title="上一个匹配" aria-label="上一个匹配" :disabled="!matches.length" @click="move(-1)"><ChevronUp :size="18" /></button><button class="icon-button" title="下一个匹配" aria-label="下一个匹配" :disabled="!matches.length" @click="move(1)"><ChevronDown :size="18" /></button></div><div ref="commandList" class="command-list"><template v-for="entry in entries" :key="entry.id"><h2 v-if="!entry.copyable">{{ entry.display }}</h2><button v-else type="button" :data-command-id="entry.id" :data-copied="copiedId === entry.id ? 'true' : undefined" :class="{ 'is-match': isMatch(entry.id), 'is-active': isActive(entry.id) }" @click="copy(entry.id, entry.copy)"><code>{{ entry.display }}</code><Check v-if="copiedId === entry.id" :size="16" aria-hidden="true" /><Copy v-else :size="16" aria-hidden="true" /></button></template></div><p class="command-result" :data-tone="copyTone" :role="copyTone === 'error' ? 'alert' : undefined" aria-live="polite">{{ copyStatus }}</p></section></template>
