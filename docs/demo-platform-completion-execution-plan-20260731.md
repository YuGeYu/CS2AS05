# CS2 Demo 完整功能续作交接方案（2026-07-31）

> **执行 AI 预验收补充（2026-08-03）**：在请求用户进行 BOT/CS2 操作前，必须先完成 [执行 AI 独立预验收方案](E:\CS2AS05\docs\demo-platform-execution-ai-preflight-plan-20260803.md) 的 A-F Gate；该文档覆盖地图 seed、真实雷达热力图、Canvas/Tauri/性能预验收和调试候选交付。

> 本文给下一位实际执行 AI。它基于 `docs/demo-platform-upstream-integration-execution-report-20260730.md`，只规划未完成部分。先读取本文和执行报告，再修改代码。禁止回退当前工作树改动。

## 1. 当前权威基线

执行前保存：

```powershell
Set-Location E:\CS2AS05
git status --short
git rev-parse HEAD
git diff --stat
Get-Content -Raw .\docs\demo-platform-upstream-integration-execution-report-20260730.md
```

报告确认的基线：

- HEAD：`8552b554993fec66866d2133d315342be3e0cbca`；本轮所有 Demo 改动仍在未提交工作树，禁止 `reset`、`restore`、`clean`。
- `LaihoE/demoparser` 固定 `ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`，Rust path dependency 直接编译。
- `akiver/cs-demo-manager` 固定 `8961f5072fe4d42803dde68e8e71b3c90b216504`，MIT license、95 个地图 PNG、metadata、provenance 已落盘。
- SQLite 已是 `PRAGMA user_version=5`，4/4 Demo 已是 report schema 5 / parser adapter 3。
- 已有表：`checksum`、`demo_paths`、`analysis_jobs`、`matches`、`players`、`match_players`、`match_rounds`、`match_events`、`player_round_stats`、`round_economy`、`position_frames`。
- 当前计数：`matches=4`、`players=28`、`match_players=31`、`match_rounds=34`、`match_events=422`，但 `player_round_stats=0`、`round_economy=0`、`position_frames=0`。
- 已完成窗口和基础 UI：主窗口 `1440x900`/最小 `1100x700`，战报窗口 `1280x800`/最小 `980x640`；录像库、总览、记分板、回合、基础任务进度。
- 已通过前端 32 files/118 tests、Rust 45 tests + 2 ignored、真实 Demo 单测、typecheck/lint/build/fmt/check/clippy；clippy 仅有 vendor 已知 warning。
- dust2 样本为 5 completed、6 logical、10 players，最后回合无 end tick；Rating unavailable 是正确数据质量结果，禁止硬补 6 或 0。

### 不得重复

不要重新实现上游固定、provenance、v5 迁移、adapter 3、checksum/path 基础表、已有 library/overview/scoreboard/rounds 页面，除非新代码造成回归。现有用户改动默认保留。

## 2. 目标与阶段门禁

| 阶段 | 必须交付 | 门禁 |
|---|---|---|
| P0 | 真正持久 worker、并发、取消、重试、panic 隔离、稳定 watcher | 新 Demo 可稳定入队、取消、重试、重启恢复 |
| P1 | player-round、经济、KAST、trade、opening、clutch、utility、事件/聊天 | 规范化表真实非空，固定向量和真实样本通过 |
| P2 | 经济、对枪、道具、事件、玩家详情、导出 UI | API/图表/表格同源，null 不伪造 |
| P3 | spatial pass、压缩 position chunks、2D viewer、热力图 | dust2 与双层地图 Canvas 非空、seek/cleanup 通过 |
| P4 | 新 BOT 闭环、真实 Tauri、多 viewport、性能、候选包 | 全部真实验收通过后才可写“功能候选” |

## 3. 上游复用边界

### demoparser

它是唯一字节解析内核。新增 `src-tauri/src/demo/parser/demoparser_adapter.rs`，一次 core pass 请求 header、round/game props、玩家 totals、击杀/伤害/购买/闪光/燃烧/烟/炸弹/聊天事件；空间 pass 单独请求 ticks、位置、yaw、装备、投掷物。

禁止引入 Python、Node native addon、WASM parser、`@akiver/cs-demo-analyzer` sidecar 或第二套协议解析器。已验证的 `IllegalPathOp` 才能走无实体回退；未知错误保留根因。parser 必须在 worker 中 `catch_unwind`。

### cs-demo-manager

继续复用固定 commit 的 MIT 资产和模式：

