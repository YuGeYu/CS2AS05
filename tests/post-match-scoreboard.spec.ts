import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'
import PostMatchScoreboard from '@/components/scoreboard/PostMatchScoreboard.vue'
import type { DemoPlayer, DemoReport } from '@/types/demo'

const player = (key: string, name: string, rating: number | null): DemoPlayer => ({
  key, name, steamId: key, userId: 1, isBot: false, teamNumber: 3, team: 'CT', kills: 14,
  deaths: 6, assists: 4, damage: 1640, headshots: 7, identitySource: 'controller',
  statsSource: 'controller_total', roundsPlayed: 20, roundsSurvived: null, kastRounds: null,
  multiKills: null, firstKills: null, firstDeaths: null, tradeKills: null, tradeDenials: null,
  adr: 82, kastPercent: null, headshotPercent: 50, roundSwing: null, economyAdjustment: null,
  ratingStatus: rating == null ? 'unavailable' : 'complete', rating: rating == null ? null : {
  modelVersion: 'lb-rating-2.0', killComponent: 2.4667, damageComponent: 1,
    survivalComponent: 1.0294, assistComponent: 1, rating,
  },
})

const report: DemoReport = {
  schemaVersion: 6, parserName: 'test', parserCommit: 'test', parserAdapterVersion: '2',
  metricsVersion: 'lb-rating-2.0', appVersion: '0.5.6',
  dataQuality: { scoreboardStatus: 'complete', warnings: [], entityParseStatus: 'strict', ratingStatus: 'partial', ratingWarnings: ['部分玩家不可用。'] },
  summary: { demoFileId: 1, fileName: 'match.dem', path: 'match.dem', mapName: 'de_dust2', serverName: 'local', fileTimeMs: 1, sizeBytes: 1, totalRounds: 20, totalKills: 28, teamAScore: null, teamBScore: null, parsedAt: 1 },
  players: [player('1', 'High', 1.23), player('2', 'Mid', .98), player('3', 'Low', .76), player('4', 'Missing', null)], rounds: [],
}

describe('post-match LBRating 2.0 scoreboard', () => {
  it('renders sorted values, tiers and unavailable state', () => {
    const wrapper = mount(PostMatchScoreboard, { props: { report } })
    expect(wrapper.text()).toContain('LBRating 2.0')
    expect(wrapper.text()).toContain('lb-rating-2.0')
    expect(wrapper.text()).not.toContain('simple-rating')
    expect(wrapper.text()).toContain('1.23 高')
    expect(wrapper.text()).toContain('0.98 中')
    expect(wrapper.text()).toContain('0.76 低')
    expect(wrapper.text()).toContain('-- 不可用')
    expect(wrapper.text()).toContain('MVPHigh')
  })

  it('uses report-round wording and expands advanced-metric disclosure', async () => {
    const wrapper = mount(PostMatchScoreboard, { props: { report } })
    await wrapper.find('.player-row').trigger('click')
    expect(wrapper.text()).toContain('报告回合 20')
    expect(wrapper.text()).toContain('KAST、交易、Swing 与经济指标')
    expect(wrapper.text()).toContain('lb-rating-2.0')
  })
})
