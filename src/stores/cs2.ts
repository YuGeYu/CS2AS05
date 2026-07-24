import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

import {
  checkCs2Process,
  discoverCs2Roots,
  getDiagnosticsPayload,
  inspectCs2Root,
  installBotPackage,
  openUpstreamPanel,
  uninstallBotPackage,
} from '@/services/tauri/cs2'
import type { Cs2EnvironmentStatus, Cs2RootCandidate, DiagnosticsPayload } from '@/types/cs2'
import type { Cs2ProcessState } from '@/types/cs2'

const ROOT_STORAGE_KEY = 'cs2-bot-improver.selected-root.v1'

function normalizeError(error: unknown) {
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  return '操作失败，请展开诊断信息后重试。'
}

export const useCs2Store = defineStore('cs2', () => {
  const candidates = ref<Cs2RootCandidate[]>([])
  const selectedRoot = ref(getStorage()?.getItem(ROOT_STORAGE_KEY) ?? '')
  const environment = ref<Cs2EnvironmentStatus | null>(null)
  const cs2ProcessState = ref<Cs2ProcessState>('checking')
  const cs2Running = computed(() => cs2ProcessState.value === 'running')
  const diagnostics = ref<DiagnosticsPayload | null>(null)
  const message = ref('')
  const busy = ref(false)
  let processCheckInFlight: Promise<void> | null = null

  async function selectRoot(rootPath: string) {
    const status = await inspectCs2Root(rootPath)
    selectedRoot.value = status.rootPath
    environment.value = status
    getStorage()?.setItem(ROOT_STORAGE_KEY, status.rootPath)
    candidates.value = dedupe([{ path: status.rootPath, source: '当前选择' }, ...candidates.value])
  }

  async function scanRoots() {
    busy.value = true
    try {
      candidates.value = dedupe(await discoverCs2Roots())
      if (!selectedRoot.value && candidates.value[0]) await selectRoot(candidates.value[0].path)
      message.value = candidates.value.length ? '已完成目录扫描。' : '没有自动找到 CS2 目录，请手动选择。'
    } catch (error) {
      message.value = normalizeError(error)
    } finally {
      busy.value = false
    }
  }

  function refreshProcessStatus() {
    if (processCheckInFlight) return processCheckInFlight
    processCheckInFlight = checkCs2Process()
      .then((running) => { cs2ProcessState.value = running ? 'running' : 'stopped' })
      .catch(() => { cs2ProcessState.value = 'unknown' })
      .finally(() => { processCheckInFlight = null })
    return processCheckInFlight
  }

  async function refresh() {
    await refreshProcessStatus()
    try {
      if (selectedRoot.value) environment.value = await inspectCs2Root(selectedRoot.value)
    } catch (error) {
      message.value = normalizeError(error)
    }
  }

  async function install() {
    if (!selectedRoot.value) throw new Error('请先选择 CS2 游戏目录。')
    busy.value = true
    try {
      const result = await installBotPackage(selectedRoot.value)
      message.value = result.message
      await refresh()
    } catch (error) {
      message.value = normalizeError(error)
      throw error
    } finally {
      busy.value = false
    }
  }

  async function openPanel() {
    busy.value = true
    try {
      const result = await openUpstreamPanel()
      message.value = result.message
    } catch (error) {
      message.value = normalizeError(error)
      throw error
    } finally {
      busy.value = false
    }
  }

  async function uninstall() {
    if (!selectedRoot.value) throw new Error('请先选择 CS2 游戏目录。')
    busy.value = true
    try {
      const result = await uninstallBotPackage(selectedRoot.value)
      message.value = result.message
      await refresh()
    } catch (error) {
      message.value = normalizeError(error)
      throw error
    } finally {
      busy.value = false
    }
  }

  async function refreshDiagnostics() {
    busy.value = true
    try {
      diagnostics.value = await getDiagnosticsPayload(selectedRoot.value || undefined)
    } catch (error) {
      message.value = normalizeError(error)
    } finally {
      busy.value = false
    }
  }

  return { candidates, selectedRoot, environment, cs2ProcessState, cs2Running, diagnostics, message, busy, selectRoot, scanRoots, refreshProcessStatus, refresh, install, openPanel, uninstall, refreshDiagnostics }
})

function dedupe(candidates: Cs2RootCandidate[]) {
  return candidates.filter((candidate, index, list) => list.findIndex((item) => item.path === candidate.path) === index)
}

function getStorage() {
  try {
    return typeof window === 'undefined' ? undefined : window.localStorage
  } catch {
    return undefined
  }
}
