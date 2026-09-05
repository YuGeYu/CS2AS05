<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Route, X } from 'lucide-vue-next'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const closeButton = ref<HTMLButtonElement | null>(null)
function close() { emit('close') }
function onKeydown(event: KeyboardEvent) { if (event.key === 'Escape') close() }
watch(() => props.open, open => { if (open) { window.addEventListener('keydown', onKeydown); closeButton.value?.focus() } else window.removeEventListener('keydown', onKeydown) })
onMounted(() => { if (props.open) window.addEventListener('keydown', onKeydown) })
onBeforeUnmount(() => window.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop map-rotation-notice-backdrop" role="presentation" @click.self="close">
      <section class="confirm-dialog map-rotation-notice" role="dialog" aria-modal="true" aria-labelledby="map-rotation-notice-title">
        <header>
          <div><p class="overline">实验功能</p><h2 id="map-rotation-notice-title"><Route :size="19" aria-hidden="true" />自动换图默认状态</h2></div>
          <button ref="closeButton" class="icon-button" type="button" aria-label="关闭自动换图默认状态说明" title="关闭说明" @click="close"><X :size="19" /></button>
        </header>
        <p class="map-rotation-notice-status">功能开发中，当前暂时无效</p>
        <p>助手中的配置回读不等于游戏插件当前运行状态。插件侧兼容修复完成并通过真实 CS2 验证后，再恢复此入口。</p>
      </section>
    </div>
  </Teleport>
</template>
