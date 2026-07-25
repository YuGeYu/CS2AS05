import { computed, nextTick, onBeforeUnmount, ref, watch, type Ref } from 'vue'

import type { PanelMode } from '@/features/panel/types'
import { useCs2Store } from '@/stores/cs2'
import { usePanelStore } from '@/stores/panel'

export const LAUNCH_EXPERIENCE_DURATION_MS = 30_000
export const LAUNCH_PROCESS_PROBE_MS = 650

type LaunchPhase = 'idle' | 'requesting' | 'waitingForProcess'

export function useCs2LaunchExperience(root: Ref<string>) {
  const cs2 = useCs2Store()
  const panel = usePanelStore()
  const phase = ref<LaunchPhase>('idle')
  const elapsedMs = ref(0)
  const mode = ref<PanelMode>('bots')
  let startedAt = 0
  let generation = 0
  let elapsedTimer: ReturnType<typeof setInterval> | undefined
  let probeTimer: ReturnType<typeof setTimeout> | undefined
  let durationTimer: ReturnType<typeof setTimeout> | undefined
  let listenerTimer: ReturnType<typeof setTimeout> | undefined

  const active = computed(() => phase.value !== 'idle')

  function clearResources() {
    if (elapsedTimer) clearInterval(elapsedTimer)
    if (probeTimer) clearTimeout(probeTimer)
    if (durationTimer) clearTimeout(durationTimer)
    if (listenerTimer) clearTimeout(listenerTimer)
    elapsedTimer = probeTimer = durationTimer = listenerTimer = undefined
    document.removeEventListener('click', dismissFromDocument, true)
  }

  function dismiss() {
    generation += 1
    clearResources()
    phase.value = 'idle'
    elapsedMs.value = 0
  }

  function dismissFromDocument() {
    dismiss()
  }

  async function probe(currentGeneration: number) {
    if (currentGeneration !== generation || !active.value) return
    await cs2.refreshProcessStatus()
    if (currentGeneration !== generation || !active.value) return
    if (cs2.cs2Running) {
      dismiss()
      return
    }
    probeTimer = setTimeout(() => void probe(currentGeneration), LAUNCH_PROCESS_PROBE_MS)
  }

  async function start(nextMode: PanelMode) {
    dismiss()
    const currentGeneration = generation
    mode.value = nextMode
    phase.value = 'requesting'
    startedAt = Date.now()
    elapsedMs.value = 0
    elapsedTimer = setInterval(() => {
      elapsedMs.value = Math.min(Date.now() - startedAt, LAUNCH_EXPERIENCE_DURATION_MS)
    }, 100)
    durationTimer = setTimeout(dismiss, LAUNCH_EXPERIENCE_DURATION_MS)
    await nextTick()
    listenerTimer = setTimeout(() => document.addEventListener('click', dismissFromDocument, true), 0)

    try {
      const result = await panel.launch(root.value, nextMode)
      if (currentGeneration !== generation || !active.value) return
      if (result.pluginAction === 'installed') {
        cs2.message = { tone: 'ready', title: 'BOT 插件已更新', message: `已自动安装 ${result.pluginVersion}，正在启动 CS2。` }
      }
      phase.value = 'waitingForProcess'
      void probe(currentGeneration)
    } catch {
      if (currentGeneration === generation) dismiss()
    }
  }

  watch(root, dismiss)
  onBeforeUnmount(dismiss)

  return { active, phase, elapsedMs, mode, start, dismiss }
}
