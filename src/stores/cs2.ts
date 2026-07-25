import { computed, ref } from 'vue'
import { defineStore } from 'pinia'

import {
  checkCs2Process,
  discoverCs2Roots,
  getDiagnosticsPayload,
  guessCs2Roots,
  inspectCs2Root,
  installBotPackage,
  openUpstreamPanel,
  uninstallBotPackage,
  stopGuessCs2Roots,
} from '@/services/tauri/cs2'
import type { Cs2EnvironmentStatus, Cs2RootCandidate, Cs2RootScanEvent, Cs2RootScanSummary, Cs2SuggestedRoot, DiagnosticsPayload, ToastMessage } from '@/types/cs2'
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
  const message = ref<ToastMessage | null>(null)
  const busy = ref(false)
  const rootScan = ref<{ running: boolean; elapsedMs: number; checkedLocations: number; currentLocation: string; candidates: Cs2SuggestedRoot[]; summary: Cs2RootScanSummary | null }>({ running: false, elapsedMs: 0, checkedLocations: 0, currentLocation: '', candidates: [], summary: null })
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
      message.value = candidates.value.length
        ? { tone: 'ready', title: '扫描完成', message: '已完成目录扫描。' }
        : { tone: 'warn', title: '未找到目录', message: '没有自动找到 CS2 目录，请手动选择。' }
    } catch (error) {
      message.value = failure(error)
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
      message.value = failure(error)
    }
  }

  async function install() {
    if (!selectedRoot.value) throw new Error('请先选择 CS2 游戏目录。')
    busy.value = true
    try {
      const result = await installBotPackage(selectedRoot.value)
      message.value = { tone: 'ready', title: '安装完成', message: result.message }
      await refresh()
    } catch (error) {
      message.value = failure(error)
      throw error
    } finally {
      busy.value = false
    }
  }

  async function openPanel() {
    busy.value = true
    try {
      const result = await openUpstreamPanel()
      message.value = { tone: 'ready', title: '原版 Panel 已启动', message: result.message }
    } catch (error) {
      message.value = failure(error)
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
      message.value = { tone: 'ready', title: '卸载完成', message: result.message }
      await refresh()
    } catch (error) {
      message.value = failure(error)
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
      message.value = failure(error)
    } finally {
      busy.value = false
    }
  }

  async function scanSuggestedRoots(onEvent?: (event: Cs2RootScanEvent) => void) {
    if (rootScan.value.running) return rootScan.value.summary
    rootScan.value = { running: true, elapsedMs: 0, checkedLocations: 0, currentLocation: '', candidates: [], summary: null }
    try {
      const summary = await guessCs2Roots((event) => {
        rootScan.value.elapsedMs = event.elapsedMs
        rootScan.value.checkedLocations = event.checkedLocations
        rootScan.value.currentLocation = event.currentLocation ?? ''
        if (event.kind === 'candidate' && event.candidate) {
          rootScan.value.candidates = mergeSuggested(rootScan.value.candidates, event.candidate)
        }
        onEvent?.(event)
      })
      rootScan.value.summary = summary
      rootScan.value.candidates = summary.candidates
      candidates.value = dedupe([
        ...summary.candidates.map(candidate => ({ path: candidate.path, source: candidate.source })),
        ...candidates.value,
      ])
      return summary
    } finally {
      rootScan.value.running = false
    }
  }

  async function stopSuggestedRoots() {
    if (!rootScan.value.running) return false
    return stopGuessCs2Roots()
  }

  return { candidates, selectedRoot, environment, cs2ProcessState, cs2Running, diagnostics, message, busy, rootScan, selectRoot, scanRoots, refreshProcessStatus, refresh, install, openPanel, uninstall, refreshDiagnostics, scanSuggestedRoots, stopSuggestedRoots }
})

function dedupe(candidates: Cs2RootCandidate[]) {
  return candidates.filter((candidate, index, list) => list.findIndex((item) => canonicalPath(item.path) === canonicalPath(candidate.path)) === index)
}

function mergeSuggested(candidates: Cs2SuggestedRoot[], candidate: Cs2SuggestedRoot) {
  const existing = candidates.findIndex((item) => canonicalPath(item.path) === canonicalPath(candidate.path))
  if (existing < 0) return [...candidates, candidate].slice(0, 3)
  const copy = [...candidates]
  const previous = copy[existing]
  if (!previous) return copy
  copy[existing] = { ...previous, ...candidate, evidence: [...new Set([...previous.evidence, ...candidate.evidence])] }
  return copy
}

function canonicalPath(path: string) { return path.replaceAll('/', '\\').toLowerCase() }

function getStorage() {
  try {
    return typeof window === 'undefined' ? undefined : window.localStorage
  } catch {
    return undefined
  }
}

function failure(error: unknown): ToastMessage {
  return { tone: 'danger', title: '操作失败', message: normalizeError(error) }
}
