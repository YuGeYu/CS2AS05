<script setup lang="ts">
import { computed, ref } from 'vue'
import { ChevronRight, Info, Pencil, ShieldAlert } from 'lucide-vue-next'
import CatalogGrid from './CatalogGrid.vue'
import WeaponEditorDialog from './WeaponEditorDialog.vue'
import ForgeImage from './ForgeImage.vue'
import {
  agentCatalog,
  gloveCatalog,
  glovePaintCatalog,
  knifeCatalog,
  knifeSkinCatalog,
  musicCatalog,
  weaponCatalog,
  weaponSkinCatalog,
  type CatalogItem,
} from '@/features/skin-forge/data/catalog'
import { useSkinForgeStore } from '@/stores/skinForge'
import { createWeapon, type WeaponLoadout } from '@/types/skin-forge'
import { agentDisplay, gloveDisplay, knifeDisplay, musicDisplay, weaponDisplay } from '@/features/skin-forge/loadout-display'

const props = defineProps<{ category: 'weapons' | 'knives' | 'gloves' | 'agents' | 'music'; disabled: boolean }>()
const category = computed(() => props.category)
const forge = useSkinForgeStore()
const editingWeapon = ref<CatalogItem | null>(null)
const search = ref('')
const activeTeam = computed(() => forge.loadout.activeTeam)
const team = computed(() => forge.loadout[activeTeam.value])
const teamWeapons = computed(() => forge.loadout.weapons[activeTeam.value])
const selectedWeapon = computed(() => editingWeapon.value ? teamWeapons.value[`weapon_${editingWeapon.value.numericId}`] ?? createWeapon(editingWeapon.value.numericId) : null)
const selectedKnife = computed(() => knifeCatalog.find((item) => Number(item.meta?.index) === team.value.knife.index) ?? knifeCatalog.find(item => item.numericId === team.value.knife.defindex && team.value.knife.defindex > 42) ?? null)
const selectedGlove = computed(() => gloveCatalog.find((item) => item.numericId === team.value.gloves.defindex && team.value.gloves.index >= 0) ?? null)

function configuredWeapon(item: CatalogItem): WeaponLoadout | undefined { return teamWeapons.value[`weapon_${item.numericId}`] }
function weaponImage(item: CatalogItem): string {
  const current = configuredWeapon(item)
  return current && current.paintKit >= 0 ? weaponSkinCatalog(item.numericId).find((skin) => skin.numericId === current.paintKit)?.image ?? item.image : item.image
}
function chooseCustom(): boolean {
  if (props.disabled) return false
  if (forge.loadout.mode !== 'custom') forge.setMode('custom')
  else forge.markDirty()
  return true
}
function saveWeapon(weapon: WeaponLoadout, done: (result: { ok: true } | { ok: false; message: string }) => void) {
  if (!chooseCustom()) { done({ ok: false, message: '完成插件部署后可编辑' }); return }
  try {
    forge.updateWeapon(activeTeam.value, weapon)
    done({ ok: true })
    editingWeapon.value = null
  } catch (error) { done({ ok: false, message: String(error) }) }
}
function selectKnife(item: CatalogItem) {
  if (!chooseCustom()) return
  team.value.knife.index = Number(item.meta?.index ?? -1)
  team.value.knife.defindex = item.numericId
  const first = knifeSkinCatalog(item.numericId)[0]
  team.value.knife.paintKit = first?.numericId ?? -1
}
function clearKnife() { if (!chooseCustom()) return; team.value.knife = { index: -1, defindex: 42, paintKit: -1, wear: 0.01, seed: 0 } }
function selectKnifePaint(item: CatalogItem) { if (!chooseCustom()) return; team.value.knife.paintKit = item.numericId }
function selectGlove(item: CatalogItem) {
  if (!chooseCustom()) return
  team.value.gloves.index = Number(item.meta?.index ?? -1)
  team.value.gloves.defindex = item.numericId
  team.value.gloves.paintKit = glovePaintCatalog(item.numericId)[0]?.numericId ?? -1
}
function selectGlovePaint(item: CatalogItem) { if (!chooseCustom()) return; team.value.gloves.paintKit = item.numericId }
function clearGlove() { if (!chooseCustom()) return; team.value.gloves = { defindex: 0, index: -1, paintKit: -1, wear: 0.01, seed: 0 } }
function selectAgent(item: CatalogItem) { if (!chooseCustom()) return; team.value.agent = Number(item.meta?.index); team.value.agentPath = String(item.meta?.model ?? '') }
function clearAgent() { if (!chooseCustom()) return; team.value.agent = null; team.value.agentPath = '' }
function selectMusic(item: CatalogItem) { if (!chooseCustom()) return; forge.loadout.musicKit = item.numericId }
function clearMusic() { if (!chooseCustom()) return; forge.loadout.musicKit = null }
function setRange(target: { wear: number; seed: number }, field: 'wear' | 'seed', event: Event) { if (!chooseCustom()) return; target[field] = Number((event.target as HTMLInputElement).value) }
const musicSummary = computed(() => musicDisplay(forge.loadout).label)
</script>

