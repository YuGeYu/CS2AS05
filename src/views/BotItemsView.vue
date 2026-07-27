<script setup lang="ts">
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import type { BotItem } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
const cs2 = useCs2Store(); const panel = usePanelStore()
const items: { key: BotItem; label: string; description: string }[] = [
  { key: 'profiles', label: '选手档案', description: '控制 Bot 档案相关效果。' },
  { key: 'agents', label: '探员模型', description: '控制 Bot 探员模型。' },
  { key: 'music', label: '音乐盒', description: '控制 Bot 音乐盒。' },
  { key: 'weapons', label: '武器皮肤', description: '控制武器皮肤。' },
  { key: 'knives', label: '刀具外观', description: '控制 Bot 刀具外观。' },
  { key: 'gloves', label: '手套外观', description: '控制 Bot 手套外观。' },
  { key: 'stickers', label: '武器印花', description: '控制武器印花。' },
  { key: 'charms', label: '武器挂件', description: '控制武器挂件。' },
]
async function update(item: BotItem, enabled: boolean) {
  try { await panel.setBotItem(cs2.selectedRoot, item, enabled) } catch { /* Store retains the visible error. */ }
}
</script>
<template><section class="tool-view" aria-labelledby="items-title"><header class="view-heading"><div><p class="overline">Bot Randomizer</p><h1 id="items-title">Bot 物品</h1></div></header><section class="toggle-list"><ToggleSwitch v-for="item in items" :key="item.key" :model-value="panel.snapshot?.botItems[item.key] ?? false" :label="item.label" :description="item.description" :disabled="!panel.snapshot?.ready || !!panel.mutationKey" @update:model-value="update(item.key, $event)" /></section><p v-if="panel.snapshot?.cs2Running" class="warning-note">修改将在重启 CS2 后完整生效。</p><p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p></section></template>
