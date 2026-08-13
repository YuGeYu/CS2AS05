<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, toRaw } from 'vue'
import { Check, Info, LoaderCircle, Plus, Trash2, X } from 'lucide-vue-next'
import CatalogGrid from './CatalogGrid.vue'
import {
  keychainCatalog,
  loadStickerCatalog,
  weaponSkinCatalog,
  type CatalogItem,
} from '@/features/skin-forge/data/catalog'
import type { Team, WeaponLoadout } from '@/types/skin-forge'

const props = defineProps<{ weapon: WeaponLoadout; weaponName: string; team: Team; disabled?: boolean }>()
const emit = defineEmits<{ close: []; save: [weapon: WeaponLoadout, done: (result: { ok: true } | { ok: false; message: string }) => void] }>()
// Props from Pinia are Vue reactive proxies; structuredClone rejects Proxy values.
// Clone the raw snapshot at the component boundary so the draft and emitted payload
// remain ordinary serializable loadout data.
function cloneWeapon(value: WeaponLoadout): WeaponLoadout {
  return structuredClone(toRaw(value))
}
const draft = ref<WeaponLoadout>(cloneWeapon(props.weapon))
const tab = ref<'skin' | 'stickers' | 'keychain' | 'details'>('skin')
const stickerItems = shallowRef<CatalogItem[]>([])
const stickerLoading = ref(false)
const activeSticker = ref(0)
const dialog = ref<HTMLElement | null>(null)
const closeButton = ref<HTMLButtonElement | null>(null)
const submitted = ref(false)
const submitError = ref('')
let submitWatchdog: ReturnType<typeof setTimeout> | undefined
const skinItems = computed(() => weaponSkinCatalog(draft.value.defindex))
const selectedSkin = computed(() => draft.value.paintKit < 0 ? null : `${draft.value.defindex}_${draft.value.paintKit}`)
const currentSticker = computed(() => draft.value.stickers[activeSticker.value])
const selectedSticker = computed(() => currentSticker.value?.id ? `sticker_${currentSticker.value.id}` : null)
const selectedKeychain = computed(() => draft.value.keychain?.id ? `keychain_${draft.value.keychain.id}` : null)

async function selectTab(value: typeof tab.value) {
  tab.value = value
  if (value === 'stickers' && !stickerItems.value.length) {
    stickerLoading.value = true
    try { stickerItems.value = await loadStickerCatalog() } finally { stickerLoading.value = false }
  }
}
function selectSkin(item: CatalogItem) { draft.value.paintKit = item.numericId }
function selectSticker(item: CatalogItem) {
  const stickers = [...draft.value.stickers]
  stickers[activeSticker.value] = { id: item.numericId, schema: 0, offsetX: 0, offsetY: 0, wear: 0, scale: 1, rotation: 0 }
  draft.value.stickers = stickers.slice(0, 5)
}
function removeSticker() {
  draft.value.stickers.splice(activeSticker.value, 1)
  activeSticker.value = Math.max(0, Math.min(activeSticker.value, draft.value.stickers.length - 1))
}
function addSticker() {
  if (draft.value.stickers.length >= 5) return
  draft.value.stickers.push({ id: 0, schema: 0, offsetX: 0, offsetY: 0, wear: 0, scale: 1, rotation: 0 })
  activeSticker.value = draft.value.stickers.length - 1
}
function selectKeychain(item: CatalogItem) { draft.value.keychain = { id: item.numericId, offsetX: 0, offsetY: 0, offsetZ: 0, seed: 0 } }
function save() {
  if (props.disabled || submitted.value || draft.value.defindex !== props.weapon.defindex) return
  submitted.value = true
  submitError.value = ''
  let acknowledged = false
  const done = (result: { ok: true } | { ok: false; message: string }) => {
    if (acknowledged) return
    acknowledged = true
    if (submitWatchdog) clearTimeout(submitWatchdog)
    if (!result.ok) { submitted.value = false; submitError.value = result.message || '提交失败，请重试'; return }
  }
  submitWatchdog = setTimeout(() => {
    if (acknowledged) return
    acknowledged = true
    submitted.value = false
    submitError.value = '提交确认超时，请重试'
  }, 10_000)
  try { emit('save', cloneWeapon(draft.value), done) } catch (error) { done({ ok: false, message: String(error) }) }
}
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape') { event.preventDefault(); emit('close'); return }
  if (event.key !== 'Tab' || !dialog.value) return
  const focusable = [...dialog.value.querySelectorAll<HTMLElement>('button:not([disabled]), input:not([disabled]), [tabindex="0"]')]
  if (!focusable.length) return
  const first = focusable[0]!; const last = focusable.at(-1)!
  if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last.focus() }
  else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first.focus() }
}
onMounted(async () => { document.addEventListener('keydown', onKeydown); await nextTick(); closeButton.value?.focus() })
onBeforeUnmount(() => { document.removeEventListener('keydown', onKeydown); if (submitWatchdog) clearTimeout(submitWatchdog) })
</script>

