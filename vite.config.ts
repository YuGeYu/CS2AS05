import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

import packageJson from './package.json' with { type: 'json' }

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    rolldownOptions: {
      input: {
        main: fileURLToPath(new URL('./index.html', import.meta.url)),
        scoreboard: fileURLToPath(new URL('./scoreboard.html', import.meta.url)),
      },
    },
  },
  server: {
    watch: {
      ignored: ['**/target/**'],
    },
  },
  define: {
    __APP_VERSION__: JSON.stringify(packageJson.version),
  },
  test: {
    exclude: ['**/node_modules/**', '**/dist/**', '**/workspace/**', '**/third_party/**'],
  },
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
})
