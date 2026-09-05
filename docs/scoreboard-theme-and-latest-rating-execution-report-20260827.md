# 战报外观同步与 LBRating 2.0 执行报告

执行日期：2026-08-27  
工作区：`E:\CS2AS05`  
依据方案：`docs/scoreboard-theme-and-latest-rating-handoff-plan-20260827.md`

## 已实现

- 独立战报入口 `src/scoreboard.ts` 在挂载 Vue 前调用 `initializeAppearancePreferences()`，复用主程序现有外观偏好和存储格式。
- 战报页继续使用既有窗口、关闭协议、报告加载和布局结构；未修改 Demo 解析、比分、Viewer 或发布流程。
- `src/ScoreboardApp.vue` 仅接受 `schemaVersion >= 6` 且 `metricsVersion === 'lb-rating-2.0'` 的报告，旧 `simple-rating` 报告显示“报告版本过旧，请重新解析。”。
- `PostMatchScoreboard.vue` 表头、模型详情和页脚统一显示 `LBRating 2.0` / `lb-rating-2.0`。
- `simple_rating.rs` 的机器版本为 `lb-rating-2.0`，Rating 使用 ADR 主导的四项模型：
  - `ADR = DMG / R`，组件 `clamp(ADR / 82, 0, 3)`，权重 0.55；
  - `KPR = K / R`，组件 `clamp(KPR / 0.70, 0, 2.5)`，权重 0.20；
  - `DPR = D / R`，生存组件 `clamp((1 - clamp(DPR, 0, 1)) / 0.68, 0, 2.5)`，权重 0.15；
  - `APR = A / R`，组件 `clamp(APR / 0.20, 0, 2.5)`，权重 0.10；
  - 最终 Rating 限制在 `0..3`，各组件和结果四舍五入四位。
- CSS 支持主程序已有的 `data-theme`、`data-palette`、`data-radius`、`data-density` 属性：浅色/深色、default/rose/tide/sunset/forest/sea/dream、auto/0/0.25/0.5/0.75/1.0，以及 compact/default/loose。
- 当前运行契约中的 Demo 缓存 metrics 版本和相关提示已切换到 `lb-rating-2.0` / `LBRating 2.0`。

## 新增表现雷达专项状态

本轮继续补充了雷达基础链路：后端 `MatchPerformanceRadar` 增加 `benchmarkMethod="second-highest-per-axis"` 与 `cohortSize`；benchmark 计算在应用 `player_keys` 展示过滤之前执行；前端增加基于完整 roster 的“本场表现排名”区域、冠军/亚军/季军语义和 0.75..2.0、0.05 步进的 SVG 缩放控件。`normalizeRadarAxis()` 继续覆盖逐轴第二高、重复值、单人、全零和安全上限保护。

但方案要求的旧轴名整体迁移为 `kpr/surviving/adr/kast/impact/rating`、Impact 可审计来源、Rating 轴接入后端、完整 PK 轴胜出统计以及真实视觉矩阵仍未全部完成。现有后端查询已证明 `player_keys` 不再影响 benchmark 范围；由于当前数据库 DTO 尚无稳定的 Rating 字段来源，Rating 轴尚未伪造接入。该报告只能标记为“部分实现/待验收”，不能视为表现雷达专项完成。

## 自动化验证

以下命令均通过：

```text
npm test -- --run tests/post-match-scoreboard.spec.ts tests/release-blocker-recovery.spec.ts tests/theme-preference.spec.ts tests/appearance-preferences.spec.ts
Test Files 4 passed; Tests 13 passed

npm run typecheck
退出码 0

npm run lint
oxlint 0 warnings / 0 errors；eslint 退出码 0

npm run build:web
构建成功；仅有既有的大 chunk 提示

cargo test --manifest-path src-tauri/Cargo.toml simple_rating
4 passed; 0 failed
```

Rust 固定向量包含：平均向量 Rating `1.0044`、零表现 Rating `0`、dominant 向量 Rating `2.6206`、缺失/零回合不可用、极端输入仍在 `0..3`，以及高影响玩家排序高于被动玩家。

