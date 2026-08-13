<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { Search, X } from 'lucide-vue-next'
import ForgeImage from './ForgeImage.vue'
import type { CatalogItem } from '@/features/skin-forge/data/catalog'
import { filterCatalog } from '@/features/skin-forge/data/catalog'

const props = withDefaults(defineProps<{
  items: CatalogItem[]
  selectedId?: string | null
  label: string
  loading?: boolean
  pageSize?: number
  compact?: boolean
  disabled?: boolean
}>(), { selectedId: null, loading: false, pageSize: 96, compact: false, disabled: false })
const emit = defineEmits<{ select: [item: CatalogItem] }>()
const query = defineModel<string>('query', { default: '' })
const limit = ref(props.pageSize)
const filtered = computed(() => filterCatalog(props.items, query.value))
const visible = computed(() => filtered.value.slice(0, limit.value))
watch(query, () => { limit.value = props.pageSize })

</script>

<template>
  <div class="catalog-browser">
    <label class="forge-search">
      <Search :size="17" aria-hidden="true" />
      <span class="sr-only">搜索{{ label }}</span>
      <input v-model="query" type="search" :disabled="disabled" :placeholder="`搜索${label}名称`" autocomplete="off" />
      <button v-if="query" type="button" :disabled="disabled" title="清除搜索" aria-label="清除搜索" @click="query = ''"><X :size="16" /></button>
    </label>
    <div class="catalog-result-line" aria-live="polite">
      <span>{{ loading ? '正在载入目录…' : `${filtered.length.toLocaleString('zh-CN')} 项` }}</span>
      <span v-if="filtered.length > visible.length">当前展示 {{ visible.length }} 项</span>
    </div>
    <div v-if="!loading && visible.length" class="catalog-grid" :class="{ 'catalog-grid--compact': compact }" role="listbox" :aria-label="label">
      <button
        v-for="item in visible"
        :key="item.id"
        type="button"
        class="catalog-card"
        :class="{ 'is-selected': selectedId === item.id }"
        role="option"
        :aria-selected="selectedId === item.id"
        :disabled="disabled"
        :title="item.secondary ? `${item.name} / ${item.secondary}` : item.name"
        @click="emit('select', item)"
      >
        <span class="catalog-card__media">
          <ForgeImage :src="item.image" :alt="item.name" />
        </span>
        <strong>{{ item.name }}</strong>
        <small v-if="item.secondary">{{ item.secondary }}</small>
      </button>
    </div>
    <div v-else-if="!loading" class="forge-empty"><Search :size="22" /><strong>没有找到匹配内容</strong><span>换一个名称或清除搜索条件。</span></div>
    <button v-if="visible.length < filtered.length" class="secondary-button catalog-more" type="button" :disabled="disabled" @click="limit += pageSize">继续显示 {{ Math.min(pageSize, filtered.length - visible.length) }} 项</button>
  </div>
</template>
