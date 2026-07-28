export interface StageSize {
  width: number
  height: number
  dpr: number
}

export interface StageInteraction {
  type: 'pointer-down' | 'pointer-move' | 'pointer-up'
  x: number
  y: number
}

export interface ThreeStageController {
  frame(timeSeconds: number, deltaSeconds: number): void
  resize(size: StageSize): void
  dispose(): void
  setActive?(active: boolean): void
  setLowPerformance?(enabled: boolean): void
  interact?(interaction: StageInteraction): void
}

export interface ThreeStageError {
  stage: 'factory' | 'frame'
  code: 'THREE_STAGE_FACTORY_FAILED' | 'THREE_STAGE_FRAME_FAILED'
  message: string
}

export type ThreeStageFactory = (canvas: HTMLCanvasElement, size: StageSize) => Promise<ThreeStageController>