## 视觉验收状态

本轮未启动真实 Tauri 窗口，也未加载真实 CS2 Demo 做四矩阵截图。因此以下证据仍待用户确认：

1. light + default + default density + 1280x800；
2. dark + default + default density + 1280x800；
3. light + compact + 980x640；
4. dark + loose + 非默认 palette + 980x640。

验收时应回读 `document.documentElement.dataset`，确认 `theme/palette/radius/density` 与主程序一致；确认页脚可见 `LBRating 2.0 · lb-rating-2.0`，无白屏、文字重叠或不可滚动内容。当前不能将真实 Windows/Tauri/CS2 验收表述为已完成。

## 变更边界

未提交、未推送、未部署、未上传 GitHub/R2，未修改版本号。工作区其他既有改动保持原样。

## 下一阶段交接方案：表现雷达相对归一化与展示改造（未实施）

本节是交给下一位【实际执行 AI】的完整实施方案。本执行报告前文结论仍然有效：本轮未实施表现雷达专项；当前雷达仍使用 `performance-radar-v1`、旧六轴和固定 benchmark。不得仅修改前端标签后宣称完成，必须完成后端 DTO、归一化、几何、排名展示和验证闭环。

### A. 六轴与几何固定契约

六边形必须为竖立方向：SVG 第 1 个顶点角度固定 `-90deg`，其后每次顺时针增加 60 度，4 号顶点在正下方。轴顺序、key 和文案锁定如下，不得跟随返回顺序或玩家选择顺序变化：

```text
1 kpr       KPR（回合击杀）
2 surviving Surviving（回合存活）
3 adr       ADR（回合伤害）
4 kast      KAST（贡献率）
5 impact    Impact（影响力）
6 rating    Rating（LBRating 2.0 · lb-rating-2.0）
```

同步修改 `src-tauri/src/models/demo.rs`、`src/types/demo.ts`、`src-tauri/src/services/demo.rs`、`src/features/demo/radar/performance-radar.ts`、`MatchPerformanceRadar.vue`、`PerformanceRadarChart.vue` 和相关测试。旧 `firepower/damage/survival/participation/teamwork/opening` 不得继续作为六轴 key。

### B. 每轴取全场第二高作为满格

归一化必须按完整本场有效参赛 roster、逐轴独立执行，不按某一个玩家的六项最大值，也不按当前选中的 1-3 人计算。对轴 `d`：

1. 收集所有 T/CT 正式参赛玩家的有限 raw 值，降序排列并去重。
2. 至少两项时取第二高值 `secondHighest_d`；只有一项时用唯一值；没有有效值则该轴为 unavailable。
3. `score_d = raw_d / secondHighest_d * 100`。第二高为 100；第一高可超过 100。例如 ADR 第一名 150、第二名 100，则分别为 150 和 100，其余按比例缩短。
4. 基准为 0 时禁止除零：所有 raw=0 则 score=0 并标记 flat；存在正值时取正值作为基准；负值、NaN、Infinity 不参与。
5. 不得把 score 截断到 100。保留独立异常保护上限（建议 400% 或经测试的更大值）仅防止异常数据破坏 DOM；正常越界必须原样显示并记录是否触发保护。

后端应返回每个维度的 `raw`、实际 `benchmark=secondHighest_d`、`score`、`source`、`quality`，并在响应增加 `benchmarkMethod: "second-highest-per-axis"`、完整 `cohortSize`。如果在前端计算，必须把完整 roster 与 selected 分离，切换选择人不能改变 benchmark。

### C. 六轴 raw 数据来源

在 `calculate_performance_radar` 中取消固定 `0.80/0.50` 等 benchmark：

```text
KPR       = kills / participated_rounds
Surviving = survived_rounds / participated_rounds
ADR       = damage_health / participated_rounds
KAST      = kast_rounds / participated_rounds
Impact    = 使用已有 firstKills、tradeKills、multiKillRounds 等可靠落库字段的可审计组合；无法证明来源则 null
Rating    = DemoPlayer.rating.rating，modelVersion 必须为 lb-rating-2.0
```

