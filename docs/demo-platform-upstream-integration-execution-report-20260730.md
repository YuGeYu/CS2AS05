# CS2 Demo 平台上游集成执行报告（2026-07-30）

## 结果

**未完成。**

本轮完成了固定上游、供应链归属、SQLite v5 基础迁移、checksum/path/job 基础数据结构、核心报告规范化入库、切片查询 API、warmup 回合修复、缺失指标诚实降级，以及录像库/总览/记分板/回合三个工作台标签。当前结果可作为后续开发基线，但不满足原方案 Phase 4-6 和第 12 节的完成门禁，因此未构建安装器、未 commit、未 push、未发布。

## 基线与权限边界

- 仓库：`E:\CS2AS05`
- HEAD：`8552b554993fec66866d2133d315342be3e0cbca`
- 分支：沿用当前分支，未新建分支。
- 保留了全部既有/用户改动；未执行 reset、restore、clean。
- 未执行 commit、push、GitHub Release、生产 updater 部署或安装器构建。

## 上游与供应链

### demoparser

- 上游：`LaihoE/demoparser`
- 固定 commit：`ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`
- `third_party/demoparser/UPSTREAM_COMMIT` 已落盘。
- `third_party/demoparser/SOURCE-MANIFEST.sha256` 包含 44 个源码条目。
- `third_party/demoparser/UPSTREAM.md` 记录上游归一化比对和本地下游差异。
- 仍直接编译 Rust path dependency；未引入第二解析内核或 sidecar。

### cs-demo-manager

- 上游：`akiver/cs-demo-manager`
- 固定 commit：`8961f5072fe4d42803dde68e8e71b3c90b216504`
- 许可证：MIT，完整文本位于 `third_party/cs-demo-manager/LICENSE`，归属已加入根 `NOTICE.md`。
- `PROVENANCE.md` 记录来源、固定 commit、获取日期和机械转换范围。
- 引入 95 个 PNG：51 个 radar、44 个 thumbnail。
- `map-manifest.json` 记录 upstream/downstream path、字节数和 SHA-256。
- `src/data/cs2-map-metadata.ts` 包含 44 条 CS2 地图元数据及坐标转换 helper。
- 未引入 Electron、PostgreSQL、Steam 账户、网络服务或 analyzer sidecar。

## 已实现

### SQLite v5 与任务状态

- v4 -> v5 前执行 WAL checkpoint，并用 `VACUUM INTO` 生成同目录备份。
- 新增 `checksum`、`demo_paths`、`analysis_jobs`、`matches`、`players`、`match_players`、`match_rounds`、`match_events`、`player_round_stats`、`round_economy`、`position_frames`。
- 旧 `report_json` 保留为兼容读取面；当前核心解析成功后同时写规范化表和兼容 JSON。
- 新增任务启动恢复、attempts、阶段/progress/error 字段和查询命令。
- 新增 `list_analysis_jobs`、`get_match_overview`、`get_match_scoreboard`、`get_match_rounds`。

当前任务实现仍是同步扫描路径上的基础状态记录，不是原方案要求的真正持久 worker、有界并发、取消和空间 pass 队列。

### 解析与数据诚实性

- parser adapter 升级为 `3`，report schema 升级为 `5`，指标版本保持 `simple-rating-v1`。
- `round_announce_match_start` 后清空暖场状态；完成回合只统计正式比赛开始后的 `round_end`。
- 未完成末回合仍保留实际观测 K/D/A/伤害，但 ADR、roundsPlayed、Rating 为 `null`/unavailable，避免错误分母。
- 同一真实 Demo 双解析排除 `parsedAt` 后 JSON 完全一致。
- 核心报告在单事务内刷新规范化 match/player/round/event 数据。

### UI

- 主窗口：`1440x900`，最小 `1100x700`。
- 战报窗口：`1280x800`，最小 `980x640`。
- Demo 页面改为全宽工作区，增加分析任务进度、总览/记分板/回合标签和方向键切换。
- web 预览截图：
  - `E:\CS2AS05\workspace\runtime\demo-platform-ui-20260730\demo-library-1440x900.png`
  - `E:\CS2AS05\workspace\runtime\demo-platform-ui-20260730\demo-library-1100x700.png`
- 2026-07-30 17:55 使用当前源码重新构建的真实 Tauri 调试实例完成一次“立即扫描”；窗口可交互且四个样本均回到已完成状态。该次真实 Tauri 操作未另存截图，以上文件只作为 web 布局证据，不能替代完整真实 Tauri 截图门禁。

## 真实数据库迁移与 writer 冲突闭环

数据库：

`C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover\demo-review\demo-review-v1.sqlite3`

迁移备份：

`C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover\demo-review\demo-review-v1.sqlite3.v4-1785402021642.bak`

- 备份 size：`114688` bytes
- 备份 SHA-256：`DC7F9A4F4AFFD57EA671822ED24AB503F4443C46B0509CB97B6DE69CC2E41319`
- 备份 `user_version=4`、`integrity_check=ok`，4/4 旧报告可读。