<template>
  <div class="forge-workbench" :class="{ 'is-disabled': disabled }" :aria-disabled="disabled">
    <div v-if="disabled" class="forge-workbench-lock" role="status"><ShieldAlert :size="18" /><span><strong>完成插件部署后开放工坊</strong>装备目录可以浏览，选择与编辑暂时锁定。</span></div>
    <section class="forge-catalog-pane" :aria-labelledby="`forge-${category}-title`">
      <header class="forge-pane-heading">
        <div><p class="overline">{{ category }} / {{ activeTeam.toUpperCase() }}</p><h2 :id="`forge-${category}-title`">{{ category === 'weapons' ? '选择武器' : category === 'knives' ? '选择刀具' : category === 'gloves' ? '选择手套' : category === 'agents' ? '选择角色' : '选择音乐盒' }}</h2></div>
        <span v-if="category === 'weapons'">{{ Object.keys(teamWeapons).length }} / {{ weaponCatalog.length }} 已配置</span>
      </header>

      <div v-if="category === 'weapons'" class="weapon-catalog-grid" role="listbox" aria-label="武器目录">
        <button v-for="item in weaponCatalog" :key="item.id" class="weapon-card" type="button" role="option" :disabled="disabled" :aria-selected="Boolean(configuredWeapon(item))" @click="editingWeapon = item">
          <span class="weapon-card__media"><ForgeImage :src="weaponImage(item)" :alt="item.name" /></span>
          <span><strong>{{ item.name }}</strong><small>{{ item.secondary }}</small><small>{{ weaponDisplay(forge.loadout, activeTeam, item).label }}</small></span>
          <Pencil :size="16" aria-label="编辑" />
        </button>
      </div>
      <div v-else-if="category === 'knives'" class="forge-split-picker">
        <CatalogGrid v-model:query="search" :items="knifeCatalog" :selected-id="selectedKnife?.id" :disabled="disabled" label="刀具" @select="selectKnife" />
        <section class="forge-subpicker"><h3>选择刀面</h3><button class="secondary-button" type="button" :disabled="disabled" @click="clearKnife">取消选择刀具</button><CatalogGrid v-if="selectedKnife" :items="knifeSkinCatalog(selectedKnife.numericId)" :selected-id="team.knife.paintKit < 0 ? null : `${selectedKnife.numericId}_${team.knife.paintKit}`" :disabled="disabled" compact label="刀面" @select="selectKnifePaint" /><div v-else class="forge-empty"><ChevronRight :size="20" /><span>未选择刀具</span></div></section>
      </div>
      <div v-else-if="category === 'gloves'" class="forge-split-picker">
        <CatalogGrid :items="gloveCatalog" :selected-id="selectedGlove?.id" :disabled="disabled" label="手套" @select="selectGlove" />
        <section class="forge-subpicker"><h3>选择涂装</h3><button class="secondary-button" type="button" :disabled="disabled" @click="clearGlove">取消选择手套</button><CatalogGrid v-if="selectedGlove" :items="glovePaintCatalog(selectedGlove.numericId)" :selected-id="team.gloves.paintKit < 0 ? null : `${selectedGlove.numericId}_${team.gloves.paintKit}`" :disabled="disabled" compact label="手套涂装" @select="selectGlovePaint" /><div v-else class="forge-empty"><ChevronRight :size="20" /><span>未选择手套</span></div></section>
      </div>
      <div v-else-if="category === 'agents'"><button class="secondary-button" type="button" :disabled="disabled" @click="clearAgent">取消选择角色</button><CatalogGrid :items="agentCatalog[activeTeam]" :selected-id="agentCatalog[activeTeam].find(item => Number(item.meta?.index) === team.agent)?.id" :disabled="disabled" label="角色" @select="selectAgent" /></div>
      <div v-else><button class="secondary-button" type="button" :disabled="disabled" @click="clearMusic">取消选择音乐盒</button><CatalogGrid :items="musicCatalog" :selected-id="forge.loadout.musicKit === null ? null : `music_${forge.loadout.musicKit}`" :disabled="disabled" label="音乐盒" @select="selectMusic" /></div>
    </section>

    <aside class="forge-preview-pane" aria-label="当前装备摘要">
      <header><div><p class="overline">CURRENT LOADOUT</p><h2>当前装备</h2></div><span class="forge-team-mark" :data-team="activeTeam">{{ activeTeam.toUpperCase() }}</span></header>
      <div class="forge-preview-image">
        <ForgeImage :src="selectedKnife?.image || selectedGlove?.image" alt="当前装备预览" loading="eager" />
      </div>
      <dl class="forge-summary">
        <div><dt>武器</dt><dd>{{ Object.keys(teamWeapons).length }} 把已配置</dd></div>
        <div><dt>刀具</dt><dd>{{ knifeDisplay(forge.loadout, activeTeam).label }}</dd></div>
        <div><dt>手套</dt><dd>{{ gloveDisplay(forge.loadout, activeTeam).label }}</dd></div>
        <div><dt>角色</dt><dd>{{ agentDisplay(forge.loadout, activeTeam).label }}</dd></div>
        <div><dt>音乐盒</dt><dd>{{ musicSummary }}</dd></div>
      </dl>
      <div v-if="category === 'knives' && selectedKnife" class="forge-parameter-panel forge-preview-controls">
        <label>磨损 <output>{{ team.knife.wear.toFixed(4) }}</output><input :value="team.knife.wear" :disabled="disabled" type="range" min="0" max="1" step="0.0001" @input="setRange(team.knife, 'wear', $event)" /></label>
        <label>图案模板 <output>{{ team.knife.seed }}</output><input :value="team.knife.seed" :disabled="disabled" type="range" min="0" max="1000" step="1" @input="setRange(team.knife, 'seed', $event)" /></label>
      </div>
      <div v-if="category === 'gloves' && selectedGlove" class="forge-parameter-panel forge-preview-controls">
        <label>磨损 <output>{{ team.gloves.wear.toFixed(4) }}</output><input :value="team.gloves.wear" :disabled="disabled" type="range" min="0" max="1" step="0.0001" @input="setRange(team.gloves, 'wear', $event)" /></label>
        <label>图案模板 <output>{{ team.gloves.seed }}</output><input :value="team.gloves.seed" :disabled="disabled" type="range" min="0" max="1000" step="1" @input="setRange(team.gloves, 'seed', $event)" /></label>
      </div>
      <div class="shared-notice"><Info :size="16" /><span>武器皮肤按 CT/T 独立；贴纸、挂件、命名和计数器按武器共用。</span></div>
      <div v-if="forge.loadout.mode === 'random'" class="forge-random-note"><ShieldAlert :size="16" /><span>随机模式已开启，插件会在重生时从 v1.8.2 目录挑选外观。</span></div>
    </aside>

    <WeaponEditorDialog v-if="editingWeapon && selectedWeapon" :weapon="selectedWeapon" :weapon-name="editingWeapon.name" :team="activeTeam" :disabled="disabled" @close="editingWeapon = null" @save="saveWeapon" />
  </div>
</template>
