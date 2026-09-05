export interface MapRotationDefault {
  rootPath: string
  configPath: string
  enabled: boolean
  source: 'existing' | 'default' | 'fallback'
  writable: boolean
  warning: string | null
  updatedAt: string | null
  configSha256: string | null
  readBackEnabled: boolean
  observedAt: string
  loadSemantics: 'next-plugin-load'
}
