<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { Check, ChevronDown, ChevronUp, RotateCcw, X } from 'lucide-vue-next'
import { NAV_ITEMS, type ViewKey } from '@/config/navigation'
import { useAppearancePreferences, type AccentPalette, type DensityPreference, type RadiusPreference, type SidebarMode } from '@/composables/useAppearancePreferences'
import { useThemePreference, type AppTheme } from '@/composables/useThemePreference'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const { preferences, update, reset } = useAppearancePreferences()
const { setTheme } = useThemePreference()
const themeOptions: Array<{ value: AppTheme; label: string; hint: string }> = [
  { value: 'light', label: '浅色', hint: '清爽明亮' }, { value: 'dark', label: '深色', hint: '夜间舒适' },
]
const paletteOptions: Array<{ value: AccentPalette; label: string; color: string }> = [
  { value: 'default', label: '默认', color: '#2563eb' }, { value: 'rose', label: '玫瑰', color: '#db2777' },
  { value: 'tide', label: '潮光', color: '#0891b2' }, { value: 'sunset', label: '日落', color: '#ea580c' },
  { value: 'forest', label: '森林', color: '#15803d' }, { value: 'sea', label: '海风', color: '#0f766e' },
  { value: 'dream', label: '薰梦', color: '#7c3aed' },
]
const radiusOptions: RadiusPreference[] = ['auto', '0', '0.25', '0.5', '0.75', '1.0']
const radiusLabels: Record<RadiusPreference, string> = { auto: 'auto', '0': '0', '0.25': '0.25', '0.5': '0.5', '0.75': '0.75', '1.0': '1.0' }
const densityOptions: Array<{ value: DensityPreference; label: string; hint: string }> = [
  { value: 'compact', label: '紧凑', hint: '更多内容' }, { value: 'default', label: '默认', hint: '平衡舒适' }, { value: 'loose', label: '宽松', hint: '更宽呼吸感' },
]
const sidebarModes: Array<{ value: SidebarMode; label: string; hint: string }> = [
  { value: 'default', label: '默认', hint: '固定侧栏' }, { value: 'embedded', label: '内嵌', hint: '融入页面' }, { value: 'floating', label: '浮动', hint: '轻盈悬浮' },
]
const orderedItems = computed(() => preferences.sidebarOrder.map(key => NAV_ITEMS.find(item => item.key === key)).filter(Boolean))
function chooseTheme(theme: AppTheme) { setTheme(theme); update({ theme }) }
function move(key: ViewKey, direction: -1 | 1) {
  const order = [...preferences.sidebarOrder]; const index = order.indexOf(key); const next = index + direction
  if (index < 0 || next < 0 || next >= order.length) return
  const current = order[index] as ViewKey
  order[index] = order[next] as ViewKey
  order[next] = current
  update({ sidebarOrder: order })
}
function toggleItem(key: ViewKey, visible: boolean) {
  const hidden = new Set(preferences.hiddenSidebarItems)
  if (visible) hidden.delete(key)
  else hidden.add(key)
  update({ hiddenSidebarItems: [...hidden] })
}
function closeOnEscape(event: KeyboardEvent) { if (event.key === 'Escape' && props.open) emit('close') }
watch(() => props.open, open => { if (open) void nextTick(() => document.querySelector<HTMLElement>('.appearance-drawer-close')?.focus()) })
onMounted(() => window.addEventListener('keydown', closeOnEscape))
onBeforeUnmount(() => window.removeEventListener('keydown', closeOnEscape))
</script>