Impact 必须返回明确公式、单位和 source，不能用常数占位。缺失轴保留 null，雷达断轴/显示 unavailable，禁止用 0 补齐。Rating 轴只读取 LBRating 2.0 最终值，不重算第二套评分。

### D. 用户缩放与越界显示

删除 `RADAR_MIN_DISPLAY_MAX=140` 作为业务满格的语义，也不能把 `RADAR_SAFE_SCORE_MAX=500` 当作第二名基准。几何以 100 为第二高参考圈，允许 score>100 向外；网格可显示 25/50/75/100，按当前最大值增加 125/150 等参考圈。SVG/父容器设为 `overflow: visible`，雷达层使用更高 stacking context，允许覆盖普通 UI，但不得遮住关闭、错误和关键操作控件。

增加可访问缩放控件（range + 减/加/重置）：建议范围 `0.75..2.0`、步进 `0.05`、默认 `1`，显示当前百分比并支持键盘。缩放只改变半径/绘图区，不改变 raw、benchmark 或 score。固定 viewBox、响应式容器和标签换行，确保 980x640 与移动宽度可读；必要时允许雷达层覆盖右侧普通信息。

### E. 右侧单人排名和多人 PK

将 `.radar-insight` 改为排名展示区，同时保留警告列表：

- 单人：显示名称、队伍/BOT、`本场表现排名 第 N / M 名`。默认按 LBRating 2.0 降序；Rating 缺失时按有效轴比例总和/平均降级，并明确“Rating 不可用”。前三名使用冠军/亚军/季军文字或图标与金银铜层级，不能只靠颜色。
- 多人：显示与 polygon 颜色/线型对应的 PK 布局；每人显示本场排名、名称、LBRating、胜出轴数量或领先轴提示。最多三人沿用现有上限；排名基于完整 roster，不是 selected 子集。
- 明确“本场表现排名”，不得称为正式比赛名次。名称和数值在移动宽度必须换行，不能溢出。雷达越界时雷达可覆盖普通展示区，但普通布局仍应避免无意义重叠。

### F. 表格、无障碍与测试

精确表格改为：轴位、维度、玩家、图形比例、原始值、本场第二高基准、数据质量。`benchmark` 文案必须写“本场第二高”，不再写固定基准。SVG `title/desc` 说明竖立方向、1-6 轴、第二高满格和第一高可越界；polygon、排名卡和缩放控件提供 aria-label。颜色、线型和文字同时表达状态。

必须新增/更新测试：

- 纯函数验证六轴顺序和角度：1 在上、4 在下、顺时针不变。
- 每轴第二高归一化：`[150,100,50] -> [150,100,50]`，切换 selected 不改变结果，重复值/单人/全零/缺失/异常上限均有断言。
- 旧 benchmark 和旧 key 不再出现于运行契约；完整 roster 排名独立于 selected。
- 右侧单人前三名、多人大于一人的 PK、Rating 缺失降级排序、并列稳定排序。
- 缩放上下限、步进、重置、键盘/aria；越界 polygon 不被裁剪。
- Rust 后端 DTO、Impact 来源、Rating 轴 `lb-rating-2.0`、旧报告/旧缓存门禁。

### G. 验收与停止条件

执行以下定向验证并保存证据：

```powershell
npm test -- --run tests/performance-radar.spec.ts tests/match-performance-radar.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml performance_radar
```

使用包含至少三名有效玩家且各轴数值不同的真实/受控 Demo，回读 JSON 证明每轴 `benchmark` 等于该轴第二高，ADR 第一名可得到 150 等越界比例，Rating 轴来自 `lb-rating-2.0`。在浅/深色及 1280x800、980x640、移动宽度截图中确认雷达完整、标签不遮挡、越界可见、缩放可用、右侧排名/PK正确。未有真实 Tauri/CS2 证据时只能报告“代码/自动化通过，实机待验收”。

任一 DTO 与前端 key 不一致、Impact 无可审计来源、只按 selected 计算 benchmark、越界被裁剪、旧固定 benchmark 仍影响结果、或测试无法证明第二高归一化时停止，不得宣称表现雷达专项完成。

