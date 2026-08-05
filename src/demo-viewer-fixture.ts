import '@/styles/main.css'
import { createApp, h } from 'vue'
import { mockIPC } from '@tauri-apps/api/mocks'
import MatchViewer2D from '@/features/demo/components/MatchViewer2D.vue'
import type { PositionPoint } from '@/types/demo'

if (!import.meta.env.DEV) throw new Error('Demo viewer fixture is development-only')

const points: PositionPoint[] = Array.from({ length: 180 }, (_, index) => ({
  tick: 1000 + Math.floor(index / 10) * 8,
  stableKey: `bot:${index % 10}`,
  name: `BOT ${index % 10}`,
  x: -2476 + 700 + (index % 10) * 230,
  y: 3239 - 650 - Math.floor(index / 10) * 110,
  z: 0,
  yaw: index * 12,
  health: 100,
  armor: 100,
  teamNumber: index % 2 ? 2 : 3,
  alive: true,
  weapon: 'ak47',
}))

mockIPC((command) => {
  if (command === 'get_round_positions') return { demoFileId: 1, roundNumber: 1, samplingHz: 8, points }
  if (command === 'ensure_spatial_analysis') return 1
  throw new Error(`Unexpected fixture command: ${command}`)
})

const rounds = [{ roundNumber: 1, startTick: 1000, freezeEndTick: 1016, endTick: 1144, officialEndTick: 1152, winnerSide: 'CT', reason: 'target_bombed', eventCount: 24 }]
const mode = new URLSearchParams(location.search).get('mode') === 'heatmap' ? 'heatmap' : 'viewer'

createApp({ render: () => h('section', { class: 'report-section fixture-shell' }, [
  h('div', { class: 'section-heading' }, [h('div', [h('h1', mode === 'viewer' ? '二维地图回放' : '位置热力图')])]),
  h(MatchViewer2D, { demoId: 1, mapName: 'de_dust2', rounds, mode }),
]) }).mount('#app')