<template>
  <div class="modal-backdrop forge-modal-backdrop" @mousedown.self="emit('close')">
    <section ref="dialog" class="forge-dialog" role="dialog" aria-modal="true" aria-labelledby="weapon-editor-title">
      <header class="forge-dialog__header">
        <div><span class="forge-team-mark" :data-team="team">{{ team.toUpperCase() }}</span><h2 id="weapon-editor-title">{{ weaponName }}</h2><p>选择外观并调整细节。附件由两个阵营共用。</p></div>
        <button ref="closeButton" class="icon-button" type="button" title="关闭编辑器" aria-label="关闭编辑器" @click="emit('close')"><X :size="19" /></button>
      </header>
      <nav class="forge-editor-tabs" aria-label="武器编辑分类">
        <button v-for="item in [{ id: 'skin', label: '皮肤' }, { id: 'stickers', label: '贴纸' }, { id: 'keychain', label: '挂件' }, { id: 'details', label: '详细设置' }]" :key="item.id" type="button" :aria-current="tab === item.id ? 'page' : undefined" @click="selectTab(item.id as 'skin' | 'stickers' | 'keychain' | 'details')">{{ item.label }}</button>
      </nav>
      <div class="forge-dialog__body">
        <div v-if="tab === 'skin'" class="forge-editor-layout">
          <CatalogGrid :items="skinItems" :selected-id="selectedSkin" label="武器皮肤" @select="selectSkin" />
          <aside class="forge-parameter-panel">
            <button class="secondary-button" type="button" @click="draft.paintKit = -1">每次重生随机皮肤</button>
            <label>磨损 <output>{{ draft.wear.toFixed(4) }}</output><input v-model.number="draft.wear" type="range" min="0" max="1" step="0.0001" /></label>
            <label>图案模板 <output>{{ draft.seed }}</output><input v-model.number="draft.seed" type="range" min="0" max="1000" step="1" /></label>
          </aside>
        </div>
        <div v-else-if="tab === 'stickers'" class="forge-editor-layout">
          <CatalogGrid :items="stickerItems" :selected-id="selectedSticker" :loading="stickerLoading" :page-size="100" compact label="贴纸" @select="selectSticker" />
          <aside class="forge-parameter-panel">
            <div class="shared-notice"><Info :size="16" /><span>贴纸按武器共用，CT/T 切换不会复制成两套。</span></div>
            <div class="sticker-slots">
              <button v-for="(_, index) in draft.stickers" :key="index" type="button" :aria-pressed="activeSticker === index" @click="activeSticker = index">{{ index + 1 }}</button>
              <button v-if="draft.stickers.length < 5" type="button" title="添加贴纸槽" aria-label="添加贴纸槽" @click="addSticker"><Plus :size="15" /></button>
            </div>
            <template v-if="currentSticker">
              <label>水平位置 <output>{{ currentSticker.offsetX.toFixed(2) }}</output><input v-model.number="currentSticker.offsetX" type="range" min="-2" max="2" step="0.01" /></label>
              <label>垂直位置 <output>{{ currentSticker.offsetY.toFixed(2) }}</output><input v-model.number="currentSticker.offsetY" type="range" min="-2" max="2" step="0.01" /></label>
              <label>缩放 <output>{{ currentSticker.scale.toFixed(2) }}</output><input v-model.number="currentSticker.scale" type="range" min="0.1" max="5" step="0.01" /></label>
              <label>旋转 <output>{{ currentSticker.rotation }}°</output><input v-model.number="currentSticker.rotation" type="range" min="-180" max="180" step="1" /></label>
              <label>磨损 <output>{{ currentSticker.wear.toFixed(2) }}</output><input v-model.number="currentSticker.wear" type="range" min="0" max="1" step="0.01" /></label>
              <button class="danger-quiet-button" type="button" @click="removeSticker"><Trash2 :size="16" />移除当前贴纸</button>
            </template>
            <p v-else class="parameter-hint">最多可添加 5 张贴纸。先添加槽位，再从目录选择。</p>
          </aside>
        </div>
        <div v-else-if="tab === 'keychain'" class="forge-editor-layout">
          <CatalogGrid :items="keychainCatalog" :selected-id="selectedKeychain" compact label="挂件" @select="selectKeychain" />
          <aside class="forge-parameter-panel">
            <div class="shared-notice"><Info :size="16" /><span>挂件按武器共用，保存后不会因队伍切换而覆盖。</span></div>
            <template v-if="draft.keychain">
              <label>水平位置 <output>{{ draft.keychain.offsetX.toFixed(2) }}</output><input v-model.number="draft.keychain.offsetX" type="range" min="-5" max="5" step="0.01" /></label>
              <label>垂直位置 <output>{{ draft.keychain.offsetY.toFixed(2) }}</output><input v-model.number="draft.keychain.offsetY" type="range" min="-5" max="5" step="0.01" /></label>
              <label>深度位置 <output>{{ draft.keychain.offsetZ.toFixed(2) }}</output><input v-model.number="draft.keychain.offsetZ" type="range" min="-5" max="5" step="0.01" /></label>
              <label>挂件图案 <output>{{ draft.keychain.seed }}</output><input v-model.number="draft.keychain.seed" type="range" min="0" max="9999" step="1" /></label>
              <button class="danger-quiet-button" type="button" @click="draft.keychain = null"><Trash2 :size="16" />移除挂件</button>
            </template>
            <p v-else class="parameter-hint">从左侧选择一个挂件后即可调整位置。</p>
          </aside>
        </div>
        <div v-else class="forge-details-form">
          <div class="shared-notice"><Info :size="16" /><span>命名标签与计数器按武器共用。</span></div>
          <label>命名标签<input v-model="draft.nameTag" type="text" maxlength="64" placeholder="给这把武器一个名字" /></label>
          <label class="forge-check"><input :checked="draft.statTrak !== null" type="checkbox" @change="draft.statTrak = draft.statTrak === null ? 0 : null" /><span>启用 StatTrak 计数器</span></label>
          <label v-if="draft.statTrak !== null">当前计数<input v-model.number="draft.statTrak" type="number" min="0" max="999999" /></label>
          <details><summary>技术信息</summary><dl><div><dt>武器 Defindex</dt><dd>{{ draft.defindex }}</dd></div><div><dt>Paint Kit</dt><dd>{{ draft.paintKit }}</dd></div></dl></details>
        </div>
      </div>
      <footer class="forge-dialog__footer">
        <span v-if="submitError" class="dialog-progress" role="alert">{{ submitError }}</span>
        <span v-else-if="stickerLoading" class="dialog-progress"><LoaderCircle :size="15" />正在载入贴纸索引</span>
        <button class="secondary-button" type="button" @click="emit('close')">取消</button>
        <button data-testid="weapon-editor-save" class="primary-button" type="button" :disabled="props.disabled || submitted" :aria-disabled="props.disabled || submitted" :aria-busy="submitted" :title="props.disabled ? '完成部署后可编辑' : submitted ? '正在等待提交确认' : '提交当前编辑'" @click="save"><LoaderCircle v-if="submitted" :size="17" class="spin" /><Check v-else :size="17" />{{ props.disabled ? '完成部署后可编辑' : submitted ? '正在提交' : '完成编辑' }}</button>
      </footer>
    </section>
  </div>
</template>
