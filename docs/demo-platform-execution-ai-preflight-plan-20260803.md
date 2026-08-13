# CS2 Demo 功能：执行 AI 独立预验收方案（2026-08-03）

> **续作索引（2026-08-03）：** 本方案阶段 A-C 和大部分自动化已经完成。当前事实见 `demo-platform-execution-ai-preflight-report-20260803.md`；下一位执行 AI 不得重跑已完成范围，应直接执行 `demo-platform-execution-ai-preflight-closeout-plan-20260803.md`，只关闭最终 EXE 四尺寸、cold start、positions decode P95、viewer FPS/CPU、20 次生命周期和 DB 静止性门禁。

> 交接对象：下一位“实际执行 AI”。
>
> 本文只包含在要求用户进入真实 CS2 之前，执行 AI 必须独立完成的工作。用户的 5 回合 BOT 录制、游戏内记分板对照、真实 `launch_demo_at_tick` 体验确认和安装后主观确认不在本阶段执行。两个 AI 不共享上下文，因此本文包含完整基线、文件、契约、命令、证据、停止条件和交接格式。

## 1. 阶段目标

执行 AI 使用现有历史 Demo 和本机 Tauri 环境，独立完成以下闭环：

1. 将固定 `cs-demo-manager` 上游的 44 条地图元数据和雷达资产可靠地带入 Rust、SQLite 和最终安装资源。
2. 将 `save_heatmap_png` 从事件点包围盒红点图升级为基于真实雷达图、真实地图坐标、楼层和筛选条件的 1024×1024 PNG。
3. 自动证明 2D viewer/heatmap Canvas 非空、坐标在合理范围、播放推进、暂停和组件卸载后 RAF/ResizeObserver/image callback 已释放。
4. 使用真实 Tauri 历史数据库完成四种窗口尺寸截图和像素检查，不把 Web 预览当作 Tauri 证据。
5. 采集冷启动、core/spatial 任务、SQLite 查询、Canvas 帧率、峰值内存和退出后数据库稳定性；测不到的字段保留 `null`。
6. 产出一个可让用户直接进行单次 BOT 验收的调试候选、明确启动路径和最短用户操作清单。

本阶段不新增 Demo 产品功能，不继续扩展指标，不构建正式发布包，不 commit/push/release/deploy。

## 2. 当前权威基线

执行前读取并保存：

```powershell
Set-Location E:\CS2AS05
git status --short
git rev-parse HEAD
git diff --stat
Get-Content -Raw .\docs\demo-platform-completion-execution-report-20260731.md
```

当前必须保留的事实：

- 工作树包含大量未提交 Demo 改动，禁止 `reset`、`restore`、`clean`，禁止新开分支。
- `LaihoE/demoparser` 固定 commit：`ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`。
- `akiver/cs-demo-manager` 固定 commit：`8961f5072fe4d42803dde68e8e71b3c90b216504`。
- SQLite 当前为 v6，`integrity_check=ok`；已有 core/spatial worker、lease/cancel/retry、规范化指标和历史 Demo 数据。
- 当前真实数据库有 `player_round_stats=320`、`round_economy=64`、`match_events=7786`；dust2 有 14 个 spatial chunks、610511 bytes 压缩数据。
- 前端 `src/data/cs2-map-metadata.ts` 已有 44 条机械转换元数据；`src/assets/maps/cs2/radars` 和 `thumbnails` 已有 95 个 PNG。
- SQLite `map_metadata` 表已创建，但当前本机数据库没有完整 seed。
- `MatchViewer2D.vue` 已使用真实雷达图和 `scaleDemoCoordinate()`；`save_heatmap_png` 仍按事件点 min/max 归一化，不使用地图元数据和雷达背景。
- 当前 `get_heatmap_points` / `save_heatmap_png` API 只接收 `kind`，没有 round/player/layer 完整筛选契约。
- 当前 Canvas cleanup 代码存在，但没有专门的组件测试和真实 Tauri 像素证据。
- `launch_demo_at_tick` 已限制已索引 Demo 和 tick 范围；本阶段只做命令构造/路径安全自动化，不实际替用户启动 CS2 验收。

## 3. 执行纪律