- `third_party/cs-demo-manager/LICENSE`、`PROVENANCE.md`、地图 manifest；
- `src/assets/maps/cs2` 雷达/缩略图；
- `src/data/cs2-map-metadata.ts` 的 `pos_x/pos_y/scale/threshold_z` 和坐标变换；
- match/round/player/heatmap 字段集合、分析队列阶段、事件/经济/对枪/2D viewer 信息架构；
- ECharts 6.1.0 的按需图表模式。

禁止引入 Electron、PostgreSQL、psql、网络账户、视频制作链。新增依赖必须写入 NOTICE/provenance。

## 4. Rust 模块和 Worker

将当前巨型 `src-tauri/src/services/demo.rs` 拆为：

```text
src-tauri/src/demo/
  catalog.rs watcher.rs coordinator.rs job_store.rs export.rs
  parser/mod.rs parser/demoparser_adapter.rs parser/requested_fields.rs
  parser/raw_analysis.rs parser/identity.rs
  normalize/rounds.rs normalize/players.rs normalize/events.rs
  normalize/player_rounds.rs
  metrics/kast.rs metrics/trades.rs metrics/opening.rs
  metrics/clutches.rs metrics/economy.rs metrics/utility.rs metrics/rating.rs
  repository/migrations.rs repository/write_core.rs
  repository/library_queries.rs repository/match_queries.rs
  spatial/position_parser.rs spatial/chunk_codec.rs spatial/map_coordinates.rs
```

旧函数可暂时 re-export，新功能不能继续堆入旧文件。

### Worker 契约

1. `analysis_jobs` 用 `BEGIN IMMEDIATE` claim；增加/复用 `lease_owner`、`lease_until`、`cancel_requested`、`log_tail`。
2. 默认 core 并发 `max(1,min(2,logical_cpu/2))`，设置范围 1..4；spatial 最多 1 个。每个 worker 独立 SQLite connection。
3. 状态：`queued -> fingerprinting -> parsing_core -> normalizing -> persisting -> computing_metrics -> done`；空间为独立 `parsing_spatial -> spatial_done`。
4. `scan_demo_roots` 和 `import_demo_file` 只校验/入队/返回，不等待 parse。新增 `enqueue_demo_analysis`、`cancel_analysis_job`、`retry_analysis_job`。
5. 取消写数据库 flag 和内存 `AtomicBool`；parser 返回后、写库前后都检查，取消的结果不得提交。
6. 单 job `catch_unwind`，panic 写 `DEMO_PARSER_PANIC`，其他 worker 继续；启动将过期 lease 恢复 queued，最多自动重试 2 次。
7. 同 checksum + kind + parser/schema/metric 只允许一个 active job，不同路径只写 `demo_paths`。

### Watcher 契约

`src-tauri/src/demo/watcher.rs` 对 `.dem` 做 300 ms debounce；size + mtime 每 750 ms 检查，连续 3 次一致且最后写入至少 3 秒才入队。`.part`、权限、header 不支持、仍在写入分别记录状态。Rust 负责真正 enqueue，`demo://filesystem-changed` 只做 UI 刷新提示。

## 5. Core 规范化和指标

### 规范化写入

复用 v5 表，不另造空的平行表：

- `player_round_stats`：正式回合、玩家 side、start money、spent、equipment、kills/deaths/assists/damage、survived、traded、kast；
- `round_economy`：回合/队伍 freeze-end 装备价值、起始金、花费、economy type、metric version；
- `match_events`：保留完整 tick/kind/actor/target/payload；补 `(demo_id,kind,tick)` 索引；
- utility/duel/clutch 第一版可从 events 聚合，不要先创建空表伪装完成。

解析、normalize、metrics 后使用一个 transaction 删除该 Demo 的旧 normalized rows，再插入新 rows；旧 report 只在 transaction 成功后更新。失败必须保留旧快照。

### 指标版本化

新增 `metrics-v2`，不得改变已有 `simple-rating-v1` 语义：

- KAST：正式实际参赛回合中 Kill/Assist/Survive/Trade 任一成立，分母是实际参赛回合。
- Trade：同回合，队友在死亡后 5.0 秒内击杀同一 killer；使用 Demo tick interval，覆盖 `< == >` 边界。
- Opening：每回合第一条有效敌对击杀；自杀、teamkill、world 排除。
- Clutch：一方只剩一人面对 1..5 名敌人，赢回合才计 1vN。
- Economy：freeze-end 装备价值 + start money + spent；阈值集中在 `metrics/economy.rs`。
- Utility：HE damage、flash victims/duration、inferno damage、smoke/flash/he/inferno throws；缺受害者数据时事件保留，统计为 unavailable。
- Rating 继续写“简易 Rating”，不得冒充 HLTV/OpenRating；缺输入为 null/unavailable。

