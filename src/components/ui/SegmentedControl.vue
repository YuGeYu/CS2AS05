<script setup lang="ts" generic="T extends string">
defineProps<{ modelValue: T | null; options: readonly { value: T; label: string }[]; disabled?: boolean; label: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: T] }>()
</script>

<template>
  <div class="segmented" role="group" :aria-label="label">
    <button v-for="option in options" :key="option.value" type="button" :aria-pressed="modelValue === option.value" :disabled="disabled" @click="emit('update:modelValue', option.value)">
      {{ option.label }}
    </button>
  </div>
</template>