- 每个阶段先写失败测试，再修改实现，再只运行相关测试，阶段结束才运行全套。
- 当前运行行为、数据库和生成 PNG 是权威证据；注释或计划不能覆盖运行结果。
- 不删除历史数据库、迁移备份、Demo、用户设置或既有 evidence。
- 新 evidence 写入新目录：

```text
E:\CS2AS05\workspace\release-evidence\demo-preflight-20260803-<HHmmss>\
```

- 每条关键命令保存 stdout、stderr、exit code、开始/结束时间和耗时。
- 不用浏览器下载、`window.open`、File System Access API 或浏览器确认框完成应用功能。
- Playwright/浏览器仅用于测试 Web UI；真实 Tauri 证据必须来自真实 WebView2 窗口。
- 不为通过门禁修改 vendored demoparser 的 10 条既有 warning。

## 4. 阶段 A：建立单一地图元数据来源

### 4.1 当前问题

`scripts/import-cs-demo-manager-maps.mjs` 只生成 TypeScript metadata 和 map manifest。Rust 不读取该 TypeScript，SQLite `map_metadata` 因此为空。Rust PNG 导出也无法定位或校验雷达资源。

### 4.2 目标文件

修改或新增：

```text
scripts/import-cs-demo-manager-maps.mjs
src/data/cs2-map-metadata.ts
src-tauri/resources/demo-maps/cs2-map-metadata.json
src-tauri/resources/demo-maps/radars/*.png
third_party/cs-demo-manager/map-manifest.json
src-tauri/tauri.conf.json
src-tauri/src/demo/map_metadata.rs
src-tauri/src/services/demo.rs
src-tauri/src/models/demo.rs
```

### 4.3 生成链

以 `akiver/cs-demo-manager@8961f507...` 的 `default-maps.ts` 和 `static/images/maps/cs2` 为唯一源。扩展导入脚本，一次生成：

1. 现有 `src/data/cs2-map-metadata.ts`。
2. Rust 可反序列化的 `src-tauri/resources/demo-maps/cs2-map-metadata.json`。
3. 供最终 Tauri 读取的 `src-tauri/resources/demo-maps/radars/*.png`。
4. 更新 manifest，分别记录前端资产、Tauri 资源、metadata JSON/TS 的 path、bytes、SHA-256。

JSON 每条必须包含：

```json
{
  "name": "de_dust2",
  "positionX": -2476,
  "positionY": 3239,
  "scale": 4.4,
  "thresholdZ": 0,
  "radarSize": 1024,
  "radarAsset": "de_dust2.png",
  "radarSha256": "...",
  "lowerRadarAsset": null,
  "lowerRadarSha256": null,
  "upstreamCommit": "8961f507..."
}
```

双层地图的 lower asset/sha 必须存在；没有 lower PNG 的地图保持 `null`，禁止自动指向 upper 图。

`tauri.conf.json` 把 `resources/demo-maps` 加入 bundle resources。开发版、`cargo test` 和最终 bundle 的资源解析分别测试，禁止仅依赖仓库相对路径。

### 4.4 SQLite v7

不要破坏 v6。新增 `PRAGMA user_version=7`：

- 迁移前 WAL checkpoint + `VACUUM INTO` 生成 `.v6-<timestamp>.bak`。
- 为 `map_metadata` 增加 `radar_asset`、`lower_radar_asset`、`lower_asset_sha256`、`radar_size`；已存在列先检测。
- `initialize()` 从编译期 `include_str!` 的 metadata JSON 做幂等 upsert，避免启动依赖外部网络或 TypeScript。
- seed transaction 后必须得到 44 行；每行 `upstream_commit`、asset hash 和数值字段完整。
- metadata 版本变化时 upsert；不能删除用户未来添加的非上游地图，建议增加 `source='cs-demo-manager'` 后只更新同 source rows。

若不希望增加 user_version，也必须提供同等的列检测和备份证据；推荐 v7，避免“数据库结构变了但版本未变”。

### 4.5 阶段 A 测试