## 6. SQLite v6 迁移

不要重建 v5。用 WAL checkpoint + `VACUUM INTO` 备份后单事务迁移到 `user_version=6`：

- `analysis_jobs` 补 lease/worker/cancel/log 字段和 claim 索引；
- `player_round_stats` 补 side/survived/traded/kast/equipment/money/source；
- `round_economy` 补 team/quality/unique index；
- `match_events` 补 source/quality 和 `(demo_id,kind,tick)`；
- spatial 新增 `position_chunks(demo_id,round_number,sampling_hz,chunk_index,encoding,payload,first_tick,last_tick)`；v5 `position_frames` 保留兼容，不复制空数据；
- `map_metadata` 保存上游 commit、asset hash、坐标参数。

迁移测试覆盖全新 DB、报告中的真实 v5 DB、重复启动、失败回滚、备份完整性、`integrity_check=ok`、旧报告可读。只对版本不匹配的 job 重算，不能每次启动重跑历史 done。

## 7. Tauri API

在 `src-tauri/src/commands/demo.rs` 和 `src/services/tauri/demo.ts` 增加：

```text
get_match_economy(demo_id)
get_match_duels(demo_id)
get_match_utility(demo_id)
get_match_events(demo_id, filters, page, page_size)
get_player_match_detail(demo_id, stable_key)
export_match(demo_id, format, destination_path)
ensure_spatial_analysis(demo_id, sampling_hz)
get_round_positions(demo_id, round_number, sampling_hz)
get_heatmap_points(demo_id, filters)
save_heatmap_png(demo_id, filters, destination_path)
launch_demo_at_tick(demo_id, tick, player_key?)
```

所有命令限制 page size/tick/round/sampling rate；路径只能来自已索引 `demo_paths`。导出由 Rust 写 `.part` 后原子 rename，取消清理 `.part`，不使用浏览器下载、`window.open` 或 File System Access API。大结果按页/回合切片，禁止整场 JSON IPC。

## 8. UI 续作：强制通用蓝色 SaaS 模板

本节覆盖上一版方案中的视觉方向。按用户最新要求，必须照搬 UI/UX Pro Max 的通用蓝色 SaaS / Data-Dense Dashboard 模板，不再以青玉/古铜作为主色。

### 8.1 设计 tokens

采用以下稳定 tokens，并集中写入 `src/styles/main.css` 或 `src/styles/demo.css`：

```css
--color-primary: #1E40AF;
--color-primary-hover: #1D4ED8;
--color-secondary: #3B82F6;
--color-accent: #D97706;
--color-background: #F8FAFC;
--color-surface: #FFFFFF;
--color-surface-muted: #E9EEF6;
--color-foreground: #1E3A8A;
--color-text: #0F172A;
--color-text-muted: #475569;
--color-border: #DBEAFE;
--color-success: #15803D;
--color-warning: #B45309;
--color-danger: #DC2626;
```

深色模式使用同一语义 token 映射，不做整页单一深蓝；状态色必须同时有文字/图标。标题使用本地等宽技术字体或现有字体，正文使用现有本地字体，禁止运行时 Google Fonts 网络依赖。

### 8.2 工作台布局

保留现有大窗口尺寸和基础三 tab，在其后加入：

```text
总览 | 记分板 | 回合 | 经济 | 对枪 | 道具 | 事件 | 地图回放 | 热力图
```

使用 Data-Dense Dashboard：顶部全宽 KPI band，下面是可扫描的表格/图表网格，左侧筛选栏，右侧主数据区。允许单层卡片用于重复项目，不允许卡片套卡片或营销 hero。玩家行点击打开详情 drawer。

拆分 `src/views/DemoReviewView.vue`：

```text
src/features/demo/components/LibraryView.vue
AnalysisQueue.vue MatchShell.vue MatchOverview.vue MatchScoreboard.vue
MatchRounds.vue MatchEconomy.vue MatchDuels.vue MatchUtility.vue
MatchEvents.vue MatchPlayerDrawer.vue MatchViewer2D.vue MatchHeatmap.vue
src/features/demo/stores/library.ts jobs.ts match.ts spatial.ts
src/features/demo/composables/useAnalysisJobs.ts useViewerPlayback.ts
```