## 本轮补充：六边形位置锁定与右侧展示执行单（仍未实施）

以下内容是本次交给【实际执行 AI】的最终执行顺序和不可变契约。它补充并细化上面的 A-G，不改变“表现雷达专项尚未完成”的状态。

### 1. 先锁定轴位，再接入数据

在任何 UI 重排或样式工作前，先让后端 DTO、TypeScript 联合类型和几何纯函数共享同一常量顺序：

```text
index 0 / 1号角：kpr       -> KPR（回合击杀）
index 1 / 2号角：surviving -> Surviving（回合存活）
index 2 / 3号角：adr       -> ADR（回合伤害）
index 3 / 4号角：kast      -> KAST（贡献率）
index 4 / 5号角：impact    -> Impact（影响力）
index 5 / 6号角：rating    -> Rating（LBRating 2.0 · lb-rating-2.0）
```

`radarPoint(0, ...)` 必须返回顶部点，`radarPoint(3, ...)` 必须返回底部点；index 递增必须是顺时针。不得再从 `dimensions` 数组当前顺序推断角点，必须按 key 映射后按上述常量输出。旧 `firepower/damage/survival/participation/teamwork/opening` 在运行 DTO、纯函数和模板中全部移除；历史执行报告文字可保留。

### 2. 第二高基准必须来自完整 roster

保留已新增的 `normalizeRadarAxis()` 纯函数，但实际接线必须在 `performance_radar_from_db` 或其紧邻的服务层完成：先查询本场所有 T/CT 正式玩家的 raw 六轴矩阵，再逐轴计算第二高 benchmark，最后把 benchmark 应用到每位玩家。`get_match_performance_radar` 接收的 `player_keys` 只能限制返回展示玩家，绝不能限制 benchmark 计算集合。

每轴响应至少包含 `raw`、`score`、`benchmark`、`source`、`quality`；报告级增加 `benchmarkMethod="second-highest-per-axis"` 与 `cohortSize`。第二高为 100 分，不是固定满分；第一名超出 100 必须保留。示例测试必须证明完整 raw 为 `[150,100,50]` 时输出 `[150,100,50]`，先选 50 再改选 150 不会改变 100 基准。

### 3. 图层、缩放和右侧模式必须同一组件完成

`PerformanceRadarChart.vue` 负责固定六角几何、越界 SVG 和缩放值；`MatchPerformanceRadar.vue` 负责选择状态、完整 roster 排名和右侧模式切换。不要把排名重新塞回雷达 SVG 里导致文本随 polygon 越界。

- SVG 与所有必要父级 `overflow: visible`，雷达容器建立更高 stacking context；普通信息允许被雷达覆盖，但关闭/错误/键盘焦点控件必须在更高层。
- 缩放控件使用可见 range 输入及减小、增大、重置按钮，范围建议 `0.75..2.0`、步进 `0.05`、默认 `1.0`；控件只改变半径/容器，不重新请求数据。
- 单人选择时右侧渲染名称、队伍/BOT、`本场表现排名 第 N / M 名`；排名优先 LBRating 2.0，缺失时按有效轴比例平均降级。前三名必须同时有冠军/亚军/季军文字或图标和金/银/铜层级。
- 多人选择时右侧切换 PK 展示，保持每位玩家与雷达颜色/线型一致，展示排名、LBRating、胜出轴数量或领先轴；排名仍来自完整 roster，最多比较三人。
- 所有排名文案必须写“本场表现排名”，不写正式比赛名次；移动宽度下名称、排名和分数必须换行。

### 4. 执行门禁

实际执行 AI 只有在下列证据全部存在时才能把报告状态改为已完成：

1. Rust/TS DTO 六轴顺序和 `benchmarkMethod` 回读一致，Impact source/公式可审计，Rating 轴确认为 `lb-rating-2.0`。
2. 单元测试证明顶部/底部角点、顺时针、第二高归一化、selected 不影响 benchmark、越界不截断和缩放边界。
3. 组件测试证明单人前三名、多人大 PK、Rating 缺失降级排名、aria-label 与关键控件层级。
4. 真实或受控 Demo JSON 证明每轴第二高值；截图覆盖浅/深色、1280x800、980x640 和移动宽度，第一名越界可见且未被普通容器裁切。
5. 若任一证据缺失，只能在报告中写“部分实现/待验收”，不得把当前 `performance-radar-v1` 兼容实现描述成新雷达完成。

