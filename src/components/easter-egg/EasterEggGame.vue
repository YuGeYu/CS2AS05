<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { ExternalLink, X } from 'lucide-vue-next'
import { appConfig } from '@/config/app'
import { STATIC_REFERENCE_PROJECTS, STATIC_SUPPORTERS } from '@/features/intro/static-data'
import { openReferenceProject, type ReferenceProjectId } from '@/services/tauri/support'
const emit = defineEmits<{ close: [] }>()
const closeButton = ref<HTMLButtonElement | null>(null); const errorMessage = ref('')
const projects = STATIC_REFERENCE_PROJECTS; const supporters = STATIC_SUPPORTERS
function close() { emit('close') }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape') close(); if (event.key === 'Tab') { event.preventDefault(); closeButton.value?.focus() } }
async function openProject(id: ReferenceProjectId) { errorMessage.value = ''; try { await openReferenceProject(id) } catch { errorMessage.value = '暂时无法打开上游项目。' } }
onMounted(async () => { window.addEventListener('keydown', onKeydown); await nextTick(); closeButton.value?.focus() })
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>
<template>
  <section class="cinema-overlay gallery-overlay static-gallery-overlay" role="dialog" aria-modal="true" aria-label="贡献陈列馆">
    <div class="static-grid" aria-hidden="true" /><header class="gallery-static-header"><div><p class="cinema-kicker">CS2AS · {{ appConfig.appVersion }}</p><h1>众行者，共铸此间</h1><span>贡献陈列馆 · 固化鸣谢档案</span></div><button ref="closeButton" class="cinema-icon-button" type="button" title="关闭" aria-label="关闭贡献陈列馆" @click="close"><X :size="20" /></button></header>
    <main class="ack-archive"><div class="ack-archive-toolbar"><span>鸣谢档案</span><small>随版本发布 · 固定快照，平等展示</small></div><div class="ack-archive-scroll"><table class="static-ack-table static-ack-table--gallery"><caption class="sr-only">上游项目与公开鸣谢名单</caption><thead><tr><th>类别</th><th>名称</th><th>贡献 / 说明</th><th>动作</th></tr></thead><tbody><tr v-for="project in projects" :key="project.id"><td><span class="ack-badge">上游</span></td><td><strong>{{ project.repository }}</strong></td><td>{{ project.description }}</td><td><button class="table-icon-button" type="button" :aria-label="`打开项目 ${project.repository}`" title="打开项目" @click="openProject(project.id)"><ExternalLink :size="16" /></button></td></tr><tr v-for="supporter in supporters" :key="supporter.id"><td><span class="ack-badge ack-badge--supporter">鸣谢</span></td><td><strong>{{ supporter.nickname || '匿名同路人' }}</strong></td><td>{{ supporter.message || '感谢你的支持与同行。' }}</td><td><span class="ack-static-mark">已收录</span></td></tr></tbody></table></div><p v-if="errorMessage" class="inline-error" role="alert">{{ errorMessage }}</p></main>
    <footer class="gallery-static-footer"><span>鸣谢名单与上游项目按 {{ appConfig.appVersion }} 版本固化</span><span>CS2AS · 长期维护</span></footer>
  </section>
</template>
