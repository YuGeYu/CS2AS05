export const appConfig = {
  appVersion: __APP_VERSION__,
  channel: import.meta.env.VITE_APP_CHANNEL ?? '',
  updateFeedUrl: import.meta.env.VITE_UPDATE_FEED_URL ?? '',
  updaterEnabled: import.meta.env.VITE_ENABLE_UPDATER === 'true',
  projectId: import.meta.env.VITE_DEFAULT_PROJECT_ID ?? '',
} as const
