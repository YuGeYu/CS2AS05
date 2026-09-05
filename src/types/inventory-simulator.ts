export interface InventorySimulatorStatus {
  selectedRoot: string
  csgoRoot: string
  cs2Running: boolean
  counterStrikeSharpInstalled: boolean
  counterStrikeSharpVersion: string | null
  legacyPlayerSkinModPresent: boolean
  legacyAppDataPresent: boolean
  inventorySimulatorPresent: boolean
  resourceVersion: string
  deployedVersion: string | null
  upstreamTag: string
  upstreamCommit: string
  missingFiles: string[]
  hashMismatches: string[]
  gamedataPresent: boolean
  coreGuidelineCompatible: boolean | null
  serviceReachable: boolean | null
  serviceCheckedAt: string | null
  ready: boolean
  blockedCode: string | null
  blockedMessage: string | null
}

export interface InventorySimulatorInstallResult {
  status: InventorySimulatorStatus
  removedLegacyPlugin: boolean
  removedLegacyAppData: boolean
  deployedFiles: string[]
}

export interface InventorySimulatorRemoveResult {
  status: InventorySimulatorStatus
  removedPaths: string[]
}

export interface InventorySimulatorServiceStatus {
  reachable: boolean
  checkedAt: string
  message: string
}
