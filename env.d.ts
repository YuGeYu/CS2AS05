/// <reference types="vite/client" />

declare const __APP_VERSION__: string

interface ImportMetaEnv {
  readonly VITE_APP_NAME: string
  readonly VITE_APP_BRAND_LABEL: string
  readonly VITE_APP_CHANNEL: string
  readonly VITE_API_BASE_URL: string
  readonly VITE_UPDATE_FEED_URL: string
  readonly VITE_ENABLE_UPDATER: string
  readonly VITE_WORKSPACE_NAME: string
  readonly VITE_DEFAULT_PROJECT_ID: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
