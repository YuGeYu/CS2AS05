<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { ExternalLink, Heart, MessageCircle, X } from 'lucide-vue-next'
import { openCommunityGroup, openResourceLink } from '@/services/tauri/support'
import type { PromotionPayload } from '@/features/promotion-push/types'

const props = defineProps<{ promotion: PromotionPayload | null; disabled?: boolean }>()
const emit = defineEmits<{ close: []; disable: [] }>()
const closeButton = ref<HTMLButtonElement | null>(null)
const donateCodeUrl = '/assets/wechat-reward.png'
let previousOverflow = ''

function close() { emit('close') }
function disable() { emit('disable') }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape') close() }
function openLink(url: string) { void openResourceLink(url).catch(() => undefined) }
function openCommunityGroupLink() { void openCommunityGroup().catch(() => undefined) }

watch(() => props.promotion, async (promotion, previous) => {
  if (promotion) {
    if (!previous) {
      previousOverflow = document.body.style.overflow
      document.body.style.overflow = 'hidden'
      window.addEventListener('keydown', onKeydown)
    }
    await nextTick()
    closeButton.value?.focus()
  } else {
    document.body.style.overflow = previousOverflow
    window.removeEventListener('keydown', onKeydown)
  }
}, { immediate: true })
</script>

<template>
  <Teleport to="body">
    <div v-if="promotion" class="modal-backdrop promotion-push-backdrop" role="presentation">
      <section class="confirm-dialog promotion-push-dialog" role="dialog" aria-modal="true" aria-labelledby="promotion-push-title" aria-describedby="promotion-push-description">
        <header class="promotion-push-header">
          <div>
            <p class="overline">凉拌娱乐 · 小小分享</p>
            <h2 id="promotion-push-title">
              <span v-if="promotion.kind === 'resource'"><ExternalLink :size="19" aria-hidden="true" />资源阁新发现</span>
              <span v-else-if="promotion.localId === 'community'"><MessageCircle :size="19" aria-hidden="true" />加入玩家交流</span>
              <span v-else><Heart :size="19" aria-hidden="true" />支持助手维护</span>
            </h2>
          </div>
          <button ref="closeButton" class="icon-button" type="button" title="暂不查看" aria-label="暂不查看" @click="close"><X :size="19" /></button>
        </header>

        <template v-if="promotion.kind === 'resource'">
          <p id="promotion-push-description" class="promotion-push-lead">资源阁最近更新了一份资源，打开详情看看是否正好派得上用场。</p>
          <article class="promotion-resource-card">
            <p class="overline">{{ promotion.resource.category }}<span v-if="promotion.resource.panType"> · {{ promotion.resource.panType }}</span></p>
            <h3>{{ promotion.resource.title }}</h3>
            <p>{{ promotion.resource.summary || '打开资源阁查看完整说明。' }}</p>
            <div class="promotion-resource-tags"><span v-for="tag in promotion.resource.tags.slice(0, 4)" :key="tag">#{{ tag }}</span></div>
          </article>
          <div class="dialog-actions promotion-push-actions">
            <button class="secondary-button" type="button" @click="openLink('https://vault.600318.xyz')"><ExternalLink :size="16" />打开资源阁</button>
            <button v-if="promotion.resource.obtainUrl" class="primary-button" type="button" @click="openLink(promotion.resource.obtainUrl)"><ExternalLink :size="16" />查看获取链接</button>
          </div>
        </template>

        <template v-else-if="promotion.localId === 'community'">
          <p id="promotion-push-description" class="promotion-push-lead">来玩家群聊聊配置、地图和实战体验。进群自愿，不影响助手任何功能。</p>
          <button class="primary-button promotion-full-button" type="button" @click="openCommunityGroupLink"><MessageCircle :size="17" />打开“凉拌娱乐”QQ群</button>
        </template>

        <template v-else>
          <p id="promotion-push-description" class="promotion-push-lead">如果助手帮到了你，可以用微信赞赏支持持续维护。不赞助不影响任何功能，按你的心意即可。</p>
          <img :src="donateCodeUrl" alt="微信赞赏码" class="donate-code promotion-push-donate-code" />
        </template>
        <footer class="promotion-push-footer">
          <button class="promotion-muted-action" type="button" @click="disable">以后不再主动提醒</button>
          <span>可随时在资源阁、群聊或关于页面主动访问</span>
        </footer>
      </section>
    </div>
  </Teleport>
</template>
