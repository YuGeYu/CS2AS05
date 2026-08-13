import { computed, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { dispatchToast } from '@/services/toast'
import {
  skinForgeCheckPlugin,
  skinForgeDeploy,
  skinForgeLoad,
  skinForgeReset,
  skinForgeSave,
  type PluginCheckResult,
  type SaveResult,
} from '@/services/tauri/skinForge'
import { DEFAULT_LOADOUT, synchronizeSharedDetails, type ForgeMode, type Loadout, type Team, type WeaponLoadout } from '@/types/skin-forge'
import { fromPlayerSkinModFile, toPlayerSkinModFile } from '@/features/skin-forge/player-skin-mod-adapter'
import { useCs2Store } from '@/stores/cs2'

export const useSkinForgeStore = defineStore('skinForge', () => {
  const cs2 = useCs2Store()
  const loadout = ref<Loadout>(structuredClone(DEFAULT_LOADOUT))
  const plugin = ref<PluginCheckResult | null>(null)
  const initializing = ref(false)
  const catalogLoading = ref(false)
  const loadoutLoading = ref(false)
  const saving = ref(false)
  const deploying = ref(false)
  const dirty = ref(false)
  const error = ref<string | null>(null)
  const lastSave = ref<SaveResult | null>(null)
  const mode = computed(() => loadout.value.mode)
  const pluginReady = computed(() => plugin.value?.allPresent === true && plugin.value.hashMismatches.length === 0)
  const busy = computed(() => initializing.value || catalogLoading.value || loadoutLoading.value || saving.value || deploying.value)
  let loadGeneration = 0

  function requireRoot() {
    if (!cs2.selectedRoot) throw new Error('请先在安装与诊断中选择 CS2 游戏目录。')
    return cs2.selectedRoot
  }
  async function load() {
    const root = cs2.selectedRoot
    const generation = ++loadGeneration
    plugin.value = null
    if (!root) return
    initializing.value = true
    loadoutLoading.value = true
    error.value = null
    try {
      const nextLoadout = fromPlayerSkinModFile(await skinForgeLoad(root))
      const nextPlugin = await skinForgeCheckPlugin(root).catch(() => null)
      if (generation !== loadGeneration || root !== cs2.selectedRoot) return
      loadout.value = nextLoadout
      plugin.value = nextPlugin
      dirty.value = false
    } catch (reason) {
      if (generation === loadGeneration && root === cs2.selectedRoot) error.value = String(reason)
    } finally {
      if (generation === loadGeneration && root === cs2.selectedRoot) {
        initializing.value = false
        loadoutLoading.value = false
      }
    }
  }
  async function save() {
    saving.value = true
    error.value = null
    try {
      const root = requireRoot()
      plugin.value = null
      plugin.value = await skinForgeCheckPlugin(root)
      if (!pluginReady.value) throw new Error('[PLAYER_SKIN_MOD_REQUIRED] 请先完整部署 PlayerSkinMod，再应用装备。')
      lastSave.value = await skinForgeSave(root, toPlayerSkinModFile(loadout.value))
      dirty.value = false
      dispatchToast({ tone: 'ready', title: '装备已写入游戏', message: `${lastSave.value.loadoutPath}\nSHA-256 ${lastSave.value.sha256.slice(0, 16)}…` })
    } catch (reason) {
      error.value = String(reason)
      dispatchToast({ tone: 'danger', title: '保存失败', message: error.value })
      throw reason
    } finally { saving.value = false }
  }
  async function reset() {
    saving.value = true
    error.value = null
    try {
      const root = requireRoot()
      plugin.value = null
      plugin.value = await skinForgeCheckPlugin(root)
      if (!pluginReady.value) throw new Error('[PLAYER_SKIN_MOD_REQUIRED] 请先完整部署 PlayerSkinMod，再重置装备。')
      await skinForgeReset(root)
      lastSave.value = null
      await load()
      dirty.value = false
    } catch (reason) {
      error.value = String(reason)
      dispatchToast({ tone: 'danger', title: '重置失败', message: error.value })
      throw reason
    } finally { saving.value = false }
  }
  async function check() {
    error.value = null
    const root = requireRoot()
    plugin.value = null
    try {
      const result = await skinForgeCheckPlugin(root)
      if (root === cs2.selectedRoot) plugin.value = result
    }
    catch (reason) { error.value = String(reason); throw reason }
  }
  async function deploy() {
    deploying.value = true
    error.value = null
    try {
      const root = requireRoot()
      const result = await skinForgeDeploy(root)
      plugin.value = null
      const verified = await skinForgeCheckPlugin(root)
      if (root !== cs2.selectedRoot) return
      plugin.value = verified
      if (!pluginReady.value) throw new Error('[PLAYER_SKIN_MOD_REQUIRED] 插件部署后的完整性复检未通过，请查看诊断信息。')
      dispatchToast({ tone: 'ready', title: '插件部署完成', message: `${result.targetDir}\nPlayerSkinMod ${result.pluginVersion}` })
    } catch (reason) {
      error.value = String(reason)
      dispatchToast({ tone: 'danger', title: '部署被阻止', message: error.value })
      throw reason
    } finally { deploying.value = false }
  }
  function markDirty() { dirty.value = true }
  function setMode(value: ForgeMode) { loadout.value.mode = value; markDirty() }
  function setTeam(value: Team) { loadout.value.activeTeam = value }
  function updateWeapon(team: Team, weapon: WeaponLoadout) {
    const key = `weapon_${weapon.defindex}`
    loadout.value.weapons[team][key] = structuredClone(weapon)
    const other: Team = team === 'ct' ? 't' : 'ct'
    const counterpart = loadout.value.weapons[other][key]
    if (counterpart) {
      counterpart.stickers = structuredClone(weapon.stickers)
      counterpart.keychain = structuredClone(weapon.keychain)
      counterpart.nameTag = weapon.nameTag
      counterpart.statTrak = weapon.statTrak
    }
    synchronizeSharedDetails(loadout.value, team)
    loadout.value.mode = 'custom'
    markDirty()
  }
  watch(() => cs2.selectedRoot, () => {
    loadGeneration += 1
    plugin.value = null
    lastSave.value = null
    initializing.value = false
    loadoutLoading.value = false
    void load()
  })
  return {
    loadout, plugin, initializing, catalogLoading, loadoutLoading, saving, deploying, busy, dirty,
    error, lastSave, mode, pluginReady, load, save, reset, check, deploy, markDirty, setMode, setTeam, updateWeapon,
  }
})
