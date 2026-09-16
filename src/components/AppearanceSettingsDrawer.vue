<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { Check, Layers3, PanelLeft, RotateCcw, Sparkles, X } from 'lucide-vue-next'
import { NAV_ITEMS, type ViewKey } from '@/config/navigation'
import {
  useAppearancePreferences,
  type AccentPalette,
  type RadiusPreference,
  type SidebarMode,
} from '@/composables/useAppearancePreferences'
import { useThemePreference, type AppTheme } from '@/composables/useThemePreference'

defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const { preferences, update, reset } = useAppearancePreferences()
const { setTheme } = useThemePreference()
const themeOptions: Array<{ value: AppTheme; label: string; hint: string }> = [
  { value: 'light', label: '晴昼', hint: '清爽明亮，信息一眼可见' },
  { value: 'dark', label: '静夜', hint: '压低亮度，专注夜间对局' },
]
const paletteOptions: Array<{ value: AccentPalette; label: string; color: string }> = [
  { value: 'default', label: '湛蓝', color: '#2563eb' },
  { value: 'rose', label: '玫瑰', color: '#db2777' },
  { value: 'tide', label: '潮光', color: '#0891b2' },
  { value: 'sunset', label: '日落', color: '#ea580c' },
  { value: 'forest', label: '森林', color: '#15803d' },
  { value: 'sea', label: '海风', color: '#0f766e' },
  { value: 'dream', label: '薰梦', color: '#7c3aed' },
]
const radiusOptions: Array<{ value: RadiusPreference; label: string }> = [
  { value: 'auto', label: '跟随设计' },
  { value: '0', label: '利落' },
  { value: '0.5', label: '柔和' },
  { value: '0.75', label: '圆润' },
  { value: '1.0', label: '饱满' },
]
const sidebarModes: Array<{
  value: SidebarMode
  label: string
  hint: string
  icon: typeof PanelLeft
}> = [
  { value: 'default', label: '默认', hint: '固定侧栏', icon: PanelLeft },
  { value: 'embedded', label: '内嵌', hint: '融入页面', icon: Layers3 },
  { value: 'floating', label: '浮动', hint: '轻盈悬浮', icon: Sparkles },
]
function chooseTheme(theme: AppTheme) {
  setTheme(theme)
  update({ theme })
}
function toggleItem(key: ViewKey, visible: boolean) {
  const hidden = new Set(preferences.hiddenSidebarItems)
  if (visible) hidden.delete(key)
  else hidden.add(key)
  update({ hiddenSidebarItems: [...hidden] })
}
function closeOnEscape(event: KeyboardEvent) {
  if (event.key === 'Escape') emit('close')
}
watch(
  () => preferences.density,
  (density) => {
    if (density !== 'default') update({ density: 'default' })
  },
  { immediate: true },
)
onMounted(() => window.addEventListener('keydown', closeOnEscape))
onBeforeUnmount(() => window.removeEventListener('keydown', closeOnEscape))
</script>

