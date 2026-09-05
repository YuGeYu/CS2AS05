<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted } from 'vue'
import { AlertTriangle, CheckCircle2, CircleX, Megaphone, RefreshCw, X } from 'lucide-vue-next'
import { announcementSeverityLabel } from '@/features/announcements/service'
import { announcementState, closeAnnouncementCenter, loadAnnouncements } from '@/features/announcements/state'

const severityIcon = { success: CheckCircle2, warning: AlertTriangle, error: CircleX }

function close() {
  closeAnnouncementCenter()
  void nextTick(() => document.querySelector<HTMLButtonElement>('[data-announcement-trigger]')?.focus())
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') close()
}

onMounted(() => document.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <div class="modal-backdrop announcement-backdrop" @click.self="close">
    <section class="announcement-dialog" role="dialog" aria-modal="true" aria-labelledby="announcement-title" aria-describedby="announcement-description">
      <header class="announcement-dialog-header">
        <div class="announcement-dialog-heading">
          <span class="announcement-dialog-icon"><Megaphone :size="21" /></span>
          <div><p class="overline">官网同步</p><h2 id="announcement-title">公告中心</h2></div>
        </div>
        <button class="icon-button" type="button" title="关闭公告" aria-label="关闭公告" @click="close"><X :size="19" /></button>
      </header>
      <p id="announcement-description" class="announcement-dialog-description">这里同步官网意见页面的全部已发布公告，最新内容排在最前。</p>

      <div v-if="announcementState.loading && !announcementState.notices.length" class="announcement-loading" role="status">
        <RefreshCw :size="18" class="spin" />正在同步官网公告…
      </div>
      <div v-else-if="announcementState.error && !announcementState.notices.length" class="announcement-empty is-error" role="alert">
        <CircleX :size="22" /><strong>暂时没有加载成功</strong><span>{{ announcementState.error }}</span>
        <button class="secondary-button" type="button" @click="loadAnnouncements"><RefreshCw :size="16" />重新加载</button>
      </div>
      <div v-else-if="!announcementState.notices.length" class="announcement-empty">
        <Megaphone :size="22" /><strong>目前没有公告</strong><span>新公告发布后会在这里同步显示。</span>
      </div>
      <div v-else class="announcement-list" aria-live="polite">
        <article v-for="notice in announcementState.notices" :key="notice.id" class="announcement-item" :data-severity="notice.severity">
          <div class="announcement-item-meta">
            <span class="announcement-item-severity"><component :is="severityIcon[notice.severity]" :size="16" />{{ announcementSeverityLabel(notice.severity) }}</span>
            <time :datetime="notice.publishedAt">{{ new Date(notice.publishedAt).toLocaleString([], { dateStyle: 'medium', timeStyle: 'short' }) }}</time>
          </div>
          <h3>{{ notice.title }}</h3>
          <p>{{ notice.content }}</p>
        </article>
      </div>
    </section>
  </div>
</template>
