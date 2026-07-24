import '@/styles/main.css'

import { createApp } from 'vue'
import { createPinia } from 'pinia'

import App from '@/App.vue'
import { initializeTheme } from '@/composables/useThemePreference'

export function mountApp() {
  initializeTheme()

  window.addEventListener('contextmenu', (event) => {
    event.preventDefault()
  })

  const app = createApp(App)

  app.use(createPinia())
  app.mount('#app')
}
