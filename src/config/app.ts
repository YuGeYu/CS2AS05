export const appConfig = {
  appVersion: __APP_VERSION__,
  channel: import.meta.env.VITE_APP_CHANNEL ?? 'prod',
  updateFeedUrl: import.meta.env.VITE_UPDATE_FEED_URL
    ?? 'https://cs2as.600318.xyz/api/software-updates/cs2-bot-improver',
  updaterEnabled: import.meta.env.VITE_ENABLE_UPDATER !== 'false',
  projectId: import.meta.env.VITE_DEFAULT_PROJECT_ID ?? 'cs2-bot-improver',
} as const
