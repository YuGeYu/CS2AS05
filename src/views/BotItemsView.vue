<script setup lang="ts">
import ToggleSwitch from '@/components/ui/ToggleSwitch.vue'
import type { BotItem } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'
import { getBotChatConfig, setBotChatConfig } from '@/services/tauri/botChatConfig'
import { ref } from 'vue'
const cs2 = useCs2Store(); const panel = usePanelStore()
const botConfig = ref(''); const configBusy = ref(false); const configMessage = ref('')
async function loadBotConfig() { if (!cs2.selectedRoot) return; configBusy.value = true; configMessage.value = ''; try { botConfig.value = await getBotChatConfig(cs2.selectedRoot) } catch (e) { configMessage.value = String(e) } finally { configBusy.value = false } }
async function saveBotConfig() { if (!cs2.selectedRoot) return; configBusy.value = true; configMessage.value = ''; try { botConfig.value = await setBotChatConfig(cs2.selectedRoot, botConfig.value); configMessage.value = '配置已保存；重启 CS2 或重载插件后生效。' } catch (e) { configMessage.value = String(e) } finally { configBusy.value = false } }
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
<template><section class="tool-view" aria-labelledby="items-title"><header class="view-heading"><div><p class="overline">Bot Randomizer</p><h1 id="items-title">Bot 物品</h1></div></header><section class="toggle-list"><ToggleSwitch v-for="item in items" :key="item.key" :model-value="panel.snapshot?.botItems[item.key] ?? false" :label="item.label" :description="item.description" :disabled="!panel.snapshot?.ready || (panel.snapshot?.cs2Running && !cs2.writeUnlocked) || !!panel.mutationKey" @update:model-value="update(item.key, $event)" /></section><section class="bot-chat-config"><header><div><p class="overline">CS2BotLlmChat</p><h2>BOT 发言配置</h2><p>读取并修改游戏内 BOT 发言插件配置。API 密钥默认为空，仅在这里填写后保存。</p></div><div><button type="button" @click="loadBotConfig" :disabled="configBusy || !cs2.selectedRoot">读取配置</button><button type="button" @click="saveBotConfig" :disabled="configBusy || !botConfig">保存配置</button></div></header><textarea v-model="botConfig" spellcheck="false" aria-label="BOT 发言 JSON 配置" placeholder="点击“读取配置”载入 JSON 配置" /><p v-if="configMessage" class="inline-status" role="status">{{ configMessage }}</p></section><p v-if="cs2.cs2Running" class="warning-note">{{ cs2.writeUnlocked ? '已确认 CS2 关闭，修改已解锁；重新启动 CS2 后完整生效。' : '检测到 CS2 仍在运行，可在全局状态条确认已关闭后解锁。' }}</p><p v-if="panel.lastError" class="inline-error" role="alert">{{ panel.lastError }}</p></section></template>