- 导入脚本连续运行两次，输出文件 SHA-256 完全一致。
- TS 与 JSON 都是 44 个唯一地图，字段逐项一致。
- manifest 中所有文件存在、size/hash 匹配。
- dust2 `(-2476,3239)` 映射 `(0,0)`。
- nuke `-496 -> lower`、`-495 -> upper`；vertigo `11699 -> lower`、`11700 -> upper`。
- 未知地图返回明确 unavailable，不映射左上角。
- v6 真实备份 `integrity_check=ok`；v7 DB `map_metadata=44`。

阶段 A Gate：数据库 44 条、资源 hash、前后端公式一致、开发/测试资源路径全部通过，否则不得进入 PNG 导出。

## 5. 阶段 B：真实雷达热力图 PNG

### 5.1 当前问题

`save_heatmap_png()` 当前计算事件点 `min_x/max_x/min_y/max_y`，按自身包围盒拉伸到 1024×1024，只写红色圆点。该图片无法与 CS2 雷达位置对齐，也没有地图背景、楼层或完整筛选。

### 5.2 API 契约

新增统一 `HeatmapFilters` DTO，替换单一 `kind`：

```text
kind: allowlist event kind
roundNumbers: 0..N，可空
playerKeys: stable keys，可空
teamNumbers: 2/3，可空
layer: upper | lower | all
radius: 4..64，默认 18
opacity: 0.05..1.0，默认 0.72
```

修改：

```text
get_heatmap_points(demo_id, filters)
save_heatmap_png(demo_id, filters, destination_path)
```

Rust、`src/types/demo.ts`、`src/services/tauri/demo.ts`、`MatchViewer2D.vue` 共用同一字段。后端严格 allowlist 和范围校验；查询使用绑定参数，禁止拼接用户字符串。

`HeatmapPoint` 增加 `z`、`teamNumber`、`tick`。如果指定 layer 但事件没有 z，写入 quality/warning 并排除，不能默认为 upper。

### 5.3 坐标和背景

Rust 新增与上游一致的纯函数：

```text
pixel_x = ((demo_x - pos_x) / scale) * output_size / radar_size
pixel_y = ((pos_y - demo_y) / scale) * output_size / radar_size
layer = z < threshold_z ? lower : upper
```

必须使用 SQLite/embedded metadata 对应地图；删除事件 bbox 归一化路径。输出逻辑：

1. 从 `matches.map_name` 获取 map。
2. 查询 `map_metadata`，校验 upstream commit 和 radar hash。
3. 从 Tauri resource dir 读取对应 upper/lower radar PNG。
4. 用 `png` decoder 统一转换为 RGBA8；拒绝损坏或 hash 不匹配资源。
5. 将事件点变换到雷达像素；非有限值、越界点计数但不写入。
6. 使用稳定 radial kernel 累加浮点密度；按最大密度归一化，避免后画点覆盖先画点。
7. 用蓝色 SaaS 视觉下仍可辨识的 heat ramp，例如透明蓝 -> 黄 -> 红；与雷达做 alpha composite。
8. 输出 1024×1024 RGBA PNG 到同目录 `.part`，flush/close 后原子替换目标。

不在 PNG 内绘制大段说明文字。API 返回扩展 `ExportMatchResult`：path、bytes、width、height、mapName、layer、inputPoints、renderedPoints、discardedPoints、assetSha256、outputSha256。

目标文件已存在时，Windows 上不能先删除再裸 rename 造成中间缺失。实现同目录临时文件 + Windows 可行的 replace 策略；失败清理 `.part`，旧目标保持完整。

### 5.4 事件来源

- death/fire 使用 actor 或事件位置；utility 使用投掷/爆炸位置。
- 每类事件在固定样本中先检查 payload 的 x/y/z 路径，不能假设所有 event schema 相同。
- 若某类事件没有坐标，`get_heatmap_points` 返回空并提供明确错误/quality，不从玩家 spatial 最近邻偷偷补值，除非增加版本化且有测试的关联算法。
- 筛选结果为空时保持现有 `DEMO_HEATMAP_EMPTY`。

### 5.5 阶段 B 自动化

