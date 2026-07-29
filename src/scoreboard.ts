import { createApp } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import ScoreboardApp from '@/ScoreboardApp.vue'
import '@/styles/scoreboard.css'

function reportBootError(value: unknown) {
  const message = value instanceof Error ? value.message : String(value)
  void invoke('report_scoreboard_boot_error', { message }).catch(() => undefined)
}

window.addEventListener('error', event => reportBootError(event.error ?? event.message))
window.addEventListener('unhandledrejection', event => reportBootError(event.reason))

try {
  createApp(ScoreboardApp).mount('#scoreboard-app')
  document.body.dataset.scoreboardMounted = 'true'
} catch (error) {
  reportBootError(error)
}
