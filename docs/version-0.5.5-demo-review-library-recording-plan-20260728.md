# CS2AS05 0.5.5 Demo 对局复盘、自动录制与退出后战报实施方案

> 文档日期：2026-07-28
> 交接对象：下一位实际执行 AI
> 本文是独立实施依据，不依赖此前对话上下文。
> 本轮仅制定方案，未修改运行代码、未构建、未发布。
> 版本约束：所有应用版本号继续保持 `0.5.5`，不得升级为 `0.5.6`。

## 1. 最终决策摘要

本次大功能正式命名为 **Demo 对局复盘**，主导航显示 **对局复盘**。页面内分为两个一级视图：

- **录像库**：添加扫描目录、手动导入 Demo、扫描、搜索、筛选、排序、解析状态和错误恢复。
- **对局报告**：展示比赛摘要、记分板、回合时间线、击杀和炸弹事件、玩家明细。

采用以下技术路线：

1. Demo 扫描、目录监听、文件稳定性判断、解析、SQLite 持久化全部在 Tauri/Rust 后端完成，前端不直接读取本机文件，不依赖浏览器 File System API。
2. Demo 解析内核使用 `LaihoE/demoparser` 固定提交 `ba39cc44cd5abfd7f34df2b3c0a7dd3630048311` 的 Rust core，以 path dependency 方式 vendor；不引入 Python、Node/N-API sidecar。
3. 数据库存放于 Tauri app data 目录，使用 `rusqlite` 的 bundled SQLite。扫描、分页、任务恢复、会话去重均以数据库为权威状态。
4. “自动录制 Demo”只作用于助手启动的 BOT/本地托管对局。启动前向两个受管 cfg 写入 `tv_enable 1` 与 `tv_autorecord 1`；官方匹配服务器不会因此被客户端强制开启 CSTV 录制。
5. 启动 CS2 前建立会话基线；检测到 CS2 从运行变为停止后，等待新 Demo 停止增长，再解析本次会话新增文件并在应用内切换到最新战报。不能简单取磁盘上 mtime 最大的旧文件。
6. 第一版只做可靠的基础战报，不做 AI 锐评、OBS、FFmpeg、视频剪辑、2D 全地图轨迹、热力图和高光视频录制。

## 2. 已核对的当前项目事实

当前工作区为 `E:\CS2AS05`，技术栈为 Vue 3 + TypeScript + Pinia + Tauri 2 + Rust，Windows 为主要运行平台。`package.json`、`src-tauri/Cargo.toml` 和 Tauri 配置当前版本均为 `0.5.5`。

### 2.1 当前入口和进程状态链

- 主导航：`src/components/AppShell.vue`
- 现有视图：`overview / presets / items / knives / commands / install`
- CS2 进程轮询：`src/composables/useCs2ProcessPolling.ts`
- CS2 状态 store：`src/stores/cs2.ts`
- 启动体验：`src/composables/useCs2LaunchExperience.ts`
- 前端启动 IPC：`src/services/tauri/panel.ts`
- Rust command：`src-tauri/src/commands/panel.rs`
- Rust 启动实现：`src-tauri/src/services/panel.rs::launch_cs2`

当前 `useCs2ProcessPolling` 每 10 秒刷新一次，并在窗口获得焦点或重新可见时刷新。`cs2` store 只覆盖当前 `checking/running/stopped/unknown`，没有保存上一状态，也没有“本次是否由助手启动”的会话信息。退出后自动解析不能只在现有 composable 中比较两个临时值；需要专属的 Demo 会话协调器持久化状态并处理边沿。

### 2.2 当前 cfg 管理基础

`src-tauri/src/services/panel.rs` 已管理：

- `game/csgo/cfg/my_bot_normal_config.cfg`
- `game/csgo/cfg/my_bot_ffa_config.cfg`
- `game/csgo/cfg/cs2as05-panel-state.json`

已有 `panel_transaction`、`replace_cfg_line`、`atomic_write`、CS2 运行中拒绝修改、双 cfg 同步写入和状态持久化模式。自动录制设置必须复用这些事务和写入基础，不得另写一套直接覆盖 cfg 的代码。

### 2.3 当前可复用 UI 和依赖

- 已安装 `tauri-plugin-dialog`，目录选择应通过 Tauri dialog，不使用 `<input webkitdirectory>`。
- 已使用 Lucide 图标，可为新导航使用 `ChartNoAxesCombined`，备选 `Clapperboard`。
- 应复用现有 `ToggleSwitch.vue`、toast、按钮、表单和状态色。
- `README.md` 当前明确写着“不提供 Demo 管理”，功能落地后必须同步修订，避免文档与产品冲突。

## 3. 参考项目调查与许可证边界

### 3.1 Local-Arena

- 仓库：<https://github.com/numakkiyu/Local-Arena>
- 调查固定提交：`fad7e4ab7441f6bb95ebe9f6186169dc424ff008`
- 许可证：AGPL-3.0
- 本地只读研究副本：`E:\CS2AS05\workspace\research\Local-Arena`
- 关键代码：`Panel/src-tauri/src/match_system.rs`、`Panel/src/panels/MatchHistoryPanel.tsx`、`Panel/src/panels/MatchResultView.tsx`