## 2026-08-28 客户端故障与备份策略补充

- 新增结构化 `Cs2ProcessInfo`/`Cs2ProcessSnapshot`、双采样状态和精确 `cs2.exe` 识别；Steam 不参与 CS2 running 判定。
- 新增 `get_cs2_process_snapshot` 与 `close_cs2(force)` IPC。关闭只操作已枚举 PID，优雅阶段等待 8 秒，强制阶段等待 5 秒，最终快照为空才报告成功。
- StatusStrip 新增关闭按钮和应用内强制关闭确认，关闭期间锁定重复操作；旧测试 mock 仍可回退布尔检查。
- `atomic_write` 默认 `keep_backup=false`，写入成功删除本次 `.backup-*`，失败仍保留回滚路径；显式 `BackupPolicy` 入口已提供。

验证：`npm run typecheck` 通过；`tests/cs2-process-polling.spec.ts` 与 `tests/release-blocker-recovery.spec.ts` 共 8 项通过；`cargo check` 通过；Rust `cs2` 目标测试 20 项通过、1 项忽略。Panel 目标测试在补齐静态 gameinfo sidecar 夹具后为 18 项通过、1 项真实环境测试忽略、0 项失败。

真实 Tauri 窗口、用户电脑“CS2/Steam 均关闭”回读、受控进程关闭截图、10 次安装备份清单和 `keepBackup=true` UI/IPC 一次性勾选链路尚未完成，当前状态为“部分实现/待真实验收”。

## 2026-08-28 客户端故障与备份策略继续执行记录

本轮在既有脏工作区上继续补齐了写前备份的受限清理链路：

- `src-tauri/src/services/panel.rs` 新增 `cleanup_owned_backups(csgo_root, dry_run, preserve_operation_id)`。
- 清理器只识别助手生成的 `*.backup-YYYYMMDD-HHMMSSmmm-<operation_id>` 文件；先 canonicalize 路径并限定在当前 `game/csgo` 根目录内。
- 普通 `backup` 目录及其内容不会被遍历；用户手工命名文件、指定 operation id 的备份不会被删除。
- `dry_run=true` 只返回待删路径；真实清理可重复执行，第二次返回空列表。
- Panel 事务成功后执行 best-effort 清理；清理失败不会覆盖主要写入结果。
- 修正安装事务测试调用，明确默认 `keep_backup=false`。

新增/通过的 Rust 回归：

```text
cargo test --manifest-path src-tauri/Cargo.toml panel::tests::cleanup_owned_backups_is_scoped_idempotent_and_preserves_requested_operation --lib
1 passed; 0 failed
```

同轮前端验证仍通过：

```text
npm run typecheck
npm test -- --run tests/cs2-process-polling.spec.ts tests/release-blocker-recovery.spec.ts
2 个测试文件通过；8 个测试通过
```

此前 `panel` 目标测试中的 `mutations_converge_to_the_aggregated_disk_snapshot` 因夹具缺少静态 gameinfo baseline 失败；本轮已补齐与受控文件一致的 `GameInfoSidecar`，复核结果为 18 项通过、1 项真实环境测试忽略、0 项失败。

本轮尚未补齐所有 Panel 写入入口的用户可见 `keepBackup` 传参，也没有执行真实 Tauri 窗口、受控进程、CS2/Steam 关闭状态回读、10 次安装备份清单或截图验收。因此当前状态仍严格为：**部分实现/待真实验收**。

## 2026-08-28 进程关闭实现补充

继续执行后完成了关闭路径的 Windows API 收敛：