曾观察到规范化表已由 adapter 3 写入，但 `demo_files` 兼容列又被 adapter 2 的旧实例覆盖。复核时本项目只剩 Vite，没有 Tauri/Rust writer；旧实例已经退出，无法再从活动进程取证其精确启动路径。处理步骤：

1. 确认 DB 连续 12 秒 SHA-256 不变。
2. 重新构建 `E:\CS2AS05\src-tauri\target\debug\ai_pc_fac.exe`，时间戳 `2026-07-30 17:52:56`。
3. 只启动该调试实例并在真实 Tauri UI 触发一次“立即扫描”。
4. 四个 Demo 均回写为 `report_schema_version=5`、`parser_adapter_version=3`，兼容 JSON 内字段同为 5/3。
5. 退出该实例后 DB SHA-256 连续 10 秒保持 `4ECA17161EA25052E49F696B32759A548A9CD425EFEB7A018187D67FEFAD4EBA`。

历史 adapter 2 job 仍保留 4 条用于审计；adapter 3 job 为 4 条 `done/100`。权威 demo 行和 JSON 不再是 adapter 2。

最终 DB：

- `PRAGMA user_version=5`
- `PRAGMA integrity_check=ok`
- `matches=4`
- `players=28`
- `match_players=31`
- `match_rounds=34`
- `match_events=422`
- `player_round_stats=0`
- `round_economy=0`
- `position_frames=0`

## 真实 Demo 证据

样本：

`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260729-0958-de_dust2-advent.dem`

- Size：`32630115` bytes
- SHA-256：`C74963C77651D40A63EE835E713386EFB0CF67EEE8B3618163593DE063E485D5`
- Map：`de_dust2`
- Completed rounds：`5`
- Logical rounds：`6`，最后一回合无 `end_tick`
- Players：`10`
- Rating status：`unavailable`
- 单独真实测试输出：`map=Some("de_dust2") completed_rounds=5 logical_rounds=6 players=10 rating_status=unavailable`

其他现有样本已由 adapter 3 重解析：

- `de_inferno`：10 logical/completed reports，10 players，Rating complete。
- `de_cache` preview：17 rounds，10 players，Rating partial。
- `de_anubis` 旧短/异常样本：0 completed、1 logical、1 player，Rating unavailable；这是旧样本数据质量证据，不作为当前 CS2 格式失败结论。

未完成与 CS2 游戏内记分板/事件的 3 玩家、3 回合人工 oracle 抽查；未记录 core 峰值内存或性能门禁数据，因此这些指标为未测。

## 自动化

以下命令最终 exit code 均为 0：

- `npm run workspace:check`
- `npm run typecheck`
- `npm run lint`：oxlint 0 warning / 0 error，eslint 通过。
- `npm test -- --pool=threads --maxWorkers=1`：32 files，118 tests passed。
- `npm run build:web`：通过；保留 Three.js chunk `724.46 kB` 的既有 >500 kB warning。
- `cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check`
- `cargo check --manifest-path .\src-tauri\Cargo.toml`
- `cargo test --manifest-path .\src-tauri\Cargo.toml --lib`：45 passed，2 ignored。
- 真实 Demo ignored test：1 passed，双解析确定性断言通过。
- `cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings`：项目 crate 通过；vendored demoparser 保留 10 条上游既有 warning。
- `git diff --check`

Node 24 测试期间保留 `localStorage is not available because --localstorage-file was not provided` 的实验性 runtime warning，不影响断言结果。

## 阻断项与下一阶段

以下原方案功能尚未实现，因此本轮必须停在“未完成”：

- Rust 仍集中在 `services/demo.rs`，尚未按 catalog/jobs/parser/normalize/metrics/repository/export 边界拆分。
- watcher 稳定状态机、真正持久 worker、有界并发、任务取消、panic 隔离和自动重试闭环不完整。
- `player_round_stats` 与 `round_economy` 无数据；KAST、交易、首杀/首死、clutch、经济、完整 utility 指标未实现。
- 录像库筛选/排序/批量操作/导出、玩家详情和错误恢复 UI 未完成。
- 经济、对枪、道具标签未实现。
- spatial pass、压缩 position chunks、地图回放、双层地图、热力图、原生 PNG 保存未实现。
- 新 BOT watcher“稳定 -> 分析 -> 新报告”真实闭环未执行。
- 真实 Tauri 多 viewport、Canvas 像素/帧/cleanup、性能、安装覆盖/卸载/重装未验收。

下一阶段必须先补齐 Phase 1-2 的 worker/指标/规范化明细，再进入 Phase 4-5；不得用当前空表或占位标签提前构建安装器。

## Git 与发布

- commit：未执行
- push：未执行
- GitHub Release：未执行
- 生产部署/updater：未执行
- 安装器：未构建

