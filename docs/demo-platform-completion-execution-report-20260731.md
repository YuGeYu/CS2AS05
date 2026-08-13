# CS2 Demo 完整功能续作执行报告（2026-07-31）

## 结果

P0-P3 已完成本地实现与真实本机 Demo 回读；P4 的新 BOT 闭环、真实 Tauri 多 viewport、Canvas 像素/cleanup、性能与安装覆盖仍未验收，因此本轮不称“功能候选”，不 commit/push/release。

## 实现

- SQLite v6：v5 启动前执行 WAL checkpoint + `VACUUM INTO`；新增 lease/worker 字段、claim 索引、事件 source/quality、`position_chunks`、`map_metadata`。旧报告仍可读。
- P0 worker：导入/扫描只校验和入队；`BEGIN IMMEDIATE` claim，core 并发 `max(1,min(2,logical_cpu/2))`，spatial 单 worker；lease、取消、重试上限、单 job `catch_unwind`、过期恢复。
- watcher：300ms debounce；Rust 侧连续 3 次 750ms 稳定检查并确认最后写入至少 3 秒后入队；filesystem event 只做 UI 刷新。
- core：同一 parser pass 保留购买、伤害、开火、闪光、烟、燃烧、聊天等事件；规范化写入 player-round、KAST、开局、交易、经济质量和事件索引，缺失输入保留 null/unavailable。
- Tauri API：经济、对枪、道具、分页事件、玩家详情、CSV/JSON 原子导出、spatial enqueue/round positions/heatmap。
- UI：蓝色 Data-Dense Dashboard token；经济/对枪/道具/事件标签、取消/重试、导出、玩家详情 drawer；Canvas 2D 回放/热力图支持回合、4/8/16Hz、楼层、播放/seek、ResizeObserver/RAF cleanup。
- 本轮补齐 P4 前置 API：`save_heatmap_png` 由 Rust 生成 1024x1024 RGBA PNG 并使用 `.part` 原子替换；`launch_demo_at_tick` 仅从已索引 Demo 路径启动 Steam/CS2 并限制 tick 范围，未使用浏览器下载或任意路径。

## 真实数据库证据

数据库：`C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover\demo-review\demo-review-v1.sqlite3`

- `PRAGMA user_version=6`
- `PRAGMA integrity_check=ok`
- v5 迁移备份：`demo-review-v1.sqlite3.v5-1785484311538.bak`，`286720` bytes
- 4 个历史 Demo 均重算为 `metrics-v2`，job 全部 `done`
- 规范化计数：`player_round_stats=320`、`round_economy=64`、`match_events=7786`
- dust2 spatial job：`spatial_done=100`；`position_chunks=14`，压缩 payload `610511` bytes，tick `896..32400`

## 自动化

- `cargo test --manifest-path .\\src-tauri\\Cargo.toml --lib`：46 passed，2 ignored（vendored demoparser 保留 10 条既有 warning）
- `npm test -- --pool=threads --maxWorkers=1`：32 files / 118 tests passed
- `npm run typecheck`、`npm run lint`、`npm run build:web`：通过
- `git diff --check`：通过
- 本轮新增 `cargo check --manifest-path .\\src-tauri\\Cargo.toml`：exit 0；新增 `png 0.17.16` 依赖，告警仍仅来自 vendored demoparser 的既有 10 条 warning。
- 本轮新增命令已纳入 `src-tauri/src/lib.rs` 与 `src/services/tauri/demo.ts`，`npm run typecheck`、`npm run lint`、`cargo fmt -- --check` 均通过。

## 未验收边界

真实 CS2 BOT 录制 5 回合闭环、真实 Tauri 多 viewport/安装重装、Canvas 像素截图与性能峰值仍需用户在目标机器执行；本机未安装 Playwright Chromium，未伪造截图证据。当前不构建安装器、不发布。
- `map_metadata` 表已创建但当前数据库尚未填充完整上游地图元数据；热力图 PNG 当前按真实事件点的坐标范围绘制，完整地图投影/事件过滤仍需真实验收。
