<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'

const props = defineProps<{ modelValue: boolean; disabled?: boolean; label: string; description?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
const justEnabled = ref(false)
let feedbackTimer: ReturnType<typeof setTimeout> | undefined

function requestChange(event: Event) {
  const input = event.target as HTMLInputElement
  const requested = input.checked
  input.checked = props.modelValue
  emit('update:modelValue', requested)
}

watch(() => props.modelValue, (current, previous) => {
  if (!current || previous !== false) return
  if (feedbackTimer) clearTimeout(feedbackTimer)
  justEnabled.value = true
  feedbackTimer = setTimeout(() => { justEnabled.value = false; feedbackTimer = undefined }, 420)
})

onBeforeUnmount(() => { if (feedbackTimer) clearTimeout(feedbackTimer) })
</script>

<template>
  <label class="toggle-row" :class="{ 'is-disabled': disabled, 'is-just-enabled': justEnabled }">
    <span><strong>{{ label }}</strong><small v-if="description">{{ description }}</small></span>
    <input type="checkbox" :checked="modelValue" :disabled="disabled" @change="requestChange" />
    <span class="toggle-track" aria-hidden="true"><span /></span>
  </label>
</template>