- 使用 dust2 metadata 和合成点验证四角/中心映射，不依赖肉眼。
- 生成 PNG 后用 decoder 回读：1024×1024、RGBA8、非透明像素、背景与原 radar 有高比例一致区域、热点附近存在色差。
- 输入点顺序打乱，输出 hash 应相同。
- 越界/NaN/Inf 被丢弃并计数。
- nuke/vertigo upper/lower 使用不同资源；未知地图、缺 lower、hash 不匹配明确失败。
- 空数据不留下 `.part`；覆盖失败时旧输出 hash 不变。
- round/player/team/layer 筛选分别只改变预期点集合。

阶段 B Gate：输出不再使用 bbox 缩放；dust2 PNG 含真实 radar 背景和正确热点；双层、过滤、原子替换全部通过。

## 6. 阶段 C：Canvas 组件自动化和资源释放

### 6.1 重构可测试边界

从 `MatchViewer2D.vue` 提取纯函数：

```text
src/features/demo/viewer/coordinates.ts
src/features/demo/viewer/frame-index.ts
src/features/demo/viewer/draw-viewer.ts
src/features/demo/viewer/draw-heatmap.ts
src/features/demo/composables/useViewerPlayback.ts
```

Vue 组件只负责 props/state/API/lifecycle。坐标函数要么从 `src/data` re-export，要么只有一个实现；禁止 Rust、TS、Canvas 各有不同公式。

### 6.2 修正已见风险

当前 `animate()` 即使到末尾把 `playing=false`，函数末尾仍会再 schedule 一帧。重构后要求：

- 只有 `playing=true` 且未到末帧才 schedule 下一帧。
- 切 round、sampling、mode、demoId、tab hidden 时停止播放并 cancel RAF。
- `document.visibilitychange` 隐藏时暂停；恢复不自动播放。
- 每次 load 使用 request generation/AbortController 语义，旧响应不得覆盖新 round。
- radar `onload/onerror` 都清理；更换 URL 时旧 Image callback 不得触发新 canvas draw。
- ResizeObserver 只 observe 一次，卸载 disconnect。
- devicePixelRatio 变化或窗口 resize 后保持 1:1 比例和清晰像素。
- 未知 map、缺 radar、无 spatial、job queued/error 分别有可读 UI，不留黑色空画布冒充成功。

### 6.3 组件测试

新增 `tests/demo-viewer.spec.ts` 或同类：

- mock `requestAnimationFrame/cancelAnimationFrame`，播放 N 帧后 tick 正确；末帧 callback 集合为 0。
- pause、round change、sampling change、unmount 后 callback 集合为 0。
- mock ResizeObserver，unmount 调用 disconnect。
- 快速切两个 round，第二请求先返回，最终只显示第二 round。
- radar load/error 与未知地图状态。
- reduced motion 下没有页面位移/热力渐入；播放功能仍可由按钮逐帧使用。
- controls 的 44px、aria-label、range label、tab/focus 不回归。

### 6.4 浏览器像素测试

执行 AI 安装 Playwright Chromium；网络慢时等待，不反复删除缓存。若 Chromium 下载失败，优先使用本机 Edge channel，不让用户安装。建立只在 dev/test 可用的 deterministic viewer fixture，禁止进入 production bundle。

在 1440×900、1100×700、1280×800、980×640：

- 截取 viewer/heatmap；
- `canvas.getContext('2d').getImageData()` 统计非透明像素和颜色数；
- 雷达不能为纯黑/纯色；玩家/热点区域相对背景有差异；
- 控件、canvas、range 不重叠；长 map/file/player 文本不溢出。

Web 证据只证明组件布局/像素，不代替下一阶段真实 Tauri。

阶段 C Gate：组件 lifecycle 测试通过，四 viewport web 像素非空，无 pending RAF/observer。

## 7. 阶段 D：真实 Tauri 四尺寸预验收

### 7.1 启动纪律

- 先检查正在运行的 `ai_pc_fac.exe`、`CS2BotImproverAssistant.exe` 和 Vite；记录 PID、path、start time。
- 只终止执行 AI 本轮启动的实例，不结束来源不明的用户进程。
- 重新构建 debug executable，记录 absolute path、size、SHA-256、mtime。
- 启动单一 debug Tauri，确认写入数据库的 adapter/schema/metrics 与源码一致，排除旧 writer。

