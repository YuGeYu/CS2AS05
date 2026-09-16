<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ExternalLink, FolderOpen, RefreshCw, Search } from 'lucide-vue-next'
import { loadVaultResources } from '@/services/resource-vault'
import { openResourceLink } from '@/services/tauri/support'
import type { VaultResource } from '@/features/resource-vault/types'
const resources = ref<VaultResource[]>([]); const loading = ref(true); const error = ref(''); const query = ref(''); const selected = ref<VaultResource | null>(null)
const filtered = computed(() => { const q = query.value.trim().toLowerCase(); return q ? resources.value.filter(item => `${item.title} ${item.summary} ${item.category} ${item.tags.join(' ')}`.toLowerCase().includes(q)) : resources.value })
async function refresh(force = false) { loading.value = true; error.value = ''; try { resources.value = await loadVaultResources(force) } catch (cause) { error.value = cause instanceof Error ? cause.message : '资源网站暂时不可用。' } finally { loading.value = false } }
async function openLink(url: string) { if (!url) return; try { await openResourceLink(url) } catch { window.open(url, '_blank', 'noopener,noreferrer') } }
onMounted(() => void refresh())
</script>
<template>
  <section class="resource-vault-view">
    <header class="page-heading"><div><p class="overline">听澜云笈阁 · 助手专享</p><h1>资源阁</h1><p>不论是否登录，CS2AS05 用户都可以在助手内查看完整资源信息。</p></div><button class="secondary-button" type="button" :disabled="loading" @click="refresh(true)"><RefreshCw :size="16" />刷新</button></header>
    <div class="resource-vault-toolbar"><label class="resource-vault-search"><Search :size="17" /><input v-model="query" type="search" placeholder="搜索资源标题、分类或标签" aria-label="搜索资源" /></label><span>{{ filtered.length }} 项资源</span></div>
    <p v-if="error" class="inline-error" role="alert">{{ error }} <button class="text-button" type="button" @click="refresh(true)">重试</button></p>
    <div v-else-if="loading" class="empty-state"><RefreshCw class="spin" :size="22" />正在从资源阁整理目录…</div>
    <div v-else-if="!filtered.length" class="empty-state"><FolderOpen :size="22" />暂时没有匹配的资源。</div>
    <div v-else class="resource-vault-grid"><article v-for="item in filtered" :key="item.id" class="resource-vault-card"><div class="resource-vault-card__meta"><span>{{ item.category }}</span><b v-if="item.isFeatured">精选</b></div><h2>{{ item.title }}</h2><p>{{ item.summary || '打开详情查看完整说明。' }}</p><div class="resource-vault-card__tags"><span v-for="tag in item.tags.slice(0, 4)" :key="tag">#{{ tag }}</span></div><footer><button class="primary-button" type="button" @click="selected = item">查看详情</button><span v-if="item.price > 0" class="resource-vault-free">助手内免费查看</span><span v-else class="resource-vault-free">免费资源</span></footer></article></div>
    <div v-if="selected" class="resource-vault-modal-layer" role="presentation" @click.self="selected = null"><article class="resource-vault-modal" role="dialog" aria-modal="true" aria-labelledby="resource-detail-title"><button class="icon-button" type="button" aria-label="关闭详情" @click="selected = null">×</button><p class="overline">{{ selected.category }} · {{ selected.panType }}</p><h2 id="resource-detail-title">{{ selected.title }}</h2><p>{{ selected.summary }}</p><div class="resource-detail-content" v-text="selected.content" /><dl><div><dt>提供方式</dt><dd>{{ selected.obtainNote || '打开外部链接获取' }}</dd></div><div v-if="selected.obtainCode"><dt>提取码</dt><dd>{{ selected.obtainCode }}</dd></div></dl><button v-if="selected.obtainUrl" class="primary-button" type="button" @click="openLink(selected.obtainUrl)"><ExternalLink :size="16" />在默认浏览器打开获取链接</button><p class="resource-vault-notice">精品资源的流光灵玉门槛仅在资源网站生效；助手内仅展示信息，不扣除任何灵玉。</p></article></div>
  </section>
</template>