Local-Arena 的核心价值是受控比赛生命周期，而不是通用 `.dem` 解析器。应学习：

- `Prepared / Launching / Loading / Warmup / Live / Finished / Interrupted` 的会话状态边界。
- `pending / recording / validating / ready / failed / disabled` 的 Demo 状态。
- 固定 session ID、request/result JSON、完成事件和结果 fingerprint 去重。
- Demo 结束后每 250ms 采样，连续稳定后才读取。
- 最小文件大小检查和 `PBDEMS2` / `HL2DEMO` header 检查。
- 对受控路径做 canonicalization，拒绝目录穿越。

可以参考其行为设计。若直接复用代码，必须保留 AGPL 署名和对应源码义务；本方案没有要求复制其 UI 或比赛系统代码。

### 3.2 CS2-insight-agent

- 仓库：<https://github.com/DrEAmSs59/CS2-insight-agent>
- 调查固定提交：`41819fc2e3188c9a3c0baa413302a1821cefbfe1`
- 许可证：PolyForm Noncommercial 1.0.0
- 本地只读研究副本：`E:\CS2AS05\workspace\research\CS2-insight-agent`
- 关键代码：`backend/app/demo_watcher.py`、`backend/app/demo_db.py`、`backend/app/demo_parse_isolation.py`、`frontend/src/pages/DemoLibraryPage.jsx`、`frontend/src/components/demoLibrary/*`

**可以复制该项目代码、样式或派生文件到当前 AGPL 项目。** 学习产品行为和数据边界：

- 多目录监听、有限扫描深度、按新到旧发现。
- `.dem` / `.zip` 类型过滤、size + mtime 稳定检测。
- create/modify/move 事件统一去重。
- pending/parsing/done/error 状态恢复。
- 搜索、地图/状态筛选、排序、分页、详情和回合时间线。
- 解析失败隔离，单个坏 Demo 不拖垮整个服务。

本期不照搬其高光提取、OBS、FFmpeg、AI、视频编排和 2D 回放功能。

### 3.3 LaihoE/demoparser

- 仓库：<https://github.com/LaihoE/demoparser>
- 固定提交：`ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`
- 上游声明许可证：MIT；固定提交中的 `LICENSE` 必须原样保留并在合入前再次核对上游归属信息。
- 本地研究副本：`E:\CS2AS05\workspace\research\demoparser`

该项目提供原生 Rust parser core：

- `src/parser`
- `src/csgoproto`

Rust package 名是通用的 `parser`，版本 `0.1.1`，未作为稳定 parser crate 正式发布。npm 的 `@laihoe/demoparser2` 是 N-API native addon，Tauri WebView 没有 Node runtime，不能从前端直接加载。

推荐 vendor 结构：

```text
third_party/demoparser/
  UPSTREAM.md
  LICENSE
  parser/
  csgoproto/
```

`UPSTREAM.md` 必须记录仓库 URL、固定 commit、获取日期、原始目录、是否有本地补丁。`NOTICE.md` 增加依赖说明。Cargo 使用明确别名，避免业务代码到处出现含糊的 `parser` 名：

```toml
cs2-demoparser = { package = "parser", path = "../third_party/demoparser/parser" }
```

若路径相对 `src-tauri/Cargo.toml` 不正确，应按最终 vendor 位置调整。原始 vendor 和本项目适配层分开：不要直接把业务聚合逻辑塞进第三方源码。

## 4. 录制指令核对与边界

用户记忆的方向正确：

```cfg
tv_enable 1
tv_autorecord 1
```

项目历史 release note 也曾使用这组指令，并提及 `game/csgo/replays`。实际实现必须遵守以下事实：

1. 该组合面向玩家本机托管、BOT 或离线服务器的 CSTV 自动录制。
2. 它不能让官方匹配、5E、完美或 FACEIT 的远端服务器因为客户端 cfg 而开启录制。
3. `tv_enable` 可能需要在地图/服务器创建前生效，因此必须在助手启动 BOT 模式前写好，不要进入地图后才临时执行。
4. 生成目录和文件名会受 CS2 当前版本、启动方式和服务器行为影响。第一版必须做真实 Windows + 当前 CS2 验证，不能硬编码断言“只会在 replays”。

### 4.1 设置文案

概览页 BOT 启动区域附近增加：

- 开关标题：`自动录制本地对局 Demo`
- 开启说明：`助手启动 BOT/本地对局前写入 CSTV 自动录制设置；CS2 退出后尝试打开本次战报。`
- Online 模式说明：`官方匹配是否提供 Demo 由服务器或平台决定，此开关不会让远端服务器开始录制。`

不要把开关描述成“录制所有 CS2 比赛”。

### 4.2 cfg 写入协议

两个受管 cfg 均使用唯一 marker block：

```cfg
// CS2AS05 DEMO RECORDING BEGIN
tv_enable 1
tv_autorecord 1
// CS2AS05 DEMO RECORDING END
```

关闭时仍保留 block 并写入 `0`：

```cfg
// CS2AS05 DEMO RECORDING BEGIN
tv_enable 0
tv_autorecord 0
// CS2AS05 DEMO RECORDING END
```

