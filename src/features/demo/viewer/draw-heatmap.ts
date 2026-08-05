import type { Cs2MapMetadata } from './coordinates'
import { mapLayer, scaleDemoCoordinate } from './coordinates'
import { drawBackground } from './draw-viewer'
import type { PositionPoint } from '@/types/demo'

export function drawHeatmap(context: CanvasRenderingContext2D, size: number, ratio: number, metadata: Cs2MapMetadata, radar: CanvasImageSource | undefined, points: readonly PositionPoint[], layer: 'upper' | 'lower') {
  drawBackground({ context, size, ratio, radar })
  for (const point of points.filter(point => mapLayer(metadata, point.z) === layer)) {
    const position = scaleDemoCoordinate(metadata, size, point.x, point.y)
    if (position.x < 0 || position.y < 0 || position.x >= size || position.y >= size) continue
    const gradient = context.createRadialGradient(position.x, position.y, 0, position.x, position.y, 18)
    gradient.addColorStop(0, 'rgba(220,38,38,.2)')
    gradient.addColorStop(1, 'rgba(217,119,6,0)')
    context.fillStyle = gradient
    context.fillRect(position.x - 18, position.y - 18, 36, 36)
  }
}