### 7.2 历史数据准备

使用现有 dust2 历史 Demo 和已有 spatial chunks，不要求用户进入 CS2。启动前保存 DB copy/hash/integrity；启动后验证：

- `user_version=7`、`map_metadata=44`；
- dust2 `matches.map_name=de_dust2`；
- spatial job done、position chunks 非空；
- UI 能打开 viewer/heatmap；
- 本阶段操作不触发全部历史 Demo 无条件重算。

### 7.3 四种尺寸

真实主窗口：1440×900、1100×700。真实战报/相关窗口：1280×800、980×640。使用 Tauri API、Windows UI automation 或受控 Win32 `SetWindowPos` 调整窗口，不手改生产默认值来逐个测试。

每个尺寸保存：

```text
library.png
overview.png
scoreboard.png
rounds.png
economy.png
duels.png
utility.png
events.png
viewer.png
heatmap.png
```

截图必须来自真实 Tauri 窗口，记录窗口 rect、DPI、display scale。用图像脚本检查：截图非空、canvas 区域不是纯色、没有大面积透明/黑屏、热点/玩家像素存在。人工检查 text/button overlap、滚动区域、sticky header、drawer 和 focus。

### 7.4 Canvas cleanup 的运行证据

在真实 Tauri 中：

1. 打开 viewer，播放 10 秒，记录 CPU/帧推进。
2. 暂停 10 秒，CPU 应显著下降且 tick 不变。
3. 切到非空间 tab 10 秒，确认不再绘制/拉取位置数据。
4. 重复进入/离开 viewer 20 次，记录 WorkingSet 和 handle count；不能单调无界增长。
5. 退出应用，确认进程、worker thread、文件句柄全部结束。

阶段 D Gate：四尺寸真实 Tauri 截图齐全，Canvas 非空，暂停/切页 cleanup 和 20 次循环无明显泄漏。

## 8. 阶段 E：性能与稳定性证据

### 8.1 新增测量脚本

新增 `scripts/measure-demo-preflight.ps1`，只读或可逆操作，输出 JSON/CSV：

- build/exe/git commit/source diff fingerprint；
- process PID/start/exit、WorkingSet64 peak、PrivateMemory peak、HandleCount peak、CPU time；
- DB path/user_version/integrity/size/WAL size/hash；
- job created/started/finished、core/spatial elapsed；
- library query、round positions decode、heatmap query/PNG export elapsed；
- viewer active/pause/tab-away 的 process samples；
- screenshot path/hash/dimensions。

脚本失败不得修改 DB 或删除 evidence。无 sqlite CLI 时使用项目内 Rust diagnostic test或已有可用结构化工具，不要求用户安装数据库软件。

### 8.2 指标门禁

沿用续作方案目标，但本轮记录真实值优先：

- 冷启动到可交互目标 `<=2.5s`；无法自动确定时为 `null`，附测量原因。
- 500 Demo library query P95 `<=150ms`；当前不足 500 条时使用事务内临时 fixture DB，不污染用户 DB。
- 单 round positions decode + IPC P95 `<=100ms`。
- viewer 目标 60 FPS，低配可接受稳定 30 FPS；同时记录 DPI/viewport/points。
- 暂停/切页后 RAF 为 0（组件测试）且真实进程 CPU 回落。
- 20 次 viewer 循环没有持续线性内存/handle 增长。

不要用上游 Linux benchmark替代本机数据，不估算未测项。

### 8.3 DB 稳定性

关闭执行 AI 启动的所有 Tauri writer 后：

```text
连续 10 秒，每 2 秒计算一次 DB/WAL/SHM 状态
```

要求主 DB hash 稳定、`integrity_check=ok`、无 active/expired lease 异常。若 WAL checkpoint 导致合理 hash 变化，记录时间线并在最终静止点重新取证，不能只报一次 hash。

阶段 E Gate：性能 JSON 完整，所有未测为 null，数据库退出稳定，无证据冲突。

## 9. 阶段 F：自动化总门禁和用户候选准备

### 9.1 全套命令

依次运行并保存输出：

