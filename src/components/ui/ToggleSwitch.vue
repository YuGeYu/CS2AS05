<script setup lang="ts">
defineProps<{ modelValue: boolean; disabled?: boolean; label: string; description?: string }>()
const emit = defineEmits<{ 'update:modelValue': [value: boolean] }>()
</script>

<template>
  <label class="toggle-row" :class="{ 'is-disabled': disabled }">
    <span><strong>{{ label }}</strong><small v-if="description">{{ description }}</small></span>
    <input type="checkbox" :checked="modelValue" :disabled="disabled" @change="emit('update:modelValue', ($event.target as HTMLInputElement).checked)" />
    <span class="toggle-track" aria-hidden="true"><span /></span>
  </label>
</template>
