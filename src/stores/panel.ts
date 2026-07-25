import { ref } from 'vue'
import { defineStore } from 'pinia'

import type { AimValue, BotItem, Difficulty, NadesValue, PanelMode, PanelSnapshot } from '@/features/panel/types'
import { getPanelSnapshot, initializePanelDefaults, launchPanelCs2, setPanelAim, setPanelBotItem, setPanelDifficulty, setPanelDropKnives, setPanelMode, setPanelNades } from '@/services/tauri/panel'

export const usePanelStore = defineStore('panel', () => {
  const snapshot = ref<PanelSnapshot | null>(null)
  const loading = ref(false)
  const mutationKey = ref('')
  const lastError = ref('')
  const pendingRestart = ref(new Set<string>())
  let refreshInFlight: Promise<void> | null = null
  let activeRoot = ''

  function normalizeError(error: unknown) {
    return typeof error === 'string' ? error : error instanceof Error ? error.message : 'Panel 操作失败。'
  }

  function resetRoot(root: string) {
    if (activeRoot === root) return
    activeRoot = root
    snapshot.value = null
    pendingRestart.value = new Set()
    lastError.value = ''
  }

  async function refresh(root: string, silent = false) {
    resetRoot(root)
    if (!root || refreshInFlight) return refreshInFlight
    loading.value = !silent
    refreshInFlight = initializePanelDefaults(root)
      .then(() => getPanelSnapshot(root))
      .then(result => { snapshot.value = result })
      .catch(error => { if (!silent) lastError.value = normalizeError(error) })
      .finally(() => { loading.value = false; refreshInFlight = null })
    return refreshInFlight
  }

  async function mutate(key: string, operation: () => Promise<PanelSnapshot>) {
    const previous = snapshot.value
    mutationKey.value = key
    lastError.value = ''
    try {
      const result = await operation()
      snapshot.value = result
      if (result.cs2Running && !['mode', 'launch'].includes(key)) pendingRestart.value = new Set([...pendingRestart.value, key])
      return result
    } catch (error) {
      snapshot.value = previous
      lastError.value = normalizeError(error)
      throw error
    } finally {
      mutationKey.value = ''
    }
  }

  const setMode = (root: string, mode: PanelMode) => mutate('mode', () => setPanelMode(root, mode))
  const setDifficulty = (root: string, level: Difficulty) => mutate('difficulty', () => setPanelDifficulty(root, level))
  const setAim = (root: string, value: AimValue) => mutate('aim', () => setPanelAim(root, value))
  const setNades = (root: string, value: NadesValue) => mutate('nades', () => setPanelNades(root, value))
  const setBotItem = (root: string, item: BotItem, enabled: boolean) => mutate(item, () => setPanelBotItem(root, item, enabled))
  const setDropKnives = (root: string, bindKey: string, selected: number[]) => mutate('knives', () => setPanelDropKnives(root, bindKey, selected))
  async function launch(root: string, mode: PanelMode) {
    mutationKey.value = 'launch'
    lastError.value = ''
    try {
      return await launchPanelCs2(root, mode)
    } catch (error) {
      lastError.value = normalizeError(error)
      throw error
    } finally {
      mutationKey.value = ''
    }
  }

  return { snapshot, loading, mutationKey, lastError, pendingRestart, refresh, resetRoot, setMode, setDifficulty, setAim, setNades, setBotItem, setDropKnives, launch }
})
