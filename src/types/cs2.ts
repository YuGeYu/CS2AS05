export interface Cs2RootCandidate {
  path: string
  source: string
}

export type Cs2ProcessState = 'checking' | 'running' | 'stopped' | 'unknown'

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