保留关闭块的原因是状态可审计、可修复漂移，且不会误删玩家在其他位置的未知配置。写入函数需：

- 识别并替换完整 marker block。
- 若存在多个旧 block，收敛为一个。
- 不删除 block 外玩家自定义指令。
- 保留原换行风格；现有 `read_text` 已处理文本读取，仍要补 BOM/编码回归测试。
- 通过 `panel_transaction` 同时更新两个 cfg 和状态文件，任意一步失败则回滚。
- CS2 运行时拒写，返回稳定错误码，例如 `[DEMO_RECORDING_CS2_RUNNING]`。

### 4.3 desired / applied / drift

`cs2as05-panel-state.json` 或新的应用设置表应保存用户期望值 `desiredEnabled`。后端 snapshot 还需分别解析两个 cfg，报告：

- `desiredEnabled`
- `normalCfgApplied`
- `ffaCfgApplied`
- `drifted`
- `writable`
- `scope: bots-only`

不要把“读取到一条 `tv_enable 1`”当作唯一真相。设置关闭但 cfg 被手工改为 1 时，UI 要显示“配置已漂移”，提供“重新应用”按钮。

## 5. 产品范围

### 5.1 本期必须完成

- 导航新增“对局复盘”。
- 多个 Demo 目录的添加、移除、启停和扫描深度设置。
- 手动选择单个 `.dem` 文件导入。
- 默认建议 `replays`、`demos`、`game/csgo`，但只扫描存在且已授权/确认的目录。
- 手动“立即扫描”、进度、取消和每目录错误报告。
- 后台目录监听和去重入库。
- Demo 文件稳定性、header、最小大小检查。
- SQLite 索引、任务状态恢复、搜索/过滤/分页。
- 基础比赛摘要、记分板、回合时间线和玩家详情。
- 自动录制开关及 cfg 状态漂移提示。
- 助手启动会话基线；CS2 退出后发现、解析并自动打开本次最新战报。
- 解析失败可重试，不影响其他 Demo。

### 5.2 明确不做

- AI 锐评、LLM、在线上传和云端分析。
- OBS、FFmpeg、视频剪辑、高光片段录制。
- 全 tick 轨迹、2D 地图回放、热力图、烟火轨迹渲染。
- 修改第三方平台客户端目录或自动登录平台。
- 无限递归扫描整个磁盘、用户目录或所有 Steam 库。
- 将官方匹配描述为由本程序自动录制。

## 6. 页面与交互设计

实施 UI 时必须再次运行 UI/UX Pro Max，并以现有桌面工具风格为准。已调查方向是“安静、紧凑、可扫描的电竞数据工具”，不要做营销式 Hero、巨型标题或卡片套卡片。

### 6.1 导航

在 `src/components/AppShell.vue`：

- `ViewKey` 增加 `demoReview`。
- 导航在“命令”之后、“安装与诊断”之前增加 `对局复盘`。
- Lucide 图标首选 `ChartNoAxesCombined`。
- `view` map 增加 `DemoReviewView`。
- 当前页面状态应允许 Demo 会话协调器请求导航到某个 report ID，不能只靠组件内私有 `current`。建议创建轻量 `useWorkspaceStore` 或让 AppShell 订阅应用事件 `demo://open-report`，由单一位置更新页面。

### 6.2 录像库布局

页面顶部为正常尺寸标题“对局复盘”，右侧命令区：

- `添加目录`：FolderPlus 图标按钮 + 文本。
- `导入 Demo`：FilePlus 图标按钮 + 文本。
- `立即扫描`：RefreshCw 图标按钮；扫描中变为进度与 Square 停止图标。

标题下使用 tabs/segmented control：`录像库`、`对局报告`。不要用装饰性圆角胶囊。

录像库主体采用致密表格，不使用每条 Demo 一张大卡片。列建议：

- 文件名
- 地图
- 比分
- 回合
- 文件时间
- 大小
- 状态
- 操作

上方筛选条：文本搜索、地图筛选、状态筛选、日期范围、排序。表格支持分页，每页 25/50/100，默认 25。行双击或“打开报告”进入详情。

目录管理使用独立 dialog 或右侧抽屉，显示路径、是否启用、深度、最近扫描、发现数量、错误。路径必须允许换行/省略并通过 tooltip 查看完整值。

状态文案：

- `待检查`
- `等待文件写入完成`
- `待解析`
- `解析中`
- `已完成`
- `解析失败`
- `文件缺失`
- `不支持`

空状态必须可操作：没有目录时直接给“添加目录”和“导入 Demo”；有目录但无结果时给“立即扫描”。

### 6.3 对局报告布局

使用无嵌套卡片的 full-width sections：

1. 摘要带：地图、比分、队伍、时长、日期/文件时间、解析版本、文件入口。
2. 记分板：按队伍分组的表格。
3. 回合时间线：回合号、起始/结束比分、胜方、结束原因、下包/拆包、击杀事件。
4. 玩家详情：武器击杀、伤害、爆头、多杀回合等。

表格需要水平滚动容器和固定最小列宽；不能让长 Steam 名称挤压所有数值。所有只有图标的按钮必须有 `title` 和 `aria-label`。颜色不能是唯一的胜负/状态提示。

