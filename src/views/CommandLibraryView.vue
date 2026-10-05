<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { Copy, RefreshCw, Search, TerminalSquare, X } from 'lucide-vue-next'
import { loadCommandLibrary, type CommandLibraryCommand, type CommandLibraryPayload } from '@/services/tauri/support'

const library = ref<CommandLibraryPayload | null>(null)
const loading = ref(true)
const error = ref('')
const query = ref('')
const selectedInitial = ref('')
const copyState = ref('')
const initials = ['+', '-', '_', ...'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '#']
function commandInitial(name: string) { const first = name[0] || '#'; return ['+', '-', '_'].includes(first) ? first : /^[a-z]$/i.test(first) ? first.toUpperCase() : '#' }
const results = computed(() => {
  const needle = query.value.trim().toLocaleLowerCase()
  return (library.value?.commands ?? []).filter((command) => {
    if (needle && ![command.name, command.defaultValue, command.flags, command.description].some((field) => field.toLocaleLowerCase().includes(needle))) return false
    return !selectedInitial.value || commandInitial(command.name) === selectedInitial.value
  })
})
async function load(force = false) { loading.value = true; error.value = ''; try { library.value = await loadCommandLibrary(force) } catch (cause) { error.value = cause instanceof Error ? cause.message : '指令资料加载失败。' } finally { loading.value = false } }
async function copyCommand(command: CommandLibraryCommand) { try { const { writeText } = await import('@tauri-apps/plugin-clipboard-manager'); await writeText(command.name); copyState.value = `已复制 ${command.name}` } catch { copyState.value = '复制失败，请手动选择命令文本。' } }
function clearFilters() { query.value = ''; selectedInitial.value = '' }
onMounted(() => void load())
</script>

<template>
  <section class="command-library-view" aria-labelledby="command-library-title">
    <header class="view-heading command-library-hero"><div><p class="overline"><TerminalSquare :size="14" /> CS2 玩家指令资料</p><h1 id="command-library-title">指令研究所</h1><p class="command-library-lede">与官网指令库同步的独立桌面资料页，搜索命令、默认值、Flags 与说明，点击即可复制。</p></div><div v-if="library" class="command-library-count"><strong>{{ library.commands.length.toLocaleString() }}</strong><span>条资料</span></div></header>
    <div class="command-library-toolbar"><label class="command-library-search"><Search :size="18" /><input v-model="query" type="search" placeholder="搜索命令、默认值、Flags 或说明" aria-label="搜索指令资料" /><button v-if="query" class="icon-button" type="button" title="清空搜索" aria-label="清空搜索" @click="query = ''"><X :size="17" /></button></label><button class="secondary-button" type="button" :disabled="loading" @click="load(true)"><RefreshCw :size="16" />重新加载</button></div>
    <nav class="command-library-initials" aria-label="按首字符筛选"><button v-for="initial in initials" :key="initial" type="button" :class="{ active: selectedInitial === initial }" :aria-pressed="selectedInitial === initial" @click="selectedInitial = selectedInitial === initial ? '' : initial">{{ initial }}</button><strong>{{ results.length.toLocaleString() }} 条结果</strong></nav>
    <p v-if="copyState" class="command-library-status" role="status">{{ copyState }}</p>
    <section v-if="loading" class="empty-state"><RefreshCw class="spin" :size="22" />正在整理官网指令资料…</section><section v-else-if="error" class="empty-state" role="alert"><p>{{ error }}</p><button class="secondary-button" type="button" @click="load(true)">重新加载</button></section><section v-else-if="!results.length" class="empty-state"><p>没有找到匹配的指令。</p><button class="secondary-button" type="button" @click="clearFilters">清除筛选</button></section>
    <template v-else><div class="command-library-table-wrap" tabindex="0" aria-label="CS2 指令表格"><table class="command-library-table"><thead><tr><th>指令</th><th>默认值</th><th>Flags</th><th>说明</th></tr></thead><tbody><tr v-for="command in results" :key="command.name"><td><button class="command-copy-text" type="button" @click="copyCommand(command)"><code>{{ command.name }}</code></button><button class="icon-button command-copy-button" type="button" :aria-label="`复制 ${command.name}`" title="复制命令" @click="copyCommand(command)"><Copy :size="17" /></button></td><td>{{ command.defaultValue || '—' }}</td><td>{{ command.flags || '—' }}</td><td>{{ command.description || '—' }}</td></tr></tbody></table></div><div class="command-library-mobile-list"><article v-for="command in results" :key="command.name" class="command-mobile-row"><header><button class="command-copy-text" type="button" @click="copyCommand(command)"><code>{{ command.name }}</code></button><button class="icon-button" type="button" :aria-label="`复制 ${command.name}`" title="复制命令" @click="copyCommand(command)"><Copy :size="17" /></button></header><dl><div><dt>默认值</dt><dd>{{ command.defaultValue || '—' }}</dd></div><div><dt>Flags</dt><dd>{{ command.flags || '—' }}</dd></div><div><dt>说明</dt><dd>{{ command.description || '—' }}</dd></div></dl></article></div></template>
    <footer v-if="library" class="command-library-source">来源：<a :href="library.source.url" target="_blank" rel="noreferrer">{{ library.source.name }}</a><span v-if="library.source.revisionId">revision {{ library.source.revisionId }}</span><span v-if="library.source.buildNote">{{ library.source.buildNote }}</span><span>导入于 {{ new Date(library.source.importedAt).toLocaleString() }}</span><small>数据与官网指令库使用同一份发布快照，游戏更新后可能与当前版本不同。</small></footer>
  </section>
</template>
