export interface Cs2RootCandidate {
  path: string
  source: string
}

export interface Cs2SuggestedRoot extends Cs2RootCandidate {
  confidence: 'verified' | 'likely'
  evidence: string[]
}

export interface Cs2RootScanEvent {
  kind: 'progress' | 'candidate'
  elapsedMs: number
  checkedLocations: number
  currentLocation?: string
  candidate?: Cs2SuggestedRoot
}

export interface Cs2RootScanSummary {
  candidates: Cs2SuggestedRoot[]
  elapsedMs: number
  checkedLocations: number
  stopReason: 'threeFound' | 'timeout' | 'userStopped'
  warnings: string[]
}

export type Cs2ProcessState = 'checking' | 'running' | 'stopped' | 'unknown'
export interface Cs2ProcessInfo { pid: number; exeName: string; exePath: string | null; parentPid: number | null; startTime: number | null }
export interface Cs2ProcessSnapshot { observedAt: number; processes: Cs2ProcessInfo[]; confidence: 'high' | 'low' | 'unknown'; sampleCount: number }

export interface ToastMessage {
  tone: 'ready' | 'warn' | 'danger' | 'info'
  title: string
  message: string
  durationMs?: number
}

export interface Cs2EnvironmentStatus {
  rootPath: string
  gameDirExists: boolean
  csgoDirExists: boolean
  metamodExists: boolean
  counterstrikeSharpExists: boolean
  gameinfoExists: boolean
  backupOnlineGameinfoExists: boolean
  backupWithbotsGameinfoExists: boolean
  baseEnvironmentReady: boolean
}

export interface OperationResult {
  success: boolean
  message: string
}

export interface DiagnosticsPayload {
  summary: string
  fullLog: string
  logPath: string
}