- `src-tauri/Cargo.toml` 增加 `Win32_System_Threading` 与 `Win32_UI_WindowsAndMessaging` features。
- `src-tauri/src/services/cs2.rs` 的优雅阶段现在只对快照中已确认的 CS2 PID 枚举窗口并发送 `WM_CLOSE`，不再调用 `taskkill`。
- 强制阶段只对同一快照 PID 使用 `OpenProcess(PROCESS_TERMINATE)` 与 `TerminateProcess`；失败会返回带 PID 的诊断错误，不触碰 Steam、助手或未知进程。
- Windows 进程快照增加只读 `exePath` 与 `startTime` 字段；映像路径通过 `QueryFullProcessImageNameW` 获取，启动时间通过 `GetProcessTimes` 获取，权限不足时保留 PID 并将字段置空。
- 修正窗口枚举回调，明确区分“已发送 WM_CLOSE”和“没有可发送窗口”。

验证：

```text
cargo check --manifest-path src-tauri/Cargo.toml -q                 # 通过
cargo test --manifest-path src-tauri/Cargo.toml cs2 --lib           # 20 passed; 0 failed; 1 ignored
```

`cs2` 目标测试中的 1 项为要求真实 CS2 根目录的环境测试，按现有 ignore 规则未执行；其余进程匹配、事务回滚、安装和发现逻辑均通过。完整真实 Tauri/Windows 关闭矩阵、窗口截图、用户电脑 CS2/Steam 均关闭回读和 10 次备份清单仍未执行，专项状态继续保持 **部分实现/待真实验收**。

当前执行机现场回读（PowerShell `Get-Process -Name cs2,steam`）结果为：

```text
NO_CS2_OR_STEAM_PROCESS
```

该结果仅证明采样时本机没有名为 `cs2` 或 `steam` 的进程，尚未通过真实 Tauri 窗口状态回读门禁。

## 2026-08-28 前端关闭控件回归

新增 [tests/status-strip-close.spec.ts](/E:/CS2AS05/tests/status-strip-close.spec.ts)，覆盖：

- `checking`、`stopped`、`unknown` 状态禁用关闭按钮；`running` 状态可操作；
- `aria-label="关闭 CS2"` 契约；
- 优雅关闭返回未成功时展示应用内强制关闭确认；
- 只有点击确认后才调用 `closeCs2(true)`；
- 关闭请求进行中锁定按钮，阻止重复请求。

验证：

```text
npm test -- --run tests/status-strip-close.spec.ts
1 个测试文件通过；6 个测试通过
npm run typecheck
通过
```

该测试使用受控 IPC mock，仅证明前端状态与交互契约；真实 Tauri 窗口、Windows 进程和截图验收仍未完成，专项状态仍为 **部分实现/待真实验收**。

## 2026-08-28 启动清理触发点补充

`initialize_panel_defaults_at_with_running` 现在在确认 CS2 未运行后，先对当前 `game/csgo` 执行一次受限 `cleanup_owned_backups`。清理继续遵循现有路径和命名保护，不进入普通 `backup` 目录；清理失败仅告警式忽略，不阻塞 Panel 初始化。

完整 Panel 目标测试复核结果：18 项通过、1 项真实环境测试忽略、0 项失败；`mutations_converge_to_the_aggregated_disk_snapshot` 已通过补齐受控 sidecar 夹具恢复绿色。

## 2026-08-28 安装备份路径回读补充

安装事务函数现在返回 `Option<PathBuf>`：

- `keep_backup=false`：成功后删除事务 backup 目录并返回 `None`；
- `keep_backup=true`：保留本次事务 backup 目录，并由 `install_bot_package` 将绝对路径写入 `OperationResult.message`，方便用户定位恢复材料。

新增受控回归 `transactional_install_returns_retained_backup_path_when_requested` 已通过，确认保留路径为绝对路径且目录实际存在。验证命令：

```text
cargo test --manifest-path src-tauri/Cargo.toml transactional_install_returns_retained_backup_path_when_requested --lib
1 passed; 0 failed
```

最终本轮定向复核：

```text
npm test -- --run tests/status-strip-close.spec.ts tests/cs2-process-polling.spec.ts
2 个测试文件通过；10 个测试通过
cargo check --manifest-path src-tauri/Cargo.toml -q
通过
git diff --check（本轮涉及文件）
通过
```

