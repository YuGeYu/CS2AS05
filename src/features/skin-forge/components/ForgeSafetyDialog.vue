<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'
import { ArrowLeft, ShieldAlert, TriangleAlert } from 'lucide-vue-next'

const emit = defineEmits<{ confirm: []; cancel: [] }>()
const dialog = ref<HTMLElement | null>(null)
const confirmButton = ref<HTMLButtonElement | null>(null)

function cancel() { emit('cancel') }
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); cancel(); return }
  if (event.key !== 'Tab' || !dialog.value) return
  const focusable = [...dialog.value.querySelectorAll<HTMLElement>('button:not([disabled]), [tabindex="0"]')]
  const first = focusable[0]
  const last = focusable.at(-1)
  if (!first || !last) return
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus() }
  else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus() }
}

onMounted(async () => {
  document.addEventListener('keydown', onKeydown)
  await nextTick()
  confirmButton.value?.focus()
})
onBeforeUnmount(() => document.removeEventListener('keydown', onKeydown))
</script>

<template>
  <Teleport to="body">
    <div class="modal-backdrop forge-modal-backdrop forge-safety-backdrop" @mousedown.self="cancel">
      <section ref="dialog" class="forge-safety-dialog" role="dialog" aria-modal="true" aria-labelledby="forge-safety-gate-title" aria-describedby="forge-safety-gate-copy">
        <header>
          <span class="forge-safety-dialog__mark"><ShieldAlert :size="24" /></span>
          <div><p class="overline">进入工坊前</p><h2 id="forge-safety-gate-title">请先确认离线使用边界</h2></div>
        </header>
        <div id="forge-safety-gate-copy" class="forge-safety-dialog__copy">
          <p>饰品修改仅用于本地或离线 <code>-insecure</code> 环境，严禁连接 VAC 保护服务器。</p>
          <ul>
            <li><TriangleAlert :size="17" />第三方插件可能带来账号与兼容性风险。</li>
            <li><TriangleAlert :size="17" />游戏更新后，现有功能可能暂时失效。</li>
            <li><TriangleAlert :size="17" />继续使用即表示你已理解并自行承担相关风险。</li>
          </ul>
        </div>
        <footer>
          <button class="secondary-button" type="button" @click="cancel"><ArrowLeft :size="17" />返回概览</button>
          <button ref="confirmButton" class="primary-button" type="button" @click="emit('confirm')"><ShieldAlert :size="17" />我已了解风险，进入工坊</button>
        </footer>
      </section>
    </div>
  </Teleport>
</template>
