<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Download,
  FilePlus2,
  FolderPlus,
  FolderSearch,
  LoaderCircle,
  MonitorUp,
  Play,
  RefreshCw,
  RotateCcw,
  Trash2,
  X,
  XCircle,
} from "lucide-vue-next";
import { dispatchToast } from "@/services/toast";
import * as demoApi from "@/services/tauri/demo";
import { useDemoStore } from "@/stores/demo";
import { useCs2Store } from "@/stores/cs2";
import type {
  DemoAnalysisJob,
  DemoListItem,
  MatchDuelMatrix,
  MatchEventPage,
  MatchUtilityRow,
  PlayerMatchDetail,
} from "@/types/demo";
import MatchViewer2D from "@/features/demo/components/MatchViewer2D.vue";
import MatchPerformanceRadar from "@/features/demo/components/MatchPerformanceRadar.vue";

const demo = useDemoStore();
const cs2 = useCs2Store();
const tab = ref<"library" | "report">("library");
const reportTab = ref<
  | "overview"
  | "scoreboard"
  | "performance"
  | "rounds"
  | "duels"
  | "utility"
  | "events"
  | "viewer"
  | "heatmap"
>("overview");
const reportTabs = [
  { key: "overview", label: "总览" },
  { key: "scoreboard", label: "记分板" },
  { key: "performance", label: "表现雷达" },
  { key: "rounds", label: "回合" },
  { key: "duels", label: "对枪" },
  { key: "utility", label: "道具" },
  { key: "events", label: "事件" },
  { key: "viewer", label: "地图回放" },
  { key: "heatmap", label: "热力图" },
] as const;
const duels = ref<MatchDuelMatrix | null>(null);
const utility = ref<MatchUtilityRow[]>([]);
const events = ref<MatchEventPage | null>(null);
const playerDetail = ref<PlayerMatchDetail | null>(null);
const radarFocusKey = ref<string | null>(null);
const reportBusy = ref(false);
const deleteCandidate = ref<DemoListItem | null>(null);
const deleteError = ref("");
const deleteJobCandidate = ref<DemoAnalysisJob | null>(null);
const deleteJobError = ref("");
const jobBusy = ref<Record<number, "cancel" | "retry" | "delete" | undefined>>({});
const spatialRounds = computed(() =>
  (demo.report?.rounds || [])
    .filter((round) => round.endTick != null)
    .map((round) => ({
      roundNumber: round.number,
      startTick: round.startTick,
      freezeEndTick: null,
      endTick: round.endTick,
      officialEndTick: null,
      winnerSide: round.winner,
      reason: round.reason,
      eventCount: round.events?.length || round.kills.length + round.bombEvents.length,
    })),
);
function moveReportTab(event: KeyboardEvent) {
  if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
  event.preventDefault();
  const current = reportTabs.findIndex((item) => item.key === reportTab.value);
  const next =
    event.key === "Home"
      ? 0
      : event.key === "End"
        ? reportTabs.length - 1
        : (current + (event.key === "ArrowRight" ? 1 : -1) + reportTabs.length) % reportTabs.length;
  reportTab.value = reportTabs[next]!.key;
  requestAnimationFrame(() =>
    (event.currentTarget as HTMLElement).parentElement
      ?.querySelector<HTMLElement>(`[data-report-tab="${reportTab.value}"]`)
      ?.focus(),
  );
}
const pages = computed(() => Math.max(1, Math.ceil(demo.total / demo.pageSize)));
const scoreboardGroups = computed(() => {
  if (!demo.report) return [];
  return [
    {
      key: "t",
      label: "T 阵营",
      players: demo.report.players.filter(
        (player) => player.teamNumber === 2 || player.team === "T",
      ),
    },
    {
      key: "ct",
      label: "CT 阵营",
      players: demo.report.players.filter(
        (player) => player.teamNumber === 3 || player.team === "CT",
      ),
    },
    {
      key: "observer",
      label: "观战阵营",
      players: demo.report.players.filter((player) => player.participantRole === "observer"),
    },
    {
      key: "unknown",
      label: "未识别队伍",
      players: demo.report.players.filter((player) => player.participantRole === "unknown"),
    },
  ].filter((group) => group.players.length);
});
const statusLabel: Record<string, string> = {
  queued: "等待 Demo 写入完成",
  parsing: "解析中",
  done: "已完成",
  error: "解析失败",
};
const jobStageLabel: Record<string, string> = {
  queued: "等待分析",
  fingerprinting: "校验文件",
  parsing_core: "解析核心数据",
  normalizing: "规范化",
  persisting: "写入数据库",
  computing_metrics: "计算指标",
  done: "已完成",
  error: "失败",
  canceled: "已取消",
};
const eventLabel: Record<string, string> = {
  player_death: "击杀",
  bomb_beginplant: "开始下包",
  bomb_planted: "已下包",
  bomb_begindefuse: "开始拆包",
  bomb_defused: "已拆包",
  bomb_exploded: "炸弹爆炸",
};
const fmtDate = (value: number | null) =>
  value ? new Date(value).toLocaleString("zh-CN", { hour12: false }) : "--";