### 6.4 概览页录制控制

在 `src/views/OverviewView.vue` 启动 BOT 的控制区域附近放置录制 Toggle，不放到“对局复盘”深处作为唯一入口。这里是用户做启动决策的时刻。

同时在“对局复盘 > 目录设置”显示只读录制状态和“前往概览设置”，避免产生两套相互竞争的开关状态。

## 7. Demo 发现、扫描和监听

### 7.1 默认建议目录

以当前 `cs2.selectedRoot` 为基础生成候选：

```text
<selectedRoot>\game\csgo\replays
<selectedRoot>\game\csgo\demos
<selectedRoot>\game\csgo
```

仅对存在的目录显示“建议添加”。`game/csgo` 默认深度 2，避免把整个游戏资源树递归到底。用户可以通过 Tauri directory dialog 添加任意 Demo 目录。

不要自动扫描整个 `C:\`、所有用户下载目录或所有 Steam library。第三方平台目录必须由用户明确添加。

### 7.2 扫描规则

- 默认深度：2。
- UI 可选：0 / 1 / 2 / 3 / 5；最大 5。
- 第一版只入库 `.dem`。ZIP 支持可以作为阶段 2；若本次一定加入 ZIP，则必须实现安全解压、路径穿越检查、解压大小上限、文件数上限、CRC/哈希去重和 app data 隔离，不能在原 ZIP 旁随意写文件。
- 扫描按 mtime 从新到旧，先让最近比赛可见。
- 使用 `spawn_blocking` 或专用后台线程；不得阻塞 Tauri command 或 UI 主线程。
- 每批提交数据库，例如 100 条一事务；进度事件节流到 100-250ms，避免 Tauri Channel 洪泛。
- 可取消；取消后已提交的数据保留，任务状态为 `cancelled`。
- 单个 root 不存在、无权限或损坏，只记录该 root 错误，其他 root 继续。

### 7.3 路径安全

- 对 root 和候选文件执行 `dunce::canonicalize`。
- 候选文件 canonical path 必须位于某个启用 root 内，手动导入文件除外；手动导入需记录 `source = manual`。
- Windows 下比较路径使用规范化后的 case-insensitive 语义，不要简单字符串前缀比较，例如 `C:\demo2` 不能被判为 `C:\demo` 子目录。
- 默认跳过 symlink、junction/reparse point、隐藏/系统目录。
- 不跟随循环链接。
- 所有原始文件只读打开，不移动、不重命名、不删除。

### 7.4 文件稳定性与有效性

新发现文件依次经历：

1. 元数据可读。
2. size >= 1024 bytes。
3. 前 8 字节/必要 header 可识别为 `PBDEMS2` 或兼容的 `HL2DEMO`。
4. 每 250ms 采样 size + mtime，连续 3 次相同才视为稳定。
5. 常规监听最多等待 30 秒；退出会话可等待 30 秒，超时进入 `waiting_stable` 或可重试错误，而不是当作永久损坏。

由于大型 Demo 写盘可能短暂停顿，“连续稳定”后仍应尝试只读打开和 parser header；若 sharing violation，退避重试。

### 7.5 fingerprint 和去重

快速 fingerprint：

```text
canonical_path + file_size + mtime_ns
```

它用于扫描幂等和同一路径更新检测。内容 SHA-256 延迟计算：仅在路径变化疑似重复、手动导入或需要跨目录去重时后台计算。不要每次全目录扫描都读取所有大 Demo 全文。

目录监听使用 Rust `notify` crate，并把 create/modify/rename 统一送入同一 debounce queue。监听事件只是“需要重新检查”的提示，数据库和文件稳定性检查才是权威状态。

## 8. 解析架构

### 8.1 业务适配层

建议新增：

```text
src-tauri/src/services/demo/
  mod.rs
  scanner.rs
  watcher.rs
  stability.rs
  parser_adapter.rs
  metrics.rs
  repository.rs
  sessions.rs
  jobs.rs
  paths.rs
  models.rs
