import type { Cs2MapMetadata } from './coordinates'
import { mapLayer, scaleDemoCoordinate } from './coordinates'
import { pointsAtTick } from './frame-index'
import type { PositionPoint } from '@/types/demo'

interface DrawOptions { context: CanvasRenderingContext2D; size: number; ratio: number; metadata: Cs2MapMetadata; radar?: CanvasImageSource; points: readonly PositionPoint[]; layer: 'upper' | 'lower' }

export function drawBackground({ context, size, ratio, radar }: Pick<DrawOptions, 'context' | 'size' | 'ratio' | 'radar'>) {
  context.setTransform(ratio, 0, 0, ratio, 0, 0)
  context.clearRect(0, 0, size, size)
  context.fillStyle = '#0f172a'
  context.fillRect(0, 0, size, size)
  if (radar) context.drawImage(radar, 0, 0, size, size)
}

export function drawViewer(options: DrawOptions & { tick: number }) {
  drawBackground(options)
  for (const point of pointsAtTick(options.points, options.tick).filter(point => mapLayer(options.metadata, point.z) === options.layer)) {
    const position = scaleDemoCoordinate(options.metadata, options.size, point.x, point.y)
    if (position.x < 0 || position.y < 0 || position.x >= options.size || position.y >= options.size) continue
    if (point.participantRole === 'observer') {
      options.context.beginPath()
      options.context.arc(position.x, position.y, 7, 0, Math.PI * 2)
      options.context.fillStyle = '#0b1120'
      options.context.fill()
      options.context.strokeStyle = '#94a3b8'
      options.context.lineWidth = 3
      options.context.stroke()
      continue
    }
    if (point.teamNumber === 2) {
      options.context.fillStyle = '#0b1120'
      options.context.fillRect(position.x - 7, position.y - 7, 14, 14)
      options.context.fillStyle = '#f59e0b'
      options.context.fillRect(position.x - 5, position.y - 5, 10, 10)
      continue
    }
    options.context.beginPath()
    options.context.arc(position.x, position.y, 6, 0, Math.PI * 2)
    options.context.fillStyle = point.teamNumber === 3 ? '#38bdf8' : '#e2e8f0'
    options.context.fill()
    options.context.strokeStyle = '#0b1120'
    options.context.lineWidth = 2
    options.context.stroke()
  }
}