const fmtSize = (value: number) => `${(value / 1024 / 1024).toFixed(1)} MB`;
async function chooseRoot() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = await open({ directory: true, multiple: false, title: "添加其他 Demo 目录" });
  if (typeof path === "string") await demo.addRoot(path);
}
async function chooseFile() {
  const { open } = await import("@tauri-apps/plugin-dialog");
  const path = await open({
    directory: false,
    multiple: false,
    filters: [{ name: "CS2 Demo", extensions: ["dem"] }],
    title: "导入 Demo",
  });
  if (typeof path === "string") {
    await demo.importFile(path);
    if (demo.report) tab.value = "report";
  }
}
async function loadMatchData(id: number) {
  reportBusy.value = true;
  try {
    [duels.value, utility.value, events.value] = await Promise.all([
      demoApi.getMatchDuels(id),
      demoApi.getMatchUtility(id),
      demoApi.getMatchEvents(id, {}, 1, 100),
    ]);
  } catch (error) {
    duels.value = null;
    dispatchToast({ tone: "warn", title: "部分数据暂不可用", message: String(error) });
  } finally {
    reportBusy.value = false;
  }
}
async function openReport(id: number) {
  await demo.openReport(id);
  if (demo.report) {
    tab.value = "report";
    await loadMatchData(id);
  }
}
async function popoutReport(id: number) {
  try {
    await invoke("open_scoreboard", { reportId: id });
  } catch (error) {
    dispatchToast({ tone: "danger", title: "战报打开失败", message: String(error) });
  }
}
const playTitle = (id: number) =>
  demo.rowBusy[id] === "play"
    ? "正在启动 CS2"
    : cs2.cs2Running
      ? "请先退出 CS2"
      : !cs2.selectedRoot
        ? "请先选择 CS2 目录"
        : "用 CS2 播放 Demo";
async function playDemo(id: number) {
  if (!cs2.selectedRoot || cs2.cs2Running) return;
  await demo.play(id, cs2.selectedRoot);
}
function requestDelete(item: DemoListItem) {
  deleteError.value = "";
  deleteCandidate.value = item;
}
function cancelDelete() {
  if (deleteCandidate.value && demo.rowBusy[deleteCandidate.value.id] === "delete") return;
  deleteCandidate.value = null;
  deleteError.value = "";
}
async function confirmDelete() {
  const item = deleteCandidate.value;
  if (!item) return;
  deleteError.value = "";
  try {
    await demo.deleteFile(item.id);
    deleteCandidate.value = null;
  } catch (error) {
    deleteError.value = error instanceof Error ? error.message : String(error);
  }
}
async function openPlayer(stableKey: string) {
  if (!demo.report) return;
  playerDetail.value = await demoApi.getPlayerMatchDetail(
    demo.report.summary.demoFileId,
    stableKey,
  );
}
function openPlayerRadar() {
  if (!playerDetail.value) return;
  radarFocusKey.value = playerDetail.value.player.stableKey;
  reportTab.value = "performance";
  playerDetail.value = null;
}
async function exportReport(format: "json" | "csv") {
  if (!demo.report) return;
  const { save } = await import("@tauri-apps/plugin-dialog");
  const destination = await save({
    title: `导出 ${format.toUpperCase()}`,
    defaultPath: `${demo.report.summary.mapName || "match"}-${demo.report.summary.demoFileId}.${format}`,
    filters: [{ name: format.toUpperCase(), extensions: [format] }],
  });
  if (!destination) return;
  const result = await demoApi.exportMatch(demo.report.summary.demoFileId, format, destination);
  dispatchToast({
    tone: "ready",
    title: "导出完成",
    message: `${result.bytesWritten} bytes · ${result.path}`,
  });
}
async function cancelJob(jobId: number) {
  if (jobBusy.value[jobId]) return;
  jobBusy.value[jobId] = "cancel";
  try {
    await demoApi.cancelAnalysisJob(jobId);
    await demo.refresh();
  } finally {
    delete jobBusy.value[jobId];
  }
}
async function retryJob(jobId: number) {
  if (jobBusy.value[jobId]) return;
  jobBusy.value[jobId] = "retry";
  try {
    await demoApi.retryAnalysisJob(jobId);
    await demo.refresh();
  } finally {
    delete jobBusy.value[jobId];
  }
}
function requestDeleteJob(job: DemoAnalysisJob) {
  if (!["error", "canceled"].includes(job.stage) || jobBusy.value[job.id]) return;
  deleteJobError.value = "";
  deleteJobCandidate.value = job;
}
function cancelDeleteJob() {
  const job = deleteJobCandidate.value;
  if (job && jobBusy.value[job.id] === "delete") return;
  deleteJobCandidate.value = null;
  deleteJobError.value = "";
}
async function confirmDeleteJob() {
  const job = deleteJobCandidate.value;
  if (!job || jobBusy.value[job.id]) return;
  deleteJobError.value = "";
  jobBusy.value[job.id] = "delete";
  try {
    await demoApi.deleteAnalysisJob(job.id);
    deleteJobCandidate.value = null;
    await demo.refresh();
    dispatchToast({ tone: "ready", title: "分析任务已删除", message: "仅移除了队列记录，原始 Demo 文件仍然保留。" });
  } catch (error) {
    deleteJobError.value = error instanceof Error ? error.message : String(error);
  } finally {
    delete jobBusy.value[job.id];
  }
}
let filterTimer: ReturnType<typeof setTimeout> | undefined;
watch([() => demo.query, () => demo.status, () => demo.pageSize], () => {
  demo.page = 1;
  clearTimeout(filterTimer);
  filterTimer = setTimeout(() => demo.refresh(), 180);
});
watch(
  () => demo.page,
  () => demo.refresh(),
);
let jobPoll: ReturnType<typeof setInterval> | undefined;
onMounted(() => {
  demo.refresh();
  jobPoll = setInterval(() => {
    if (
      demo.jobs.some((job) => !["done", "spatial_done", "error", "canceled"].includes(job.stage)) &&
      !demo.busy
    )
      demo.refresh();
  }, 1000);
});
onBeforeUnmount(() => clearInterval(jobPoll));
</script>

