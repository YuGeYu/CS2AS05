<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'
import { Heart, X } from 'lucide-vue-next'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const closeButton = ref<HTMLButtonElement | null>(null)
const donateCodeUrl = '/assets/wechat-reward.png'
let previousOverflow = ''
function close() { emit('close') }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape') close() }
watch(() => props.open, async open => {
  if (open) { previousOverflow = document.body.style.overflow; document.body.style.overflow = 'hidden'; window.addEventListener('keydown', onKeydown); await nextTick(); closeButton.value?.focus() }
  else { document.body.style.overflow = previousOverflow; window.removeEventListener('keydown', onKeydown) }
})
</script>
<template>
  <Teleport to="body"><div v-if="open" class="modal-backdrop donate-backdrop" role="presentation" @click.self="close">
    <section class="confirm-dialog donate-dialog" role="dialog" aria-modal="true" aria-labelledby="donate-title">
      <header><div><p class="overline">支持持续维护</p><h2 id="donate-title"><Heart :size="19" aria-hidden="true" />支持助手维护</h2></div><button ref="closeButton" class="icon-button" type="button" aria-label="关闭赞助窗口" @click="close"><X :size="19" /></button></header>
      <p>感谢每一位愿意支持项目的玩家。不赞助不影响任何功能，按你的心意即可。</p>
      <img :src="donateCodeUrl" alt="微信赞赏码" class="donate-code" />
    </section>
  </div></Teleport>
</template>