```powershell
Set-Location E:\CS2AS05
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
git status --short
```

`clippy` 若仍只有 vendored demoparser 10 条既有 warning，记录精确路径；项目 crate 新 warning 必须修。Three.js chunk 等既有 build warning 单独记录，不能伪装为 0 warning。

### 9.2 `launch_demo_at_tick` 自动化边界

不在本阶段真实启动用户的 CS2。把命令构造提取为纯函数并测试：

- path 只来自 database id；不存在/源文件消失失败。
- tick `0`、`10_000_000` 允许，`-1`、`10_000_001` 拒绝。
- player key 空允许，非空返回 `DEMO_PLAYER_UNSUPPORTED`。
- path 含空格/中文时作为单独 argument，不发生 shell 拼接。
- Steam 不存在返回明确错误。

真实启动和跳 tick 留给用户阶段。

### 9.3 交给用户的调试候选

本阶段只构建 `src-tauri/target/debug/ai_pc_fac.exe` 或项目当前正确 debug binary，不构建正式 installer。记录：

```text
absolute path
size
SHA-256
mtime
source HEAD
git diff fingerprint
DB backup path/hash
evidence directory
```

退出所有执行 AI 启动的实例，确认 DB 静止。给用户的下一步只包含 UI 操作，不要求其运行命令：启动候选、开启自动录像、打 5 回合、制造炸弹/道具事件、正常退出、等待新报告、点击 tick 跳转。

## 10. 明确不属于本阶段

- 用户真实 BOT 五回合录制。
- 用户游戏内记分板截图和三玩家/三回合 oracle。
- 真实验证 Steam/CS2 已启动并跳到正确 tick。
- 正式签名 installer、覆盖安装、卸载/重装。
- commit、push、GitHub Release、R2/D1、生产 updater。
- 新指标、新页面、新网络服务或视频功能。

执行 AI 不得以“用户尚未操作”为理由跳过 A-F；A-F 完成后才向用户请求一次真实游戏验收。

## 11. 停止条件

出现任一项，停止交付用户候选并写阻断报告：

- metadata 不是 44 条或 TS/JSON/SQLite/manifest 不一致。
- radar 资源缺失/hash 不一致，开发可用但 packaged resource 路径不可用。
- PNG 仍使用事件 bbox 缩放、没有真实地图背景、楼层或筛选无效。
- 未知地图/缺 lower 被伪装为成功。
- Canvas 空白、纯色、坐标集体越界、末帧仍持续 RAF、卸载未 disconnect。
- Web 截图冒充真实 Tauri，或四尺寸存在文字/按钮重叠。
- 20 次进入/离开 viewer 内存或 handle 持续线性增长。
- 启动旧 writer 覆盖 adapter/schema/metrics，或退出后 DB hash 不稳定。
- 用户数据库迁移/备份/integrity 失败。
- 自动化、项目 clippy、build 任一失败。
- staged/evidence 包含用户 Demo、DB、密钥、日志或未知二进制。
- 用户要求停止。

## 12. 执行 AI 最终报告模板

```text
结果：AI 预验收未完成 / AI 预验收通过，可请求用户 BOT 验收

工作树：HEAD、status、diff fingerprint、未回退说明
地图：upstream commit、TS/JSON/SQLite 数量、asset/hash、v7 backup/integrity
热力图：map/layer/filters、输入/渲染/丢弃点、PNG path/hash/dimensions
Canvas：组件测试、RAF/observer cleanup、web/Tauri 像素检查
Tauri：debug EXE path/hash、四 viewport 截图、窗口 rect/DPI
性能：cold start、query P95、decode P95、FPS、peak memory、handle trend
数据库：before/after/final hash、WAL、integrity、jobs/leases
自动化：每条命令 exit code、warning 归属
未执行：BOT 游戏、真实 tick 跳转、installer、commit/push/release
用户下一步：只列一次真实游戏验收的 UI 操作
证据：绝对路径
```

只有地图、PNG、Canvas、真实 Tauri、性能、数据库和全套自动化全部过门禁，最终结果才写“AI 预验收通过”。这仍不是“功能候选”；用户完成真实 BOT/CS2 验收后才能进入安装候选阶段。
