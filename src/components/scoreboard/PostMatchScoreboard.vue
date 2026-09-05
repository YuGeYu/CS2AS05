<script setup lang="ts">
import { computed, ref } from 'vue'
import type { DemoPlayer, DemoReport } from '@/types/demo'

const props = defineProps<{ report: DemoReport }>()
const expanded = ref<string | null>(null)
const teamPlayers = (team: number, label: string) => props.report.players.filter(player => player.teamNumber === team || player.team === label)
const groups = computed(() => [
  { key: 'ct', label: 'CT 阵营', players: teamPlayers(3, 'CT') },
  { key: 't', label: 'T 阵营', players: teamPlayers(2, 'T') },
  { key: 'observer', label: '观战阵营', players: props.report.players.filter(player => player.participantRole === 'observer') },
  { key: 'unknown', label: '未识别阵营', players: props.report.players.filter(player => player.participantRole === 'unknown') },
].map(group => ({ ...group, players: [...group.players].sort((a, b) => (b.rating?.rating ?? -1) - (a.rating?.rating ?? -1)) })).filter(group => group.players.length))
const value = (number: number | null, digits = 0) => number == null ? '--' : number.toFixed(digits)
const percent = (number: number | null) => number == null ? '--' : `${number.toFixed(1)}%`
const ratingLevel = (player: DemoPlayer) => player.rating == null ? '不可用' : player.rating.rating >= 1.1 ? '高' : player.rating.rating >= .9 ? '中' : '低'
const ratingClass = (player: DemoPlayer) => player.rating == null ? 'none' : player.rating.rating >= 1.1 ? 'high' : player.rating.rating >= .9 ? 'mid' : 'low'
const highest = (pick: (player: DemoPlayer) => number | null) => [...props.report.players].sort((a, b) => (pick(b) ?? -1) - (pick(a) ?? -1))[0]
const honors = computed(() => [
  { label: 'MVP', player: highest(player => player.rating?.rating ?? null) },
  { label: '最高 ADR', player: highest(player => player.adr) },
  { label: '最高 HS%', player: highest(player => player.headshotPercent) },
  { label: '最高 Kills', player: highest(player => player.kills) },
])
</script>

<template>
  <section class="post-scoreboard">
    <div class="match-band"><div><span>地图</span><strong>{{ report.summary.mapName || '--' }}</strong></div><div><span>比分</span><strong>{{ report.summary.teamAScore == null || report.summary.teamBScore == null ? '-- / 未验证' : `CT ${report.summary.teamAScore} : ${report.summary.teamBScore} T` }}</strong></div><div><span>报告回合</span><strong>{{ report.summary.totalRounds }}</strong></div><div class="match-file"><span>Demo</span><strong :title="report.summary.fileName">{{ report.summary.fileName }}</strong></div></div>
    <div class="honors"><div v-for="honor in honors" :key="honor.label"><span>{{ honor.label }}</span><strong>{{ honor.player?.name || '--' }}</strong></div></div>
    <div class="scoreboard-table-wrap"><table><thead><tr><th>玩家</th><th>K-D-A</th><th>ADR</th><th>HS%</th><th>LBRating 2.0</th></tr></thead><tbody v-for="group in groups" :key="group.key"><tr class="team-row" :data-team="group.key"><th colspan="5">{{ group.label }} · {{ group.players.length }} 人</th></tr><template v-for="player in group.players" :key="player.key"><tr class="player-row" tabindex="0" @click="expanded = expanded === player.key ? null : player.key" @keydown.enter="expanded = expanded === player.key ? null : player.key"><td :title="player.name || undefined">{{ player.name || '--' }}</td><td>{{ value(player.kills) }}-{{ value(player.deaths) }}-{{ value(player.assists) }}</td><td>{{ value(player.adr, 1) }}</td><td>{{ percent(player.headshotPercent) }}</td><td><span class="rating" :data-rating="ratingClass(player)">{{ player.rating ? player.rating.rating.toFixed(2) : '--' }} {{ ratingLevel(player) }}</span></td></tr><tr v-if="expanded === player.key" class="detail-row"><td colspan="5"><div><span>身份 {{ player.participantRole === 'observer' ? '观战阵营' : player.steamId || `玩家 ${player.userId ?? '--'}` }}</span><span>报告回合 {{ value(player.roundsPlayed) }}</span><span>模型 {{ player.rating?.modelVersion || '--' }}</span></div><p>KAST、交易、Swing 与经济指标需要逐回合同步状态，本版本暂不提供。</p><p v-if="!player.rating">{{ report.dataQuality.ratingWarnings?.join('；') || '该玩家缺少 K/D/A、伤害或报告回合。' }}</p></td></tr></template></tbody></table></div>
    <footer><span>解析 {{ report.dataQuality.scoreboardStatus }} · Rating {{ report.dataQuality.ratingStatus }}</span><span>LBRating 2.0 · lb-rating-2.0 · ADR 主导的多因子模型</span></footer>
  </section>
</template>