<template>
  <Teleport to="body">
    <div v-if="open" class="appearance-drawer-layer" role="presentation" @click.self="emit('close')">
      <aside class="appearance-drawer" role="dialog" aria-modal="true" aria-labelledby="appearance-title">
        <header class="appearance-drawer__header">
          <div><p class="overline">安装与诊断 · 个性化</p><h2 id="appearance-title">主题设置</h2><p>把助手调成你顺手的样子，设置会自动保存。</p></div>
          <button class="icon-button appearance-drawer-close" type="button" aria-label="关闭主题设置" title="关闭主题设置" @click="emit('close')"><X :size="19" /></button>
        </header>
        <div class="appearance-drawer__body">
          <section class="appearance-setting-group"><h3>主题</h3><div class="appearance-choice-grid appearance-choice-grid--two"><button v-for="item in themeOptions" :key="item.value" class="appearance-choice" :class="{ active: preferences.theme === item.value }" type="button" @click="chooseTheme(item.value)"><span>{{ item.label }}</span><small>{{ item.hint }}</small><Check v-if="preferences.theme === item.value" :size="15" /></button></div></section>
          <section class="appearance-setting-group"><h3>颜色</h3><div class="appearance-palette-grid"><button v-for="item in paletteOptions" :key="item.value" class="appearance-palette" :class="{ active: preferences.palette === item.value }" type="button" :aria-label="`选择${item.label}配色`" @click="update({ palette: item.value })"><i :style="{ background: item.color }" aria-hidden="true" /><span>{{ item.label }}</span><Check v-if="preferences.palette === item.value" :size="14" /></button></div></section>
          <section class="appearance-setting-group"><h3>圆角</h3><div class="appearance-segmented"> <button v-for="item in radiusOptions" :key="item" type="button" :class="{ active: preferences.radius === item }" @click="update({ radius: item })">{{ radiusLabels[item] }}</button></div><p class="appearance-hint">控制按钮、卡片和面板的圆角程度。</p></section>
          <section class="appearance-setting-group"><h3>布局</h3><div class="appearance-choice-grid appearance-choice-grid--three"><button v-for="item in densityOptions" :key="item.value" class="appearance-choice" :class="{ active: preferences.density === item.value }" type="button" @click="update({ density: item.value })"><span>{{ item.label }}</span><small>{{ item.hint }}</small></button></div></section>
          <section class="appearance-setting-group"><h3>启动体验</h3><label class="appearance-toggle-row"><span><strong>取消开屏动画</strong><small>启动时直接进入助手主界面</small></span><input type="checkbox" :checked="preferences.skipIntro" @change="update({ skipIntro: ($event.target as HTMLInputElement).checked })" /><i aria-hidden="true"><b /></i></label></section>
          <section class="appearance-setting-group"><h3>左侧侧边栏</h3><div class="appearance-choice-grid appearance-choice-grid--three"><button v-for="item in sidebarModes" :key="item.value" class="appearance-choice" :class="{ active: preferences.sidebarMode === item.value }" type="button" @click="update({ sidebarMode: item.value })"><span>{{ item.label }}</span><small>{{ item.hint }}</small></button></div></section>
          <section class="appearance-setting-group"><div class="appearance-group-heading"><div><h3>左侧侧边栏项目</h3><p>安装与诊断始终保留，其他项目可以隐藏或调整顺序。</p></div><button class="text-button" type="button" @click="reset"><RotateCcw :size="14" />恢复默认</button></div><div class="appearance-nav-list"><div v-for="(item, index) in orderedItems" :key="item!.key" class="appearance-nav-row" :data-hidden="preferences.hiddenSidebarItems.includes(item!.key)"><span class="appearance-nav-index">{{ String(index + 1).padStart(2, '0') }}</span><component :is="item!.icon" :size="17" /><strong>{{ item!.label }}</strong><label v-if="item!.key !== 'install'" class="appearance-nav-visibility"><input type="checkbox" :checked="!preferences.hiddenSidebarItems.includes(item!.key)" @change="toggleItem(item!.key, ($event.target as HTMLInputElement).checked)" /><span>显示</span></label><span v-else class="appearance-required">必选</span><button class="icon-button" type="button" :disabled="index === 0" :aria-label="`将${item!.label}上移`" title="上移" @click="move(item!.key, -1)"><ChevronUp :size="16" /></button><button class="icon-button" type="button" :disabled="index === orderedItems.length - 1" :aria-label="`将${item!.label}下移`" title="下移" @click="move(item!.key, 1)"><ChevronDown :size="16" /></button></div></div></section>
        </div>
        <footer class="appearance-drawer__footer"><span><Check :size="15" />设置会自动保存到本机</span><button class="primary-button" type="button" @click="emit('close')">完成</button></footer>
      </aside>
    </div>
  </Teleport>
</template>
