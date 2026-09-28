<script setup lang="ts">
defineProps<{ open: boolean; title: string; description: string; confirmLabel?: string; busy?: boolean }>()
const emit = defineEmits<{ cancel: []; confirm: [] }>()
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="modal-backdrop" role="presentation" @click.self="emit('cancel')">
      <section class="confirm-dialog" role="dialog" aria-modal="true" :aria-labelledby="`${title}-confirm-title`">
        <h2 :id="`${title}-confirm-title`">{{ title }}</h2>
        <p>{{ description }}</p>
        <div class="confirm-dialog-actions"><button class="text-button" type="button" :disabled="busy" @click="emit('cancel')">取消</button><button class="danger-button" type="button" :disabled="busy" @click="emit('confirm')">{{ busy ? '处理中…' : confirmLabel || '确认' }}</button></div>
      </section>
    </div>
  </Teleport>
</template>