<template>
  <section class="tool-view demo-review" aria-labelledby="demo-title">
    <header class="view-heading demo-heading">
      <div>
        <p class="overline">本地数据</p>
        <h1 id="demo-title">对局复盘</h1>
      </div>
      <div class="demo-actions">
        <button class="secondary-button" type="button" @click="chooseRoot">
          <FolderPlus :size="18" />添加其他目录</button
        ><button class="secondary-button" type="button" @click="chooseFile">
          <FilePlus2 :size="18" />导入 Demo</button
        ><button
          class="primary-button"
          type="button"
          :disabled="demo.busy === 'scan' || !demo.roots.length"
          @click="demo.scan"
        >
          <RefreshCw :size="18" :class="{ spinning: demo.busy === 'scan' }" />{{
            demo.busy === "scan" ? "正在扫描" : "立即扫描"
          }}
        </button>
      </div>
    </header>
    <div class="demo-tabs" role="tablist" aria-label="对局复盘视图">
      <button type="button" role="tab" :aria-selected="tab === 'library'" @click="tab = 'library'">
        录像库</button
      ><button
        type="button"
        role="tab"
        :aria-selected="tab === 'report'"
        :disabled="!demo.report"
        @click="tab = 'report'"
      >
        对局报告
      </button>
    </div>
    <p v-if="demo.error" class="inline-error" role="alert">{{ demo.error }}</p>
    <template v-if="tab === 'library'">
      <section
        v-if="demo.jobs.some((job) => !['done', 'spatial_done'].includes(job.stage))"
        class="analysis-jobs"
        aria-labelledby="jobs-title"
      >
        <div class="section-heading">
          <div>
            <h2 id="jobs-title">分析队列</h2>
            <p>任务状态由本地 SQLite 持久保存。</p>
          </div>
        </div>
        <div
          v-for="job in demo.jobs
            .filter((item) => !['done', 'spatial_done'].includes(item.stage))
            .slice(0, 8)"
          :key="job.id"
          class="analysis-job"
        >
          <div>
            <strong :title="job.fileName">{{ job.fileName }}</strong
            ><span
              >{{ jobStageLabel[job.stage] || job.stage
              }}<template v-if="job.errorDetail"> · {{ job.errorDetail }}</template></span
            >
          </div>
          <progress :value="job.progress" max="100">{{ job.progress }}%</progress
          ><output>{{ job.progress }}%</output
          ><button
            v-if="!['error', 'canceled'].includes(job.stage)"
            class="icon-button"
            type="button"
            title="取消分析"
            aria-label="取消分析"
            :disabled="Boolean(jobBusy[job.id])"
            @click="cancelJob(job.id)"
          >
            <LoaderCircle v-if="jobBusy[job.id] === 'cancel'" class="spinning" :size="17" />
            <XCircle v-else :size="17" />
          </button>
          <template v-else>
            <button
              class="icon-button"
              type="button"
              title="重试分析"
              aria-label="重试分析"
              :disabled="Boolean(jobBusy[job.id])"
              @click="retryJob(job.id)"
            >
              <LoaderCircle v-if="jobBusy[job.id] === 'retry'" class="spinning" :size="17" />
              <RotateCcw v-else :size="17" />
            </button>
            <button
              class="icon-button demo-delete-button"
              type="button"
              title="删除任务记录"
              aria-label="删除任务记录"
              :disabled="Boolean(jobBusy[job.id])"
              @click="requestDeleteJob(job)"
            >
              <LoaderCircle v-if="jobBusy[job.id] === 'delete'" class="spinning" :size="17" />
              <Trash2 v-else :size="17" />
            </button>
          </template>
        </div>
      </section>
      <section class="demo-roots" aria-labelledby="roots-title">
        <div class="section-heading">
          <div>
            <h2 id="roots-title">扫描目录</h2>
            <p>CS2 目录会自动关联 game\csgo；通常无需手动添加。</p>
          </div>
        </div>
        <div v-if="demo.roots.length" class="root-list">
          <div v-for="root in demo.roots" :key="root.id" class="root-row">
            <label
              ><input
                type="checkbox"
                :checked="root.enabled"
                @change="demo.updateRoot(root, ($event.target as HTMLInputElement).checked)"
              /><span class="root-path" :title="root.path">{{ root.path }}</span
              ><small v-if="root.origin === 'selected_cs2_root'">自动</small></label
            ><select
              :value="root.scanDepth"
              aria-label="扫描深度"
              @change="
                demo.updateRoot(
                  root,
                  root.enabled,
                  Number(($event.target as HTMLSelectElement).value),
                )
              "
            >
              <option v-for="depth in [0, 1, 2, 3, 5]" :key="depth" :value="depth">
                深度 {{ depth }}
              </option></select
            ><span>{{ fmtDate(root.lastScanAt) }}</span
            ><button
              class="icon-button"
              type="button"
              title="移除目录"
              aria-label="移除目录"
              @click="demo.removeRoot(root.id)"
            >
              <Trash2 :size="17" />
            </button>
          </div>
        </div>
        <div v-else class="demo-empty">
          <strong>尚未添加 Demo 目录</strong
          ><span>添加 CS2 replays 目录，或直接导入一个 .dem 文件。</span
          ><button class="secondary-button" type="button" @click="chooseRoot">
            <FolderPlus :size="18" />添加其他目录
          </button>
        </div>
      </section>
      <div class="demo-filters">
        <label
          ><span class="sr-only">搜索录像</span
          ><input v-model="demo.query" type="search" placeholder="搜索文件名或地图" /></label
        ><select v-model="demo.status" aria-label="解析状态">
          <option value="all">全部状态</option>
          <option value="queued">等待写入完成</option>
          <option value="done">已完成</option>
          <option value="parsing">解析中</option>
          <option value="error">解析失败</option></select
        ><select v-model.number="demo.pageSize" aria-label="每页数量">
          <option :value="25">25 / 页</option>
          <option :value="50">50 / 页</option>
          <option :value="100">100 / 页</option>
        </select>
      </div>
      <div class="demo-table-wrap">
        <table class="demo-table">
          <thead>
            <tr>
              <th>文件名</th>
              <th>地图</th>
              <th>回合</th>
              <th>击杀</th>
              <th>文件时间</th>
              <th>大小</th>
              <th>状态</th>
              <th class="demo-action-heading">操作</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in demo.items"
              :key="item.id"
              @dblclick="item.status === 'done' && openReport(item.id)"
            >
              <td>
                <strong :title="item.path">{{ item.fileName }}</strong>
              </td>
              <td>{{ item.mapName || "--" }}</td>
              <td>{{ item.totalRounds ?? "--" }}</td>
              <td>{{ item.kills ?? "--" }}</td>
              <td>{{ fmtDate(item.mtimeMs) }}</td>
              <td>{{ fmtSize(item.sizeBytes) }}</td>
              <td>
                <span class="demo-status" :data-status="item.status">{{
                  statusLabel[item.status] || item.status
                }}</span>
              </td>
              <td>
                <div class="demo-row-actions">
                  <button
                    class="icon-button"
                    type="button"
                    :title="playTitle(item.id)"
                    aria-label="用 CS2 播放 Demo"
                    :disabled="
                      Boolean(demo.rowBusy[item.id]) || cs2.cs2Running || !cs2.selectedRoot
                    "
                    @click.stop="playDemo(item.id)"
                  >
                    <LoaderCircle
                      v-if="demo.rowBusy[item.id] === 'play'"
                      class="spinning"
                      :size="17"
                    /><Play v-else :size="17" /></button
                  ><button
                    class="icon-button"
                    type="button"
                    title="在文件夹中显示 Demo"
                    aria-label="在文件夹中显示 Demo"
                    :disabled="Boolean(demo.rowBusy[item.id])"
                    @click.stop="demo.reveal(item.id)"
                  >
                    <LoaderCircle
                      v-if="demo.rowBusy[item.id] === 'reveal'"
                      class="spinning"
                      :size="17"
                    /><FolderSearch v-else :size="17" /></button
                  ><button
                    v-if="item.status === 'done'"
                    class="icon-button"
                    type="button"
                    title="弹出本局战报"
                    aria-label="弹出本局战报"
                    @click.stop="popoutReport(item.id)"
                  >
                    <MonitorUp :size="17" /></button
                  ><button
                    v-if="item.status === 'done'"
                    class="text-button"
                    type="button"
                    @click.stop="openReport(item.id)"
                  >
                    打开报告</button
                  ><button
                    v-else-if="item.status === 'error'"
                    class="icon-button"
                    type="button"
                    title="重新解析"
                    aria-label="重新解析"
                    @click.stop="demo.retry(item.id)"
                  >
                    <RotateCcw :size="16" /></button
                  ><button
                    class="icon-button demo-delete-button"
                    type="button"
                    title="删除 Demo"
                    aria-label="删除 Demo"
                    :disabled="Boolean(demo.rowBusy[item.id])"
                    @click.stop="requestDelete(item)"
                  >
                    <LoaderCircle
                      v-if="demo.rowBusy[item.id] === 'delete'"
                      class="spinning"
                      :size="17"
                    /><Trash2 v-else :size="17" />
                  </button>
                </div>
              </td>
            </tr>
            <tr v-if="!demo.items.length">
              <td colspan="8">
                <div class="demo-empty">
                  <strong>{{
                    demo.busy === "list" ? "正在读取录像库" : "没有符合条件的 Demo"
                  }}</strong
                  ><span>导入文件或扫描已添加目录。</span>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
      <footer class="demo-pagination">
        <span>共 {{ demo.total }} 个 Demo</span
        ><button type="button" :disabled="demo.page <= 1" @click="demo.page--">上一页</button
        ><span>{{ demo.page }} / {{ pages }}</span
        ><button type="button" :disabled="demo.page >= pages" @click="demo.page++">下一页</button>
      </footer>
    </template>
    <template v-else-if="demo.report">
      <section class="report-summary">
        <div>
          <p class="overline">比赛摘要</p>
          <h2>{{ demo.report.summary.mapName || "未知地图" }}</h2>
          <p :title="demo.report.summary.path">{{ demo.report.summary.fileName }}</p>
          <div class="report-export">
            <button class="secondary-button" type="button" @click="exportReport('csv')">
              <Download :size="16" />CSV</button
            ><button class="secondary-button" type="button" @click="exportReport('json')">
              <Download :size="16" />JSON
            </button>
          </div>
        </div>
        <dl>
          <div>
            <dt>回合</dt>
            <dd>{{ demo.report.summary.totalRounds || "--" }}</dd>
          </div>
          <div>
            <dt>击杀事件</dt>
            <dd>{{ demo.report.summary.totalKills }}</dd>
          </div>
          <div>
            <dt>文件时间</dt>
            <dd>{{ fmtDate(demo.report.summary.fileTimeMs) }}</dd>
          </div>
          <div>
            <dt>指标版本</dt>
            <dd>{{ demo.report.metricsVersion }}</dd>
          </div>
        </dl>
      </section>
      <div class="match-tabs" role="tablist" aria-label="比赛数据" @keydown="moveReportTab">
        <button
          v-for="item in reportTabs"
          :key="item.key"
          type="button"
          role="tab"
          :data-report-tab="item.key"
          :aria-selected="reportTab === item.key"
          :tabindex="reportTab === item.key ? 0 : -1"
          @click="reportTab = item.key"
        >
          {{ item.label }}
        </button>
      </div>
      <section v-if="reportTab === 'overview'" class="report-section match-overview">
        <div class="section-heading">
          <div>
            <h2>比赛总览</h2>
            <p>缺失数据保持为 --，不以 0 代替解析失败。</p>
          </div>
        </div>
        <div class="overview-kpis">
          <div>
            <span>比分</span
            ><strong>{{
              demo.report.summary.teamAScore == null || demo.report.summary.teamBScore == null
                ? "--"
                : `CT ${demo.report.summary.teamAScore} : ${demo.report.summary.teamBScore} T`
            }}</strong>
          </div>
          <div>
            <span>正式回合</span><strong>{{ demo.report.summary.totalRounds || "--" }}</strong>
          </div>
          <div>
            <span>玩家</span><strong>{{ demo.report.players.length || "--" }}</strong>
          </div>
          <div>
            <span>数据质量</span><strong>{{ demo.report.dataQuality.scoreboardStatus }}</strong>
          </div>
        </div>
      </section>
      <section v-if="reportTab === 'scoreboard'" class="report-section">
        <div class="section-heading scoreboard-heading">
          <div>
            <h2>记分板</h2>
            <p>
              {{
                demo.report.dataQuality.scoreboardStatus === "complete"
                  ? "终局身份与统计已完整解析。"
                  : demo.report.dataQuality.scoreboardStatus === "partial"
                    ? "已启用兼容恢复，缺失字段保持为 --。"
                    : "当前报告没有可用的玩家身份。"
              }}
            </p>
          </div>
          <button
            v-if="demo.report.dataQuality.scoreboardStatus !== 'complete'"
            class="secondary-button"
            type="button"
            :disabled="demo.busy === `retry-${demo.report.summary.demoFileId}`"
            @click="demo.retry(demo.report.summary.demoFileId)"
          >
            <RotateCcw :size="16" />重新解析
          </button>
        </div>
        <div
          v-if="demo.report.dataQuality.warnings.length"
          class="scoreboard-warning"
          role="status"
          aria-live="polite"
        >
          <p v-for="warning in demo.report.dataQuality.warnings" :key="warning">{{ warning }}</p>
        </div>
        <div class="demo-table-wrap">
          <table class="demo-table scoreboard">
            <thead>
              <tr>
                <th>玩家</th>
                <th>SteamID</th>
                <th>队伍</th>
                <th>K</th>
                <th>D</th>
                <th>A</th>
                <th>伤害</th>
                <th>爆头</th>
              </tr>
            </thead>
            <tbody>
              <template v-for="group in scoreboardGroups" :key="group.key"
                ><tr class="scoreboard-group" :data-team="group.key">
                  <th colspan="8">{{ group.label }} · {{ group.players.length }} 人</th>
                </tr>
                <tr v-for="player in group.players" :key="player.key">
                  <td :title="player.name || undefined">
                    <button class="player-link" type="button" @click="openPlayer(player.key)">
                      {{ player.name || "--" }}
                    </button>
                  </td>
                  <td>
                    {{ player.steamId || (player.userId != null ? `玩家 ${player.userId}` : "--") }}
                  </td>
                  <td>
                    {{ player.team || (player.participantRole === "observer" ? "观战阵营" : "--") }}
                  </td>
                  <td>{{ player.kills ?? "--" }}</td>
                  <td>{{ player.deaths ?? "--" }}</td>
                  <td>{{ player.assists ?? "--" }}</td>
                  <td>{{ player.damage ?? "--" }}</td>
                  <td>{{ player.headshots ?? "--" }}</td>
                </tr></template
              >
              <tr v-if="!demo.report.players.length">
                <td colspan="8">
                  {{
                    demo.report.schemaVersion < 2
                      ? "报告由旧解析器生成，需要重新解析。"
                      : demo.report.dataQuality.warnings[0] || "解析器未能恢复玩家身份。"
                  }}
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
      <section v-if="reportTab === 'performance'" class="report-section report-performance-section">
        <MatchPerformanceRadar
          :demo-id="demo.report.summary.demoFileId"
          :initial-player-key="radarFocusKey"
        />
      </section>
      <section v-if="reportTab === 'rounds'" class="report-section">
        <div class="section-heading">
          <div>
            <h2>回合时间线</h2>
            <p>击杀与炸弹事件按已解析的回合边界归组。</p>
          </div>
        </div>
        <div class="round-list">
          <details
            v-for="round in demo.report.rounds"
            :key="round.number"
            :open="round.number === 1"
          >
            <summary>
              <strong>第 {{ round.number }} 回合</strong
              ><span>Tick {{ round.startTick ?? "--" }} - {{ round.endTick ?? "--" }}</span
              ><span>{{ round.kills.length }} 击杀 · {{ round.bombEvents.length }} 炸弹事件</span>
            </summary>
            <div class="event-list">
              <div
                v-for="event in [...round.kills, ...round.bombEvents].sort(
                  (a, b) => a.tick - b.tick,
                )"
                :key="`${event.tick}-${event.kind}`"
              >
                <time>Tick {{ event.tick }}</time
                ><strong>{{ eventLabel[event.kind] || event.kind }}</strong
                ><span
                  >{{ event.actor || "--"
                  }}<template v-if="event.target"> → {{ event.target }}</template
                  ><template v-if="event.weapon"> · {{ event.weapon }}</template
                  ><template v-if="event.headshot"> · 爆头</template></span
                >
              </div>
              <p v-if="!round.kills.length && !round.bombEvents.length">
                本回合没有已解析的关键事件。
              </p>
            </div>
          </details>
        </div>
      </section>
      <section v-if="reportTab === 'duels'" class="report-section">
        <div class="section-heading">
          <div>
            <h2>对枪矩阵</h2>
            <p>仅统计双方真实击杀；格子为 X 击杀 Y : Y 击杀 X。</p>
          </div>
        </div>
        <div v-if="reportBusy" class="demo-empty" role="status">正在读取对枪数据…</div>
        <div v-else-if="!duels" class="demo-empty" role="status">当前报告没有可用的对枪数据。</div>
        <div v-else class="duel-matrix-wrap">
          <table class="duel-matrix">
            <caption>
              {{
                duels.teamX.label
              }}
              ×
              {{
                duels.teamY.label
              }}
              双向击杀
            </caption>
            <thead>
              <tr>
                <th class="duel-axis-corner">
                  {{ duels.teamX.label }} 玩家 ↓ / {{ duels.teamY.label }} 玩家 →
                </th>
                <th
                  v-for="player in duels.teamY.players"
                  :key="player.key"
                  :title="player.name || player.key"
                >
                  <span>{{ player.name || player.key }}</span
                  ><small v-if="player.isBot">BOT</small>
                </th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="xPlayer in duels.teamX.players" :key="xPlayer.key">
                <th :title="xPlayer.name || xPlayer.key">
                  <span>{{ xPlayer.name || xPlayer.key }}</span
                  ><small v-if="xPlayer.isBot">BOT</small>
                </th>
                <td
                  v-for="yPlayer in duels.teamY.players"
                  :key="yPlayer.key"
                  :aria-label="`${xPlayer.name || xPlayer.key} 击杀 ${yPlayer.name || yPlayer.key} ${duels.cells.find((cell) => cell.xPlayerKey === xPlayer.key && cell.yPlayerKey === yPlayer.key)?.xKillsY || 0} 次；${yPlayer.name || yPlayer.key} 击杀 ${xPlayer.name || xPlayer.key} ${duels.cells.find((cell) => cell.xPlayerKey === xPlayer.key && cell.yPlayerKey === yPlayer.key)?.yKillsX || 0} 次`"
                >
                  <template
                    v-for="cell in duels.cells.filter(
                      (item) => item.xPlayerKey === xPlayer.key && item.yPlayerKey === yPlayer.key,
                    )"
                    :key="cell.xPlayerKey + cell.yPlayerKey"
                    ><strong>{{ cell.xKillsY }} : {{ cell.yKillsX }}</strong></template
                  ><template
                    v-if="
                      !duels.cells.some(
                        (cell) =>
                          cell.xPlayerKey === xPlayer.key && cell.yPlayerKey === yPlayer.key,
                      )
                    "
                    ><strong>0 : 0</strong></template
                  >
                </td>
              </tr>
              <tr v-if="!duels.teamX.players.length || !duels.teamY.players.length">
                <td colspan="99">没有可识别的双方玩家。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
      <section v-if="reportTab === 'utility'" class="report-section">
        <div class="section-heading">
          <div>
            <h2>道具贡献</h2>
            <p>伤害受害者数据缺失时保持为 --。</p>
          </div>
        </div>
        <div class="demo-table-wrap">
          <table class="demo-table">
            <thead>
              <tr>
                <th>玩家</th>
                <th>HE 伤害</th>
                <th>致盲人数</th>
                <th>投掷事件</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in utility" :key="row.stableKey">
                <td>{{ row.playerName || row.stableKey }}</td>
                <td>{{ row.heDamage ?? "--" }}</td>
                <td>{{ row.flashVictims }}</td>
                <td>{{ row.throws }}</td>
              </tr>
              <tr v-if="!utility.length">
                <td colspan="4">当前报告没有可用道具数据。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
      <section v-if="reportTab === 'events'" class="report-section">
        <div class="section-heading">
          <div>
            <h2>完整事件流</h2>
            <p>最多显示当前页 100 条，结果来自规范化事件表。</p>
          </div>
        </div>
        <div class="demo-table-wrap">
          <table class="demo-table">
            <thead>
              <tr>
                <th>Tick</th>
                <th>回合</th>
                <th>类型</th>
                <th>发起者</th>
                <th>目标</th>
                <th>质量</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="row in events?.items || []" :key="row.id">
                <td>{{ row.tick }}</td>
                <td>{{ row.roundNumber ?? "--" }}</td>
                <td>{{ eventLabel[row.kind] || row.kind }}</td>
                <td>{{ row.actorKey || "--" }}</td>
                <td>{{ row.targetKey || "--" }}</td>
                <td>{{ row.quality || "--" }}</td>
              </tr>
              <tr v-if="!events?.items.length">
                <td colspan="6">当前筛选没有事件。</td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>
      <section v-if="reportTab === 'viewer'" class="report-section">
        <div class="section-heading">
          <div>
            <h2>二维地图回放</h2>
            <p>按回合加载压缩 spatial chunk，播放控制在卸载时自动释放。</p>
          </div>
        </div>
        <MatchViewer2D
          :demo-id="demo.report.summary.demoFileId"
          :map-name="demo.report.summary.mapName"
          :rounds="spatialRounds"
          mode="viewer"
        />
      </section>
      <section v-if="reportTab === 'heatmap'" class="report-section">
        <div class="section-heading">
          <div>
            <h2>位置热力图</h2>
            <p>以真实采样坐标渲染；未知地图不投影到默认坐标。</p>
          </div>
        </div>
        <MatchViewer2D
          :demo-id="demo.report.summary.demoFileId"
          :map-name="demo.report.summary.mapName"
          :rounds="spatialRounds"
          mode="heatmap"
        />
      </section>
    </template>
    <aside v-if="playerDetail" class="player-drawer" aria-label="玩家详情">
      <header>
        <div>
          <p class="overline">玩家详情</p>
          <h2>{{ playerDetail.player.name || playerDetail.player.stableKey }}</h2>
        </div>
        <button
          class="icon-button"
          type="button"
          title="关闭"
          aria-label="关闭玩家详情"
          @click="playerDetail = null"
        >
          <X :size="18" />
        </button>
      </header>
      <button class="secondary-button player-radar-link" type="button" @click="openPlayerRadar">
        在表现雷达中查看
      </button>
      <dl>
        <div>
          <dt>K / D / A</dt>
          <dd>
            {{ playerDetail.player.kills ?? "--" }} / {{ playerDetail.player.deaths ?? "--" }} /
            {{ playerDetail.player.assists ?? "--" }}
          </dd>
        </div>
        <div>
          <dt>ADR</dt>
          <dd>{{ playerDetail.player.adr?.toFixed(1) ?? "--" }}</dd>
        </div>
        <div>
          <dt>KAST</dt>
          <dd>
            {{
              playerDetail.player.kastPercent == null
                ? "--"
                : `${playerDetail.player.kastPercent.toFixed(1)}%`
            }}
          </dd>
        </div>
        <div>
          <dt>LBRating 2.0</dt>
          <dd>{{ playerDetail.player.rating?.toFixed(2) ?? "--" }}</dd>
        </div>
      </dl>
      <div class="demo-table-wrap">
        <table class="demo-table">
          <thead>
            <tr>
              <th>回合</th>
              <th>阵营</th>
              <th>K</th>
              <th>D</th>
              <th>A</th>
              <th>伤害</th>
              <th>KAST</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="round in playerDetail.rounds" :key="round.roundNumber">
              <td>{{ round.roundNumber }}</td>
              <td>{{ round.side || "--" }}</td>
              <td>{{ round.kills ?? "--" }}</td>
              <td>{{ round.deaths ?? "--" }}</td>
              <td>{{ round.assists ?? "--" }}</td>
              <td>{{ round.damageHealth ?? "--" }}</td>
              <td>{{ round.kast == null ? "--" : round.kast ? "是" : "否" }}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </aside>
    <Teleport to="body"
      ><div
        v-if="deleteCandidate"
        class="modal-backdrop demo-delete-backdrop"
        @click.self="cancelDelete"
      >
        <section
          class="confirm-dialog"
          role="alertdialog"
          aria-modal="true"
          aria-labelledby="delete-demo-title"
        >
          <h2 id="delete-demo-title">删除这段录像？</h2>
          <p>
            <strong>{{ deleteCandidate.fileName }}</strong>
          </p>
          <p>将永久删除磁盘上的 Demo 文件及其本地分析记录。此操作无法撤销。</p>
          <p v-if="deleteError" class="inline-error" role="alert">{{ deleteError }}</p>
          <div class="dialog-actions">
            <button
              class="secondary-button"
              type="button"
              :disabled="demo.rowBusy[deleteCandidate.id] === 'delete'"
              @click="cancelDelete"
            >
              保留录像</button
            ><button
              class="danger-button"
              type="button"
              :disabled="demo.rowBusy[deleteCandidate.id] === 'delete'"
              @click="confirmDelete"
            >
              <LoaderCircle
                v-if="demo.rowBusy[deleteCandidate.id] === 'delete'"
                class="spinning"
                :size="17"
              /><Trash2 v-else :size="17" />{{
                demo.rowBusy[deleteCandidate.id] === "delete" ? "正在删除" : "确认删除"
              }}
            </button>
          </div>
        </section>
      </div></Teleport
    >
    <Teleport to="body"
      ><div
        v-if="deleteJobCandidate"
        class="modal-backdrop demo-delete-backdrop"
        @click.self="cancelDeleteJob"
      >
        <section
          class="confirm-dialog"
          role="alertdialog"
          aria-modal="true"
          aria-labelledby="delete-analysis-job-title"
        >
          <h2 id="delete-analysis-job-title">删除这条分析任务？</h2>
          <p><strong>{{ deleteJobCandidate.fileName }}</strong></p>
          <p>将从分析队列中移除这条失败/已取消任务记录。</p>
          <p>原始 Demo 文件、录像库记录和已有对局报告不会删除。删除后仍可重新扫描或重新导入。</p>
          <p v-if="deleteJobError" class="inline-error" role="alert">{{ deleteJobError }}</p>
          <div class="dialog-actions">
            <button class="secondary-button" type="button" :disabled="jobBusy[deleteJobCandidate.id] === 'delete'" @click="cancelDeleteJob">
              取消
            </button>
            <button class="danger-button" type="button" :disabled="jobBusy[deleteJobCandidate.id] === 'delete'" @click="confirmDeleteJob">
              <LoaderCircle v-if="jobBusy[deleteJobCandidate.id] === 'delete'" class="spinning" :size="17" />
              <Trash2 v-else :size="17" />
              {{ jobBusy[deleteJobCandidate.id] === 'delete' ? '正在删除' : '删除任务' }}
            </button>
          </div>
        </section>
      </div></Teleport
    >
  </section>
</template>
