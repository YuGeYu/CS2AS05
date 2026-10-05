export interface DemoTracerStatus {
  resourceVersion: string
  playbackVersion: string
  guiPath: string | null
  playbackPackagePath: string | null
  guiResourceReady: boolean
  playbackResourceReady: boolean
  playbackInstalled: boolean
  installedFileCount: number
  missingFiles: string[]
  hashMismatches: string[]
  counterStrikeSharpReady: boolean
  metamodReady: boolean
  cs2Running: boolean
  selectedRoot: string | null
  ready: boolean
  blockedCode: string | null
  blockedMessage: string | null
}

export interface DemoTracerInstallResult {
  status: DemoTracerStatus
  installedFiles: string[]
  backupPath: string | null
}

export interface DemoTracerOperationResult {
  success: boolean
  message: string
}