```

第三方 parser 只负责从 Demo 输出原始 header/player/event 数据。`parser_adapter.rs` 将其转换为本项目稳定 DTO；`metrics.rs` 负责按版本聚合 K/D/A、ADR、KAST 等。前端永远不依赖第三方 crate 的内部结构。

### 8.2 最小解析输入

第一版只请求以下信息，避免全 tick 解析带来的 CPU 和内存压力：

- header
- player info
- `round_start`
- `round_freeze_end`（若稳定可得）
- `round_end`
- `round_officially_ended`（若稳定可得）
- `player_death`
- `player_hurt`
- `weapon_fire`（仅确有指标需要时）
- `bomb_planted`
- `bomb_defused`
- `bomb_exploded`
- `bomb_beginplant` / `bomb_begindefuse`（只用于时间线）
- `player_disconnect` / `player_connect`（用于名单变化，若 parser 支持稳定）

先用真实样本证明字段，再实现指标。不要为了显示一个数而推测缺失值。

### 8.3 解析隔离和资源限制

`demoparser` 是库内解析，坏输入可能返回错误，也可能造成较重 CPU/内存。最低要求：

- 每次只并发解析 1 个 Demo；可配置上限最多 2，不默认并行吃满机器。
- parser 在 `spawn_blocking` 中运行，panic 用 `catch_unwind` 转成任务失败。
- 记录阶段和耗时：header / events / aggregate / persist。
- 单任务软超时只用于 UI 告警；Rust 阻塞线程无法安全强杀时，不能假装已终止。若真实测试证明恶意/损坏 Demo 会挂死，则第二阶段改为同 exe 的 `--demo-parser-worker` 子进程，用 JSON/stdout 协议并设置硬超时。
- 解析结果先在内存完成验证，再用单事务写入，不能留下半场数据。
- 文件在解析中变化时丢弃本次结果，重新排队。

### 8.4 兼容性和版本化

每个结果保存：

- `schema_version = 1`
- `parser_name = laihoe-demoparser`
- `parser_upstream_commit`
- `parser_adapter_version`
- `metrics_version`
- `app_version = 0.5.5`
- `parsed_at`

CS2 更新后 parser 可能暂时不兼容。UI 要区分：

- 文件不可读
- header 不支持
- parser 不支持当前 Demo 协议
- 聚合失败
- 文件被删除

升级 parser/metrics 后，旧结果保留并标记“可重新解析”，不要静默覆盖导致统计口径变化。

## 9. 数据和指标定义

### 9.1 比赛摘要

- 地图
- Demo 文件名、路径、大小、mtime
- header 中可可靠取得的服务器名/来源
- 总回合、最终比分、队伍名
- 比赛时长：只在事件 tick/timestamp 可可靠换算时显示
- 解析状态、解析时间、parser/metrics 版本

Demo 通常不能可靠提供真实开赛墙钟时间。没有证据时显示文件时间并明确标注“文件时间”，不要命名为“比赛开始时间”。

### 9.2 记分板

- 玩家名、SteamID、队伍
- K / D / A
- K-D
- 总伤害、ADR
- 爆头数、爆头率
- 首杀、首死
- KAST%
- 多杀回合

ADR 分母是该玩家实际参与且可判定的回合数，不应一律使用全场总回合。KAST 定义必须文档化：本回合至少满足 Kill / Assist / Survived / Traded 之一。Trade 的时间窗和判定对象必须写入 `metrics_version`，初版如果证据不足就不显示 KAST。

不要在第一版自行发明“Rating”。只有明确选择并测试公开公式后才能加入，且必须展示模型版本。缺失数据统一返回 `null`，前端显示 `--`，不能返回伪造的 0。

### 9.3 回合时间线

- round number
- 回合开始和结束 tick/time
- 回合前后比分
- 胜方与结束原因
- 击杀：时间、击杀者、受害者、武器、爆头、穿烟/穿墙等仅在字段可靠时展示
- 炸弹：开始下包、下包、开始拆包、拆包、爆炸

所有事件必须绑定内部 `round_id`。暂停、热身、重开和加时样本必须单独验证，不能简单按 `round_end` 数组下标假定连续正式回合。

## 10. SQLite 设计

数据库建议：

```text
<appData>/demo-review/demo-review-v1.sqlite3
```

使用 `rusqlite = { version = "...", features = ["bundled"] }`，启用 WAL、foreign keys 和合理 busy timeout。所有 migration 在 Rust 内显式版本化，使用 `PRAGMA user_version` 或 migration 表。

建议表：

```text
demo_roots
  id, path, canonical_path, enabled, scan_depth, source,
  last_scan_at, last_error_code, last_error_detail, created_at, updated_at

demo_files
  id, path, canonical_path, file_name, extension, source,
  size_bytes, mtime_ns, quick_fingerprint, sha256,
  status, error_code, error_detail,
  discovered_at, stable_at, parsed_at, missing_at,
  map_name, server_name, total_rounds,
  team_a_name, team_b_name, team_a_score, team_b_score,
  active_result_id

parse_results
  id, demo_file_id, schema_version, parser_name,
  parser_commit, parser_adapter_version, metrics_version,
  app_version, created_at, duration_ms, is_current

teams
players
rounds
kills
damage_events_or_aggregates
bomb_events

scan_jobs
  id, kind, status, started_at, finished_at, progress_json,
  error_code, error_detail, cancel_requested

launch_sessions
  id, selected_root, mode, assistant_launched,
  recording_desired, started_at, process_seen_at, stopped_at,
  baseline_json_or_session_demo_rows, result_demo_file_id,
  auto_opened_at, state, error_code, error_detail

session_baseline_files
  session_id, quick_fingerprint

settings
  key, value_json, updated_at
