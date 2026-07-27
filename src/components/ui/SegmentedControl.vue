<script setup lang="ts" generic="T extends string">
import { computed } from 'vue'

const props = defineProps<{ modelValue: T | null; options: readonly { value: T; label: string }[]; disabled?: boolean; pending?: boolean; label: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: T] }>()
const activeIndex = computed(() => props.options.findIndex(option => option.value === props.modelValue))
const indicatorStyle = computed(() => ({ '--segment-count': props.options.length, '--segment-index': Math.max(activeIndex.value, 0) }))
</script>

<template>
  <div class="segmented" role="group" :aria-label="label" :aria-busy="pending || undefined" :style="indicatorStyle">
    <span v-if="activeIndex >= 0" class="segmented__indicator" aria-hidden="true" />
    <button v-for="option in options" :key="option.value" type="button" :aria-pressed="modelValue === option.value" :disabled="disabled || pending" @click="emit('update:modelValue', option.value)">
      {{ option.label }}
    </button>
  </div>
</template>
