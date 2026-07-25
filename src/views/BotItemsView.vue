<script setup lang="ts">
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import type { BotItem } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
const cs2 = useCs2Store(); const panel = usePanelStore()
const items: { key: BotItem; label: string; description: string }[] = [
  { key: 'skins', label: '武器皮肤', description: '允许 Bot 随机使用武器皮肤。' },
  { key: 'profiles', label: '选手档案', description: '启用 Bot 选手资料随机化。' },
  { key: 'agents', label: '探员模型', description: '启用 Bot 探员模型随机化。' },
  { key: 'music', label: '音乐盒', description: '启用 Bot 音乐盒随机化。' },
]
const update = (item: BotItem, enabled: boolean) => panel.setBotItem(cs2.selectedRoot, item, enabled).catch(() => undefined)
</script>
<template><section class="tool-view" aria-labelledby="items-title"><header class="view-heading"><div><p class="overline">Bot Randomizer</p><h1 id="items-title">Bot 物品</h1></div></header><section class="toggle-list"><ToggleSwitch v-for="item in items" :key="item.key" :model-value="panel.snapshot?.botItems[item.key] ?? false" :label="item.label" :description="item.description" :disabled="!panel.snapshot?.ready || !!panel.mutationKey" @update:model-value="update(item.key, $event)" /></section><p v-if="panel.snapshot?.cs2Running" class="warning-note">修改将在重启 CS2 后完整生效。</p></section></template>
