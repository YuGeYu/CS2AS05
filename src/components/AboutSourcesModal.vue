<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { ArrowLeft, ExternalLink, X } from 'lucide-vue-next'

import { appConfig } from '@/config/app'
import { REFERENCE_PROJECT_GROUPS } from '@/features/support/reference-projects'
import { openReferenceProject, type ReferenceProjectId } from '@/services/tauri/support'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const view = ref<'about' | 'sources'>('about')
const dialog = ref<HTMLElement | null>(null)
const errorMessage = ref('')

watch(() => props.open, async (open) => {
  if (!open) return
  view.value = 'about'
  errorMessage.value = ''
  await nextTick()
  dialog.value?.focus()
})

function close() {
  view.value = 'about'
  emit('close')
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && props.open) close()
}

const sourceGroups = REFERENCE_PROJECT_GROUPS

async function openProject(project: ReferenceProjectId) {
  errorMessage.value = ''
  try {
    await openReferenceProject(project)
  } catch {
    errorMessage.value = '暂时无法打开上游项目。'
  }
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation">
      <section ref="dialog" class="about-dialog" role="dialog" aria-modal="true" aria-labelledby="about-title" tabindex="-1">
        <header class="modal-heading about-nav">
          <button v-if="view === 'sources'" class="icon-button" type="button" aria-label="返回关于页面" @click="view = 'about'">
            <ArrowLeft :size="18" />
          </button>
          <div>
            <p class="overline">关于与来源</p>
            <h2 id="about-title">{{ view === 'about' ? '关于 CS2 人机增强助手' : '参考项目与许可' }}</h2>
          </div>
          <button class="icon-button modal-close" type="button" aria-label="关闭关于与来源" @click="close">
            <X :size="18" />
          </button>
        </header>

        <div v-if="view === 'about'" class="about-content">
          <p>当前版本 <strong>{{ appConfig.appVersion }}</strong></p>
          <p>用于安装和管理 CS2 BOT 增强定制资源包，并提供官方 Panel 启动入口。</p>
          <p>桌面助手与对应修改源码以 AGPL-3.0-or-later 发布。</p>
          <button class="secondary-button source-link" type="button" @click="view = 'sources'">参考项目与许可</button>
        </div>

        <div v-else class="about-content">
          <div class="source-groups">
            <section v-for="group in sourceGroups" :key="group.title" class="source-group">
              <h3>{{ group.title }}</h3>
              <article v-for="project in group.projects" :key="project.id" class="source-project">
                <div><strong>{{ project.repository }}</strong><p>{{ project.description }}</p><small v-if="project.license">许可：{{ project.license }}</small></div>
                <button class="secondary-button" type="button" :aria-label="`打开项目 ${project.repository}`" @click="openProject(project.id)"><ExternalLink :size="17" /><span>打开项目</span></button>
              </article>
            </section>
          </div>
          <p>许可与固定提交以项目根目录 NOTICE.md 及 third_party 来源记录为准。</p>
          <p v-if="errorMessage" class="inline-error" role="alert">{{ errorMessage }}</p>
        </div>
      </section>
    </div>
  </Teleport>
</template>