现有 `useDemoStore` 保留 facade，逐步委托新 store，避免破坏已通过测试的 library。

### 8.3 图表、图标和动画

- ECharts 6.1.0 按需 import，图表旁始终显示数值/表格替代；对枪矩阵同时提供排序表。
- 使用 `lucide-vue-next` 图标，icon-only 控件最小 44x44 px，带 tooltip 和 aria-label；禁止 emoji 结构图标。
- 页面/tab 进入 180–240 ms ease-out，退出 120–160 ms ease-in；只动 opacity 和 4..8px transform。
- 进度、回合时间轴、viewer 播放可动画；表格行、KPI、按钮不循环发光、不改变尺寸。
- `prefers-reduced-motion: reduce` 关闭位移、数字 tween、热力图渐入；viewer 只保留即时帧切换。
- 1440x900、1100x700、1280x800、980x640 均不得文字/按钮重叠；长路径/中文名 ellipsis + tooltip。

## 9. Spatial、Viewer、Heatmap

`ensure_spatial_analysis` 触发独立 spatial job，默认 8 Hz，可选 4/8/16 Hz；关键事件 tick 强制保留。解析 x/y/z/yaw/health/armor/team/alive/weapon/bomb/grenade，按 round delta 编码并 zstd 压缩写 `position_chunks`。

坐标继续使用 cs-demo-manager metadata：

```text
screenX = ((demoX - posX) / scale) * imageSize / radarSize
screenY = ((posY - demoY) / scale) * imageSize / radarSize
```

Nuke/Vertigo 按 `threshold_z` 切 upper/lower；未知地图显示明确空态，不能把点伪装到左上角。

`MatchViewer2D.vue` 使用 Canvas：播放/暂停/倍速/seek/回合/楼层/事件 overlay/tooltip/legend；ResizeObserver、RAF、worker 在卸载时全部释放。热力图按击杀、死亡、射击、道具、玩家、队伍、回合、阵营过滤，并通过 Rust 原生保存 PNG。

## 10. 测试和真实验收

### 自动化

新增 Rust 测试：worker claim/lease/concurrency/cancel/retry/panic/restart；watcher 三次稳定、`.part`、写入中、权限；round/identity；KAST/trade/opening/clutch/economy/utility；v6 migration；chunk encode/decode/坐标/双层地图。

新增前端测试：任务进度/取消/重试/stale request；经济/对枪/道具/事件 null fallback；图表表格一致；tabs ARIA/键盘；Canvas nonblank/seek/cleanup；四种 viewport；reduced motion。

执行：

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test -- --pool=threads --maxWorkers=1
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml --lib
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
git diff --check
```

### 新 BOT 闭环

用户真实启动本地 BOT，完成至少 5 个正式回合，包含炸弹和一种道具后退出 CS2。验收：Demo 连续三次稳定 -> Rust watcher 自动 enqueue -> worker progress -> 新 checksum/new demo id -> normalized 表非空 -> 新报告/UI 可读。中途点击取消，确认无半写入；retry 后完成。

### 真实 Tauri

使用真实窗口保存 library、overview、scoreboard、rounds、economy、duels、utility、viewer、heatmap 在四种尺寸的截图。检查无重叠、Canvas 非空、seek/暂停有效、切页后无 RAF/监听器泄漏。记录冷启动、core/spatial parse、query P95、viewer FPS 和内存；未测写 `null`。

## 11. 停止与发布边界

以下任一项出现，停止称候选：worker 仍同步、取消会写库、panic 拖垮应用、规范化表为空却声称指标完成、缺失值补 0、旧样本失败被宣称当前格式不兼容、viewer 空白/错层/泄漏、旧报告覆盖新 Demo、最小窗口重叠、自动化或真实 BOT/Tauri 失败。

只有 P0-P4、真实新 Demo、真实 Tauri、多 viewport、Canvas cleanup、性能和安装测试全部通过，才能写“功能候选”。本轮不 commit/push/release/deploy，除非用户另外明确授权；安装器最多构建 5 次。

最终报告必须列出：上游 commit/许可证、复用文件、worker 证据、normalized 行数、metric version、空间 chunk/Canvas 证据、每条命令 exit code、真实 Demo hash、真实 Tauri 截图、安装/Git/生产是否执行和剩余阻断项。
