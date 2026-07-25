import { onBeforeUnmount, onMounted } from 'vue'

export const CS2_PROCESS_POLL_INTERVAL_MS = 10_000

export function useCs2ProcessPolling(refresh: () => Promise<unknown>) {
  let timer: ReturnType<typeof setInterval> | undefined

  const refreshWhenVisible = () => {
    if (document.visibilityState === 'visible') void refresh()
  }

  onMounted(() => {
    void refresh()
    timer = setInterval(() => void refresh(), CS2_PROCESS_POLL_INTERVAL_MS)
    window.addEventListener('focus', refreshWhenVisible)
    document.addEventListener('visibilitychange', refreshWhenVisible)
  })

  onBeforeUnmount(() => {
    if (timer) clearInterval(timer)
    window.removeEventListener('focus', refreshWhenVisible)
    document.removeEventListener('visibilitychange', refreshWhenVisible)
  })
}