```

列表查询必须在 SQL 做筛选、排序、分页，不能把所有 Demo 全量发到 Vue 再过滤。用户选择排序字段必须映射白名单 SQL 列，不能直接拼接任意字符串。

删除库记录默认只删除数据库解析结果，**不删除原 Demo 文件**。若未来加入删除原文件，必须单独二次确认，本期不做。

## 11. CS2 启动到退出后弹战报的完整时序

### 11.1 建立会话

在 `src-tauri/src/services/panel.rs::launch_cs2_inner` 真正启动 Steam 前：

1. 验证 root、模式和 CS2 未运行。
2. 若 BOT 模式且自动录制 desired 为 true，确认两个 cfg 已应用；有 drift 时在启动事务中修复，失败则阻止启动并给出可操作错误。
3. 取得启用的扫描 roots，加上本 CS2 root 的默认候选目录。
4. 快速扫描/查询已知 Demo fingerprint，建立 baseline。
5. 创建 `launch_sessions`：`Prepared -> Launching`，记录 `started_at`、root、mode、recording_desired、assistant_launched = true。
6. 启动 Steam/CS2。
7. 返回 `session_id` 给前端，现有 `LaunchResult` 增加该字段。

baseline 必须在进程启动前完成。若目录很多，不应全文件 SHA-256；只需对本次候选时间窗内文件记录 quick fingerprint，或利用数据库已知状态加一次有界目录快照。

### 11.2 进程边沿

新增 `useDemoSessionCoordinator` 或在 demo store 中订阅 `cs2.cs2ProcessState`：

- `stopped -> running`：通知 Rust session `process_seen_at`，状态 `Running`。
- `running -> stopped`：通知 Rust完成 session，状态 `WaitingForDemo`。
- `unknown`：不结束会话，等待下一次可靠状态。

10 秒轮询可能使短会话无法看到 running。为降低漏检：

- launch 后前 60 秒将进程检查频率临时提高到 1-2 秒；或更优，让 Rust session coordinator 自己每 1 秒检查进程，前端只展示状态。
- 推荐后者：会话生命周期属于后端，不能依赖 WebView 是否可见或组件是否挂载。
- 应用关闭后重新打开时，Rust 从数据库恢复未完成 session；若 CS2 已停止则继续发现 Demo。

### 11.3 退出后选择本次 Demo

检测到可靠退出后：

1. 对 session 候选 roots 做有界增量扫描。
2. 候选必须满足 `mtime >= session.started_at - tolerance`，建议 tolerance 5 秒，并且 quick fingerprint 不在 baseline。
3. 等待每个候选稳定，最长 30 秒。
4. 按稳定完成时间优先，其次 mtime、size 排序。
5. 若只有一个高置信候选，绑定 session 并解析。
6. 若多个候选且无法可靠区分，不自动误选；弹应用内选择 dialog，列出文件名、路径、mtime、大小。
7. 没找到时 session 进入 `NoDemoFound`，显示“本次未发现新 Demo”，提供“选择文件”和“重新扫描”。

不能只调用 `read_dir(replays).max_by(mtime)`，否则会把旧 Demo 当成本局结果。

### 11.4 自动打开报告

解析成功后后端发 Tauri event/channel payload：

```json
{
  "sessionId": "...",
  "demoFileId": 123,
  "resultId": 456,
  "fingerprint": "..."
}
```

前端收到后：

- 若主窗口最小化，使用 Tauri window API show/unminimize/focus。
- 导航到 `对局复盘 -> 对局报告` 并打开 `resultId`。
- 只进行应用内页面切换，不新建系统窗口。
- 若用户正操作 modal 或输入，先 toast“本次对局报告已生成”，不要粗暴覆盖未保存交互；点击 toast 再打开。

去重键为 `session_id + demo_fingerprint`，数据库记录 `auto_opened_at`。重启、重复 watcher 事件或重复解析不能再次强制弹页。

### 11.5 非助手启动的 CS2

- watcher 仍可发现新 Demo 并入库。
- 默认不自动强制打开报告，因为没有可靠 session baseline。
- 可在后续高级设置增加“检测所有 CS2 会话”，本期不作为默认行为。
- 自动录制关闭时，录像库、手动导入和解析仍完全可用。

## 12. IPC 和前端状态设计

### 12.1 Rust commands

建议新增 `src-tauri/src/commands/demo.rs`，注册到 `commands/mod.rs` 和 `lib.rs`：

```text
get_demo_settings
set_demo_recording_enabled
list_demo_roots
add_demo_root
update_demo_root
remove_demo_root
scan_demo_roots
cancel_demo_scan
import_demo_file
list_demos
get_demo_report
queue_demo_parse
retry_demo_parse
cancel_demo_parse
open_demo_folder
reveal_demo_file
resolve_session_demo
get_active_demo_session
```

扫描/解析不应让一个 invoke 长时间占用并返回巨型结果。command 返回 job ID，进度通过 Tauri Channel 或事件发送；列表和报告再用查询 command 获取。

### 12.2 TypeScript 层

建议新增：

```text
src/types/demo.ts
src/services/tauri/demo.ts
src/stores/demo.ts
src/composables/useDemoSessionCoordinator.ts
src/views/DemoReviewView.vue
src/components/demo/DemoLibraryToolbar.vue
src/components/demo/DemoRootsDialog.vue
src/components/demo/DemoTable.vue
src/components/demo/DemoReport.vue
src/components/demo/RoundTimeline.vue
src/components/demo/ScoreboardTable.vue
src/components/demo/SessionDemoPicker.vue
```

Pinia store 保存查询条件、当前页、当前 report ID、job 进度和 settings snapshot。大型 report 可以按 result ID 缓存，但切换 root/settings 时不得错误复用。

所有跨 IPC 类型必须有 Rust serde DTO 和 TypeScript 镜像，字段使用 Tauri 当前项目约定的 camelCase。错误保持稳定 code + 中文 detail，前端不要靠匹配中文字符串分支。

## 13. 建议实施阶段

### 阶段 0：保护现场和固定证据

- 先执行 `git status --short`，记录当前脏工作树，不回退、不覆盖已有 0.5.5 改动。
- 固定三个研究仓库 commit 和许可证，写 `third_party/demoparser/UPSTREAM.md`。
- 准备合法可提交的 Demo 测试夹具；若真实 Demo 太大，不应直接提交仓库，可用本地 fixtures manifest + hash。

### 阶段 1：parser spike

- vendor Rust core，建立最小 adapter。
- 用至少 3 个真实 CS2 Demo 验证 header、players、round_end、player_death、bomb events。
- 记录解析时间和峰值内存。
- 先输出本项目 DTO JSON fixture，不急于做完整 UI。
- 若固定 parser commit 无法解析当前 CS2 Demo，先评估上游更新并重新固定 commit，不在旧 parser 上堆脆弱补丁。

### 阶段 2：SQLite、扫描和手动导入

- migration、repository、roots、scan jobs、稳定性和 fingerprint。
- 先实现手动导入单文件，再实现有界目录扫描，最后加 watcher。
- 完成错误恢复、取消、文件缺失和重新发现。

### 阶段 3：录像库 UI

- 导航、表格、筛选分页、目录管理、扫描进度和空/错/加载状态。
- 用 mock/fixture 验证长路径、长玩家名、100+ 行、窄窗口和 125%/150% Windows 缩放。

### 阶段 4：报告聚合和 UI

- 比赛摘要、记分板、回合时间线、玩家详情。
- 指标按 `metrics_version` 测试。
- 未知字段显示 `--`，不推测。

### 阶段 5：自动录制和会话闭环

- marker block、desired/applied/drift、CS2 运行中拒写。
- launch session baseline、Rust 进程观察、退出后稳定等待、候选选择和自动打开。
- 这是功能最关键的端到端阶段，必须用真实 CS2 做人工验收。

### 阶段 6：文档、兼容和发布前检查

- 更新 README，删除“不提供 Demo 管理”的旧声明，补充本地数据与录制边界。
- 更新 `NOTICE.md`、第三方许可证、隐私说明。
- 保持所有版本号 `0.5.5`。
- 大项目验收按用户约束简化，但核心 Rust 单测、前端类型检查和关键 Vitest 不可省。

## 14. 自动化测试矩阵

### 14.1 Rust 单元/集成测试

- marker block 新增、替换、重复收敛、开启/关闭、CRLF/LF、BOM、未知行保留。
- 双 cfg 任一写入失败时事务回滚。
- CS2 运行中拒改录制设置。
- canonical path 子目录判断和 Windows 大小写。
- 扫描深度 0/1/2/5，跳过 reparse point 和循环。
- 不存在/无权限 root 不阻断其他 root。
- 文件 size/mtime 稳定检测、超时、sharing violation 重试。
- `PBDEMS2`、`HL2DEMO`、过小文件、错误 header。
- create/modify/rename watcher debounce 去重。
- quick fingerprint 幂等；同路径内容更新生成新版本。
- SQLite migration 从空库、重复启动、损坏/不可写错误。
- 搜索、地图/状态筛选、排序白名单、分页总数。
- parser adapter 的 header/player/event fixture。
- panic 捕获、错误状态、重试后成功。
- 正常回合、加时、重开、热身事件分组。
- K/D/A、ADR、首杀首死、KAST（若实现）的公式 fixture。
- 会话 baseline 排除旧文件。
- 多候选时不自动误选。
- session + fingerprint 自动打开去重。
- 应用重启恢复未完成 session。

### 14.2 前端 Vitest

- AppShell 出现“对局复盘”并可切换。
- 录像库 loading/empty/error/data 状态。
- 搜索筛选和分页参数正确传给 IPC。
- 扫描进度、停止按钮和 root 局部错误。
- 录制 Toggle 在 CS2 运行时禁用并解释原因。
- desired/applied drift 显示和重新应用。
- report 缺失数据显示 `--`。
- 长路径/长名字不溢出按钮和表格。
- Tauri 完成事件只导航一次。
- 多候选选择 dialog 的确认/取消。

### 14.3 不依赖真实 CS2 的端到端夹具

准备临时目录模拟：

- 旧 Demo + 会话后新 Demo。
- 写入中不断增长的 Demo。
- 同名覆盖。
- 多目录同时出现文件。
- 文件在扫描后删除。
- 损坏 header。
- parser 不支持协议。

文件 watcher 测试在 Windows CI/本机可能有时序波动，应以最终数据库状态轮询断言，不使用固定 100ms 睡眠赌事件到达。

## 15. Windows + 真实 CS2 人工验收

实际执行 AI 无法可靠替代真实玩家完成游戏内过程，应在交付时给用户一份短验收清单。至少覆盖：

1. 关闭 CS2，开启“自动录制本地对局 Demo”，确认两个 cfg marker block 均为 1。
2. 通过助手启动 BOT 模式，确认启动前设置已应用。
3. 完成至少 2-3 回合后正常退出 CS2。
4. 记录 Demo 实际生成目录、文件名、header、文件大小停止增长所需时间。
5. 确认程序只选择本次新增 Demo，不选择目录中的旧 Demo。
6. 确认解析完成后主窗口出现并打开对应战报，且只打开一次。
7. 对照游戏最终比分、玩家 K/D 和至少 3 个击杀事件、1 个下包/拆包事件。
8. 再进行一次 BOT 对局，证明第二次会话 baseline 不复用第一次结果。
9. 游戏运行中尝试切换录制设置，应明确拒绝且不破坏 cfg。
10. Online 模式启动后确认 UI 不声称已为官方匹配录制。
11. 手动添加一个外部 Demo 目录并扫描，确认可解析非助手生成 Demo。
12. 在 Demo 写入中关闭/重开应用，确认任务能恢复或安全重试。

建议保留以下验收证据到 `docs/screenshots` 或不提交的大文件测试记录：

- cfg marker 前后对比。
- 实际 Demo 路径和 `sha256`。
- session started/stopped timestamps。
- 自动选中的 fingerprint。
- 报告截图和关键数据人工对照表。

## 16. 性能和可用性门槛

- 空库打开页面不等待扫描完成。
- 10,000 条 demo_files 的分页查询应保持交互可用，目标本机 < 200ms；超出时补索引。
- 扫描进度事件节流，不能让 Vue 每个文件重渲染一次。
- 解析默认单并发，UI 始终可取消排队任务并继续浏览旧报告。
- 大型报告不要一次渲染数千事件；回合列表折叠或虚拟化，默认只展开选中回合。
- watcher 只监听启用 root，应用退出时正确释放。
- app data/数据库不可写时功能进入只读/错误状态，不影响概览、BOT 设置和其他既有页面启动。

## 17. 关键风险与处理

### 风险 A：`tv_autorecord` 实际目录/命名变化

处理：不依赖单一路径；使用默认候选 + 用户目录 + session baseline；以真实运行结果为权威。

### 风险 B：CS2 更新使 parser 失效

处理：固定 parser commit 和版本字段，错误分类，保留原文件与旧结果，可重新解析；先做 parser spike。

### 风险 C：进程轮询漏掉退出或 WebView 被关闭

处理：会话 coordinator 放 Rust 后端并持久化，不依赖页面挂载；重启恢复。

### 风险 D：大型 Demo 卡死或吃满内存

处理：单并发、`spawn_blocking`、阶段日志、panic 捕获；必要时升级为子进程 worker 硬隔离。

### 风险 E：扫描范围过大或目录循环

处理：只扫描明确 roots、最大深度 5、跳过 reparse point、可取消、每 root 隔离错误。

### 风险 F：统计看似完整但口径错误

处理：真实事件证据优先，指标版本化，缺失显示 `--`；不急于加入 Rating/KAST。

### 风险 G：许可证污染

处理：CS2-insight-agent 可以复制；Local-Arena 可以复制；demoparser 固定 commit、原样 LICENSE、UPSTREAM/NOTICE、原始与适配层分离。

## 18. 完成定义

只有同时满足以下条件，才能宣布本次大功能完成：

- 版本仍为 `0.5.5`。
- 新导航、录像库、报告、目录管理、扫描进度/取消、搜索筛选分页完整可用。
- 至少 3 类真实 Demo 可解析，坏 Demo 不拖垮应用。
- 所有展示数据来自可靠 header/events/聚合，缺失值不伪造。
- 自动录制设置正确、可审计、可检测 drift，且只明确承诺 BOT/本地对局。
- 从助手启动 BOT 到 CS2 退出、发现本次新 Demo、稳定等待、解析、打开本次报告的完整流程在干净基线下成功复现两次。
- 多候选不误选，旧 Demo 不误弹，重复事件不重复弹。
- 应用重启后数据库和未完成会话可恢复。
- 第三方许可证、NOTICE、README 和数据存储说明同步更新。
- 核心 Rust 测试、TypeScript typecheck、关键 Vitest 通过；若因环境未跑某项，交付中必须明确说明。

## 19. 实际执行 AI 的第一步

不要直接从 UI 开始。先按以下顺序行动：

1. 读取当前 `git status` 和本文，保护所有现有未提交改动。
2. 完成 demoparser vendor + 最小 parser spike，用真实当前版本 Demo 证明能得到 header、players、round_end、player_death 和 bomb events。
3. 根据 spike 的真实字段冻结 `DemoReportV1` DTO 和 metrics 范围。
4. 实现 SQLite migration、单文件导入和扫描，形成“文件 -> 稳定 -> 解析 -> 数据库 -> 查询报告”的窄端到端闭环。
5. 再做录像库/报告 UI。
6. 最后接 cfg 自动录制和 CS2 session 退出闭环，并交给用户做真实游戏内验收。

这条顺序能尽早暴露 parser 与当前 CS2 Demo 协议的兼容性风险，避免先完成大量 UI 后才发现底层数据拿不到。
