<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { BookOpen, ShieldAlert, X } from 'lucide-vue-next'
const { page } = defineProps<{ page: 'tutorial' | 'safety' }>()
const emit = defineEmits<{ close: [] }>()
const closeButton = ref<HTMLButtonElement | null>(null)
function keydown(event: KeyboardEvent) { if (event.key === 'Escape') emit('close') }
onMounted(() => { document.addEventListener('keydown', keydown); closeButton.value?.focus() })
onBeforeUnmount(() => document.removeEventListener('keydown', keydown))
</script>
<template>
  <div class="modal-backdrop forge-modal-backdrop" @mousedown.self="emit('close')">
    <section class="forge-help-dialog" role="dialog" aria-modal="true" :aria-labelledby="`forge-help-${page}`">
      <header>
        <BookOpen v-if="page === 'tutorial'" :size="21" />
        <ShieldAlert v-else :size="21" />
        <div><h2 :id="`forge-help-${page}`">{{ page === 'tutorial' ? '快速上手' : '离线使用说明' }}</h2><p>{{ page === 'tutorial' ? '从目录选择，不需要记住任何编号。' : '明确边界，保护账号与现有游戏环境。' }}</p></div>
        <button ref="closeButton" class="icon-button" type="button" title="关闭" aria-label="关闭" @click="emit('close')"><X :size="18" /></button>
      </header>
      <ol v-if="page === 'tutorial'" class="forge-guide-list">
        <li><strong>选择阵营和模式</strong><span>CT/T 外观分别保存；随机模式会在重生时从插件目录挑选。</span></li>
        <li><strong>点开武器卡片</strong><span>选择皮肤，再调整磨损、图案、贴纸、挂件、命名和计数器。</span></li>
        <li><strong>完成刀具、手套、角色和音乐盒</strong><span>工坊只显示可用组合，不需要输入技术编号。</span></li>
        <li><strong>部署并应用装备</strong><span>关闭 CS2 后部署插件；保存成功会显示目标路径和回读哈希。</span></li>
        <li><strong>在本地对局中刷新</strong><span>重生后查看新外观；可使用 <code>skin_menu</code> 重载装备、<code>skin_random</code> 切换随机模式、<code>skin_reset</code> 重置配置。</span></li>
      </ol>
      <div v-else class="forge-copy-sections">
        <section><h3>只用于本地或离线环境</h3><p>请使用 <code>-insecure</code> 启动参数，并仅进入离线练习或自建测试环境。不要连接 VAC 保护服务器。</p></section>
        <section><h3>工坊不会覆盖基础框架</h3><p>部署仅管理 PlayerSkinMod 自身目录；CounterStrikeSharp 与其他插件由现有安装流程管理。</p></section>
        <section><h3>升级可恢复</h3><p>替换插件前会备份现有目录并保留 <code>player_loadout.json</code>，校验失败会恢复旧文件。</p></section>
      </div>
      <footer><button class="primary-button" type="button" @click="emit('close')">知道了</button></footer>
    </section>
  </div>
</template>
