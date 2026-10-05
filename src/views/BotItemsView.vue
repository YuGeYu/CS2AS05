<script setup lang="ts">
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import type { BotItem } from '@/features/panel/types'
import { usePanelStore } from '@/stores/panel'
const panel = usePanelStore()
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
</script>
<template><section class="tool-view" aria-labelledby="items-title"><header class="view-heading"><div><p class="overline">官方 Panel · Bot Randomizer</p><h1 id="items-title">Bot 物品</h1></div></header><p class="warning-note" role="status">BOT 物品当前全部开启且暂不可更改。由于技术限制，分类开关与机器人物品档案请打开上游 Panel v1.4.5 使用。</p><section class="toggle-list"><ToggleSwitch v-for="item in items" :key="item.key" :model-value="true" :label="item.label" :description="item.description" :disabled="true" /></section><p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p></section></template>