<template>
  <Teleport to="body"
    ><div
      v-if="open"
      class="appearance-drawer-layer"
      role="presentation"
      @click.self="emit('close')"
    >
      <aside
        class="appearance-drawer appearance-drawer--revamped"
        role="dialog"
        aria-modal="true"
        aria-labelledby="appearance-title"
      >
        <header class="appearance-drawer__header">
          <div>
            <p class="overline">外观工坊</p>
            <h2 id="appearance-title">把助手调成你的风格</h2>
            <p>颜色、轮廓与导航方式即时生效，并自动保存在本机。</p>
          </div>
          <button
            class="icon-button appearance-drawer-close"
            type="button"
            aria-label="关闭外观工坊"
            @click="emit('close')"
          >
            <X :size="19" />
          </button>
        </header>
        <div class="appearance-drawer__body">
          <section class="appearance-setting-group">
            <div class="appearance-group-heading">
              <div>
                <h3>明暗主题</h3>
                <p>选择更适合当前环境的视觉亮度。</p>
              </div>
            </div>
            <div class="appearance-choice-grid appearance-choice-grid--two">
              <button
                v-for="item in themeOptions"
                :key="item.value"
                class="appearance-choice appearance-choice--featured"
                :class="{ active: preferences.theme === item.value }"
                type="button"
                @click="chooseTheme(item.value)"
              >
                <span>{{ item.label }}</span
                ><small>{{ item.hint }}</small
                ><Check v-if="preferences.theme === item.value" :size="16" />
              </button>
            </div>
          </section>
          <section class="appearance-setting-group">
            <h3>强调色</h3>
            <div class="appearance-palette-grid">
              <button
                v-for="item in paletteOptions"
                :key="item.value"
                class="appearance-palette"
                :class="{ active: preferences.palette === item.value }"
                type="button"
                :aria-label="`选择${item.label}配色`"
                @click="update({ palette: item.value })"
              >
                <i :style="{ background: item.color }" aria-hidden="true" /><span>{{
                  item.label
                }}</span
                ><Check v-if="preferences.palette === item.value" :size="14" />
              </button>
            </div>
          </section>
          <section class="appearance-setting-group">
            <h3>界面轮廓</h3>
            <div class="appearance-segmented appearance-segmented--radius">
              <button
                v-for="item in radiusOptions"
                :key="item.value"
                type="button"
                :class="{ active: preferences.radius === item.value }"
                @click="update({ radius: item.value })"
              >
                {{ item.label }}
              </button>
            </div>
            <p class="appearance-hint">统一影响页面、卡片、弹窗、输入框和操作控件。</p>
          </section>
          <section class="appearance-setting-group">
            <div class="appearance-group-heading">
              <div>
                <h3>侧栏形态</h3>
                <p>三种布局拥有清晰不同的空间感。</p>
              </div>
            </div>
            <div
              class="appearance-choice-grid appearance-choice-grid--three appearance-sidebar-modes"
            >
              <button
                v-for="item in sidebarModes"
                :key="item.value"
                class="appearance-choice"
                :class="{ active: preferences.sidebarMode === item.value }"
                type="button"
                @click="update({ sidebarMode: item.value })"
              >
                <component :is="item.icon" :size="20" /><span>{{ item.label }}</span
                ><small>{{ item.hint }}</small
                ><Check v-if="preferences.sidebarMode === item.value" :size="14" />
              </button>
            </div>
          </section>
          <section class="appearance-setting-group">
            <div class="appearance-group-heading">
              <div>
                <h3>导航项目</h3>
                <p>安装与诊断始终保留，其他项目按需显示；导航顺序保持固定。</p>
              </div>
              <button class="text-button" type="button" @click="reset">
                <RotateCcw :size="14" />恢复默认
              </button>
            </div>
            <div class="appearance-nav-list">
              <label
                v-for="item in NAV_ITEMS"
                :key="item.key"
                class="appearance-nav-row"
                :data-hidden="preferences.hiddenSidebarItems.includes(item.key)"
                ><component :is="item.icon" :size="18" /><strong>{{ item.label }}</strong
                ><span v-if="item.key === 'install'" class="appearance-required">始终保留</span
                ><input
                  v-else
                  type="checkbox"
                  :checked="!preferences.hiddenSidebarItems.includes(item.key)"
                  :aria-label="`显示${item.label}`"
                  @change="toggleItem(item.key, ($event.target as HTMLInputElement).checked)"
              /></label>
            </div>
          </section>
          <section class="appearance-setting-group">
            <h3>启动体验</h3>
            <label class="appearance-toggle-row"
              ><span
                ><strong>跳过开屏动画</strong
                ><small>每天首次启动仍会展示一次，版本号也可随时重播</small></span
              ><input
                type="checkbox"
                :checked="preferences.skipIntro"
                @change="update({ skipIntro: ($event.target as HTMLInputElement).checked })" /><i
                aria-hidden="true"
                ><b /></i
            ></label>
          </section>
        </div>
        <footer class="appearance-drawer__footer">
          <span><Check :size="15" />所有更改已自动保存</span
          ><button class="primary-button" type="button" @click="emit('close')">完成</button>
        </footer>
      </aside>
    </div></Teleport
  >
</template>