## 2026-08-28 关闭日志审计补充

关闭 IPC 已增加脱敏 runtime.log 阶段记录：

- `[CS2_CLOSE_BEGIN]`：记录 `force` 阶段；
- `[CS2_CLOSE_SNAPSHOT]`：记录 `observedAt`、`sampleCount`、`confidence` 和 PID 列表；
- `[CS2_CLOSE_SIGNAL]`：逐 PID 记录 `wm_close` 或 `terminate` 阶段；
- `[CS2_CLOSE_FINAL]`：仅在最终进程列表为空时记录成功；
- `[CS2_CLOSE_TIMEOUT]`：超时记录仍存在的 PID 列表。

日志不记录凭据、命令行参数或用户文件内容。`cargo check --manifest-path src-tauri/Cargo.toml -q` 已在该变更后通过。

## 2026-08-28 桌面构建验证

执行 `npm run build:desktop` 成功，产物为：

```text
E:\CS2AS05\src-tauri\target\release\CS2BotImproverAssistant.exe
```

该构建验证覆盖 Vite Web 产物、Tauri Rust 编译、Windows API features 和命令注册的联合编译；构建过程未执行发布、安装或上传。

## 2026-08-28 真实 Tauri 窗口与状态条布局证据

启动 release 产物后，真实窗口现场回读为：

```text
PID: 3088
标题: CS2 人机增强助手
Responding: True
窗口矩形: 51,52,1505,960
```

首次截图暴露顶部状态条在长 CS2 路径下可能把右侧状态项推出可视区。已将 `.status-path` 调整为 `width: 0; flex: 1 1 0%` 并允许状态条溢出可见，使路径优先省略、CS2 状态和关闭按钮保留空间。

重新构建并启动后，截图证据保存为：

```text
E:\CS2AS05\artifacts\tauri-window-status-strip-final-20260828.png
SHA-256: 4741AB8B975A3F8F2B03927FFADBF84E731ED781138409BBBA6FCBC582593FD2
```

该截图确认真实 Tauri 窗口成功渲染且进程响应正常；仍未通过真实 CS2/Steam 运行矩阵或关闭按钮点击验收。测试进程已在截图后关闭。

## 2026-08-28 窄窗口主工作台现场补充

同一 release 进程（PID 3928，标题 `CS2 人机增强助手`，`Responding=True`）已进入主工作台后调整窗口尺寸并截图：

```text
E:\CS2AS05\artifacts\tauri-window-980x640-main-20260828.png
SHA-256: 106373ACF9B16E14A684311F95141EA82932282789E5129C8361BB8F887DFC2F
窗口尺寸: 980x640

E:\CS2AS05\artifacts\tauri-window-640x640-current-20260828.png
SHA-256: 95B3815FB7155DE6E10DBB3076921B65FB5CF6226A18F2A1B6E55029A93C731A
窗口尺寸: 640x640
```

两张图均确认真实 Tauri 主工作台可启动、导航和状态条能够渲染；640x640 图中路径区域已优先省略。980x640 图仍观察到状态条右侧内容在窗口边界处被截断，关闭按钮未完整出现在可视区域，因此窄窗口横向布局门禁仍为 **待修复/待验收**，不能宣称响应式状态条专项完成。该现场只证明桌面渲染，不替代真实 CS2/Steam 运行、优雅关闭、强制关闭和用户电脑最终回读。

## 2026-08-28 本轮结束前最小回归

```text
npm test -- --run tests/status-strip-close.spec.ts tests/cs2-process-polling.spec.ts
2 个测试文件通过；10 个测试通过

npm run typecheck
通过

cargo check --manifest-path src-tauri/Cargo.toml -q
通过（仅保留既有 third_party 警告）

git diff --check
通过
```

本轮真实窗口进程已停止。真实 CS2/Steam 关闭矩阵、优雅关闭和强制关闭点击、10 次安装备份清单，以及用户电脑最终状态回读仍未完成；专项状态保持 **部分实现/待真实验收**。
