import { flushPromises, mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import MatchPerformanceRadar from '@/features/demo/components/MatchPerformanceRadar.vue'
import * as api from '@/services/tauri/demo'
import type { MatchPerformanceRadar as RadarReport, PerformanceRadarPlayer } from '@/types/demo'

vi.mock('@/services/tauri/demo', () => ({ getMatchPerformanceRadar: vi.fn() }))

const keys = ['firepower', 'damage', 'survival', 'participation', 'teamwork', 'opening'] as const
const makePlayer = (stableKey: string, isBot = false, teamNumber: number | null = 2): PerformanceRadarPlayer => ({
  stableKey, name: stableKey, isBot, teamNumber, teamName: teamNumber === 2 ? 'T' : teamNumber === 3 ? 'CT' : null, roundsPlayed: teamNumber === 2 || teamNumber === 3 ? 10 : null,
  rawStats: { participatedRounds: 10, kills: 8, deaths: 4, assists: 2, damageHealth: 700, survivedRounds: 5, kastRounds: 7, multiKillRounds: 1, firstKills: 1, firstDeaths: 1, tradeKills: 1 },
  dimensions: keys.map((key, index) => ({ key, label: ['火力', '输出', '生存', '参战', '协同', '先手影响'][index], score: teamNumber === 2 || teamNumber === 3 ? 50 + index : null, raw: teamNumber === 2 || teamNumber === 3 ? .5 : null, rawLabel: '.50', unit: index === 2 || index === 3 ? '百分比' : 'KPR', benchmark: 1, source: 'test', quality: teamNumber === 2 || teamNumber === 3 ? 'complete' : 'unavailable' })),
  warnings: [],
})
const roster = [makePlayer('bot-first', true), makePlayer('human-b', false, 3), makePlayer('human-a'), makePlayer('human-c'), makePlayer('observer', false, 1)]
const response = (demoFileId: number, players = roster): RadarReport => ({ demoFileId, modelVersion: 'performance-radar-v1', denominator: 'player_participated_rounds', players, warnings: [], benchmarkMethod: 'second-highest-per-axis', cohortSize: players.length })

describe('MatchPerformanceRadar', () => {
  beforeEach(() => {
    vi.mocked(api.getMatchPerformanceRadar).mockImplementation(async (demoId, selected) => response(demoId, selected?.length ? roster.filter(player => selected.includes(player.stableKey)) : roster))
  })

  it('selects a non-BOT player deterministically and excludes observers from the picker', async () => {
    const wrapper = mount(MatchPerformanceRadar, { props: { demoId: 1 } })
    await flushPromises()
    expect(wrapper.get('button[aria-pressed="true"]').text()).toContain('human-a')
    expect(wrapper.findAll('.radar-player-picker button').map(button => button.text())).not.toContain('observer')
    expect(wrapper.text()).toContain('1 名观战者或无正式回合身份已保留')
    expect(wrapper.text()).toContain('表现雷达 v1 · 不等于 Rating')
    wrapper.unmount()
  })

  it('rejects a fourth player and explains the three-player limit', async () => {
    const wrapper = mount(MatchPerformanceRadar, { props: { demoId: 1 } })
    await flushPromises()
    const buttons = wrapper.findAll('.radar-player-picker button')
    await buttons.find(button => button.text().includes('human-b'))!.trigger('click'); await flushPromises()
    await buttons.find(button => button.text().includes('human-c'))!.trigger('click'); await flushPromises()
    expect(wrapper.findAll('button[aria-pressed="true"]')).toHaveLength(3)
    const bot = wrapper.findAll('.radar-player-picker button').find(button => button.text().includes('bot-first'))!
    expect(bot.attributes('disabled')).toBeDefined()
    expect(bot.attributes('title')).toContain('最多同时比较三名玩家')
    expect(wrapper.text()).toContain('已选择三名玩家')
    wrapper.unmount()
  })

  it('ignores stale responses after a Demo switch and releases timers on unmount', async () => {
    let resolveFirst!: (value: RadarReport) => void
    vi.mocked(api.getMatchPerformanceRadar)
      .mockImplementationOnce(() => new Promise(resolve => { resolveFirst = resolve }))
      .mockResolvedValueOnce(response(2, [makePlayer('new-demo')]))
    const wrapper = mount(MatchPerformanceRadar, { props: { demoId: 1 } })
    await wrapper.setProps({ demoId: 2 }); await flushPromises()
    resolveFirst(response(1, [makePlayer('stale-demo')]))
    await flushPromises()
    expect(wrapper.text()).toContain('new-demo')
    expect(wrapper.text()).not.toContain('stale-demo')
    wrapper.unmount()
  })
})
