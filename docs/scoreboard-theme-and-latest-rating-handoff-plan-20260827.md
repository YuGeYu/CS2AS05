# 独立战报外观跟随设置与最新 Rating 交接方案

日期：2026-08-27  
交接对象：下一个【实际执行 AI】  
范围：修复独立“本局战报”窗口的外观设置同步，将评分模型升级为 LBRating 2.0（机器版本 `lb-rating-2.0`），并按本次新增要求重做表现雷达的轴契约、全场相对归一化、几何布局和右侧排名展示。不得借此重做 Demo 解析、比分判定或 Viewer。

## 1. 目标与完成标准

### 1.1 外观

战报窗口必须配合助手当前设置，不能固定为暗色。至少支持现有设置中的浅色/深色主题，并同步现有外观偏好中的调色板、圆角和密度（如这些设置确实影响主界面；不应凭空增加战报专属设置）。主窗口切换设置后再次打开战报应使用新设置；战报已打开时若跨窗口事件无法可靠传播，必须在下一次打开时读取最新持久化设置并在方案报告中说明限制，不能声称实时同步。

右下角/页脚继续保留 Rating 模型标识，显示 `LBRating 2.0 · lb-rating-2.0`；不得继续显示 `simple-rating-v1` 或 `simple-rating-v2` 作为当前模型。

### 1.2 Rating

本次目标模型是 LBRating 2.0，显示名与机器版本分离：显示名 `LBRating 2.0`，机器版本 `lb-rating-2.0`。实现仍位于 `src-tauri/src/services/simple_rating.rs`，由 `src-tauri/src/services/demo.rs` 的 `METRICS_VERSION = "lb-rating-2.0"` 驱动报告缓存和解析。执行 AI 必须按本方案的新公式实现。

完成必须满足：

1. `scoreboard.html` 独立入口初始化与主程序相同的外观偏好数据属性。
2. `scoreboard.css` 在浅色和深色下均有完整可读配色，不再依赖暗色硬编码；表格、边框、标题栏、状态、Rating 高中低和错误提示在两主题均有足够对比度。
3. 页脚保留 `LBRating 2.0 · lb-rating-2.0 · ADR 主导的多因子模型`，UI、DTO、测试夹具和入口版本检查一致；运行时不得出现旧 simple-rating 或 `openRating` 作为当前模型。
4. 后端计算使用 LBRating 2.0 公式、缺失输入返回 unavailable、结果范围 `0..3`，不以 UI 修复掩盖算法错误。
5. 旧缓存/旧报告不会被当成 LBRating 2.0 直接展示；`ScoreboardApp.vue` 的 metricsVersion 门禁必须改为 `lb-rating-2.0`。
6. 完成自动化测试、浅/深色截图或 DOM 证据；真实 Windows/Tauri 玩家视觉验收若未执行必须明确标记待用户确认。

## 2. 已调查基线与问题定位

- 独立入口：`src/scoreboard.ts` 仅导入 `scoreboard.css` 并挂载 `ScoreboardApp`，没有调用 `initializeTheme()` 或 `initializeAppearancePreferences()`。
- 主程序入口：`src/app/create-app.ts` 调用 `initializeTheme()`；`src/App.vue` 调用 `initializeAppearancePreferences()`，后者设置 `data-theme`、`data-palette`、`data-radius`、`data-density`、`data-sidebarMode`。
- 独立窗口组件：`src/ScoreboardApp.vue` 当前校验 `schemaVersion < 6` 或 `metricsVersion !== 'simple-rating-v2'`；实施时必须保留门禁并改为 `lb-rating-2.0`，拒绝全部旧 simple-rating 报告。
- 战报样式：`src/styles/scoreboard.css` 的 `:root`、`.scoreboard-shell`、标题栏、表格、footer 等全部是 `#111214/#1d1f22/#444950` 等暗色固定值，没有 `data-theme` 分支或共享语义变量。
- 战报组件：`src/components/scoreboard/PostMatchScoreboard.vue` 页脚仍硬编码旧的 `simple-rating-v1`，必须改为 LBRating 2.0。
- 后端：`src-tauri/src/services/simple_rating.rs` 的 `MODEL_VERSION` 当前为 `simple-rating-v2`；本次必须改成 LBRating 2.0 的 ADR 主导权重。`src-tauri/src/services/demo.rs` 的 `METRICS_VERSION` 也必须同步。
- 现有 `tests/post-match-scoreboard.spec.ts` 测试夹具仍使用 `simple-rating-v1`，而 `tests/release-blocker-recovery.spec.ts` 已要求独立入口接受 `simple-rating-v2`；本次必须统一为 `lb-rating-2.0`，并增加旧 v1/v2 均拒绝断言。
- 旧 `docs/simple-rating-release-handoff-0.5.5-20260729.md` 描述的是历史 v1 方案，不能覆盖当前运行代码事实；执行报告要把它作为历史资料，不要按其中 v1 公式回退。

## 3. 外观实现方案

### 3.1 入口初始化

修改 `src/scoreboard.ts`，在创建 Vue 应用前调用与主程序等价的初始化函数。推荐直接调用 `initializeAppearancePreferences()`；它会读取现有外观持久化记录并设置所有 `data-*` 属性，同时应用主题。不要只调用 `initializeTheme()` 后声称完整设置已同步。

如果该模块依赖主应用导航或产生不适合独立入口的副作用，执行 AI 必须先用类型检查和入口构建验证；可抽取一个无导航副作用的共享 `initializeAppearanceForDocument()`，但不得复制两套解析规则。保留现有设置存储键和迁移行为，不新建第二套设置文件。

### 3.2 CSS 语义变量

优先方案是把可复用的颜色/尺寸语义变量抽到独立共享 CSS（例如 `src/styles/theme-tokens.css`），由主窗口和 `scoreboard.css` 同时导入；若为控制范围暂不抽取，至少在 `scoreboard.css` 定义完整的 `:root` 浅色默认变量和 `:root[data-theme='dark']` 覆盖，并按现有 `data-palette` 对主色做映射。不能继续把暗色十六进制直接散落到组件规则。

至少覆盖这些语义：`--scoreboard-bg`、`--scoreboard-surface`、`--scoreboard-surface-muted`、`--scoreboard-raised`、`--scoreboard-text`、`--scoreboard-text-muted`、`--scoreboard-border`、`--scoreboard-border-strong`、`--scoreboard-primary`、`--scoreboard-primary-contrast`、`--scoreboard-warning`、`--scoreboard-danger`、`--scoreboard-shadow`。

浅色示例应是明亮背景、深色文字、清晰边界；深色保留当前灰色科技风。不要做紫色渐变、装饰性大卡片或改变数据密集型布局。Rating 高/中/低除颜色外必须继续有“高/中/低”文字，保证色觉可访问性。

### 3.3 偏好属性映射

- `data-theme=light|dark`：切换背景、文字、边框、标题栏、表头和状态颜色。
- `data-palette`：沿用主程序现有调色板语义映射主色/强调色；未知值回退 default。
- `data-radius`：将现有 `auto/0/0.25/0.5/0.75/1.0` 映射到战报边框/按钮/表格容器的圆角；不改变固定窗口尺寸。
- `data-density`：映射标题栏、表格行高、内边距的 compact/default/loose 三档，同时保证 `980x640` 最小窗口仍可滚动且无文字重叠。
- `data-sidebarMode` 对独立战报无布局意义，可读取但不必应用。

不要在战报窗口重新持久化设置，也不要从战报窗口提供第二个主题开关；设置的唯一编辑入口仍是主程序外观设置。

### 3.4 跨窗口更新边界

主窗口设置改变时，若同源 WebView 可收到 `cs2as:theme-changed`，战报文档可监听该事件并调用共享应用函数；但不要假设 Tauri 独立窗口一定共享 Vue 内存。最可靠契约是：每次战报窗口启动/显示前读取持久化设置。不得引入浏览器 `storage` 事件作为唯一机制，也不得新增 localStorage 数据格式；沿用项目已有设置存储（这里只读/复用，不扩大用途）。

## 4. LBRating 2.0 一致性实施方案

### 4.1 单一权威

保留并核对：

```text
src-tauri/src/services/simple_rating.rs::MODEL_VERSION = "lb-rating-2.0"
src-tauri/src/services/demo.rs::METRICS_VERSION = "lb-rating-2.0"
src/ScoreboardApp.vue 的 metricsVersion 门禁 = "lb-rating-2.0"
```

所有当前模型展示统一写产品名 `LBRating 2.0`，机器字段统一写 `lb-rating-2.0`。`PostMatchScoreboard.vue` 页脚建议文案：

```text
LBRating 2.0 · lb-rating-2.0 · ADR 主导的多因子模型
```

如产品希望更严谨，可补充“非 HLTV 官方模型”，但不能写成 HLTV/OpenRating，也不能保留 simple-rating 版本作为当前模型。

### 4.2 LBRating 2.0 算法与数据质量

执行 AI 应将 `simple_rating.rs` 改为 ADR 主导的确定性公式。输入仍为 K/D/A、damage 与 completed rounds，五项均存在且 R>0 才计算；缺失或零回合返回 unavailable：

```text
ADR = DMG / R; KPR = K / R; APR = A / R; DPR = D / R
adrComponent      = clamp(ADR / 82.0, 0.0, 3.0)   # 主导
killComponent     = clamp(KPR / 0.70, 0.0, 2.5)
survivalComponent = clamp((1.0 - clamp(DPR, 0.0, 1.0)) / 0.68, 0.0, 2.5)
assistComponent   = clamp(APR / 0.20, 0.0, 2.5)
rating = clamp(0.55 * adrComponent + 0.20 * killComponent
  + 0.15 * survivalComponent + 0.10 * assistComponent, 0.0, 3.0)
```

ADR 权重为 55%，高于任何单项 K/D/A 辅助项；Rust 使用 f64，组件和最终值四舍五入四位，UI 显示两位。增加 average、zero、ADR 主导和 invalid 固定向量，并验证结果始终在 0..3。不得用 0 补输入、不得把 performance radar 当成 Rating。

average 向量 `K=14,D=6,A=4,DMG=1640,R=20` 的期望值为 `rating=1.0044`（未四舍五入约 1.0044118）；这是新权重下的基准，不得沿用旧算法的其他期望值。`DemoReport`/SQLite 的 metrics cache 必须以 `lb-rating-2.0` 失效旧 simple-rating 报告；如现有 schema 版本只表达字段结构而不能表达模型变更，至少确保所有 cache-hit 查询同时检查该 metrics 值，并为旧报告排队重解析。

确认 `src-tauri/src/services/demo.rs` 仍在拥有 completed rounds 后才计算 LBRating，报告级 ratingStatus、玩家级 unavailable、缓存 metricsVersion 门禁切换到 `lb-rating-2.0`。结果不一致时优先修后端/迁移，不在前端四舍五入层修正。

### 4.3 前端与测试夹具

更新 `tests/post-match-scoreboard.spec.ts` 的 DemoReport 和玩家 DTO 夹具：`metricsVersion`、`modelVersion` 改为 `lb-rating-2.0`，并使用 LBRating 2.0 固定向量结果；保留排序、MVP、两位小数、档位、不可用、报告回合和先进指标暂不提供断言。

增加断言：

- 页脚包含 `LBRating 2.0` 与 `lb-rating-2.0`，不包含 simple-rating 或 `openRating`。
- `ScoreboardApp.vue` 对 simple-rating v1/v2 报告均拒绝并触发错误，不得静默展示旧报告。
- `simple_rating.rs` 的 LBRating 固定向量、ADR 主导排序、缺失输入、零回合和 clamp 测试通过。
- 全仓库当前运行代码（排除历史方案文档）不得残留会影响运行的 v1 常量；历史文档可保留，但执行报告需说明其已过时。

## 5A. 表现雷达逻辑与布局专项（新增交接要求）

### 5A.1 六轴位置锁定与命名

雷达必须是“竖立”的正六边形：1 号轴在正上方（坐标角度 `-90deg`），之后按顺时针每 60 度排列，4 号轴在正下方。轴顺序不可由数据返回顺序、字母排序或玩家选择顺序改变，固定为：

```text
1 KPR         回合击杀
2 surviving   回合存活
3 ADR         回合伤害
4 KAST        贡献率
5 impact      影响力
6 rating      LBRating 2.0 · lb-rating-2.0
```

建议机器 key 直接统一为 `kpr | surviving | adr | kast | impact | rating`，显示 label 统一为上述中文/英文组合。同步修改 `src-tauri/src/models/demo.rs`、`src/types/demo.ts`、`src/features/demo/radar/performance-radar.ts`、`MatchPerformanceRadar.vue`、`PerformanceRadarChart.vue` 及相关测试。保留 `performance-radar-v1` 作为接口模型版本可以接受，但六轴不得继续使用旧的 `firepower/damage/survival/participation/teamwork/opening` key。

### 5A.2 全场第二名归一化算法

“满格”按**本场所有有效参赛玩家、逐轴独立**计算，不按某一个玩家的六项最大值，也不只按当前选中的 1-3 人计算。对每个轴 `d`：

1. 收集全场 T/CT 有正式回合且该轴 raw 有限的玩家值，按数值降序排列。
2. 去重后取第二高值 `secondHighest_d`；若只有一名有效玩家，使用该唯一值作为满格；若无有效值，该轴整体 unavailable。
3. 当前玩家图形比例 `ratio = raw_d / secondHighest_d`。`raw_d == secondHighest_d` 显示满格 100；第一名若为 150、第二名为 100，则显示 150（超出六边形一半）；其他玩家按比例缩短。
4. `secondHighest_d <= 0` 时不能除零：若 raw 同为 0，显示 0 并标记 `flat/partial`；若存在正值，满格基准取正值；负值或非法值不可进入比例。
5. 不要对 ratio 做 100% 截断。允许合理上限（例如 4.0 或 500%）仅作为防止异常数据撑爆 DOM 的安全上限，并把“已安全截断”写入 warning；正常第一名超出应原样保留。

归一化必须在所有玩家集合上完成，再将归一化结果应用到 selected players；切换选择人时基准不得变化。推荐后端返回每个 dimension 的 `raw` 和 `benchmark`（此时 benchmark 是实际 `secondHighest_d`），并在 `MatchPerformanceRadar` 响应增加 `cohortSize`/`benchmarkMethod: "second-highest-per-axis"`；也可前端纯函数计算，但必须把完整 roster 一并保留，禁止用已筛选数组重新计算基准。`score` 建议为百分比数值（100=第二高、150=第一名示例），而 SVG 几何接受同一数值并允许越界。

### 5A.3 六轴原始数据来源

后端 `calculate_performance_radar` 不得再使用固定 0.80、0.50 等 benchmark。六轴 raw 统一按报告已完成回合计算：

```text
KPR       = kills / participated_rounds
Surviving = survived_rounds / participated_rounds
ADR       = damage_health / participated_rounds
KAST      = kast_rounds / participated_rounds
Impact    = 可审计的现有影响字段；优先 firstKills、tradeKills、multiKillRounds 等已落库数据，缺少可靠来源则 null/unavailable
Rating    = DemoPlayer.rating.rating，且 modelVersion 必须是 lb-rating-2.0
```

Impact 必须在响应中说明公式、单位和 source；不能把任意固定常数冒充影响力。若某玩家/某轴缺数据，保留 null，雷达按现有规则断开该轴/显示不可用，不用 0 补齐。Rating 轴不能重新计算另一套评分，必须读取 LBRating 2.0 的最终值。

### 5A.4 前端几何与用户缩放

修改 `src/features/demo/radar/performance-radar.ts`：

- 删除 `RADAR_MIN_DISPLAY_MAX=140` 作为满格逻辑，也不要把 `RADAR_SAFE_SCORE_MAX=500` 当作第二名基准；保留独立的异常保护上限即可。
- `radarScaleMax` 默认使用 100 作为六边形满格，允许 score>100 向外；网格至少显示 25/50/75/100，必要时按当前最大可见 ratio 增加 125/150 等参考圈，但 100 圈必须清晰标识“本轴第二高”。
- `radarPoint` 的角度固定从 -90 度开始，顺时针递增；不要因 viewBox 或 CSS transform 把 1 号位旋到左侧。
- SVG 容器必须 `overflow: visible`，外扩图形不能被 figure、workspace 或父级 `overflow:hidden` 裁掉。若与标题/表格遮挡，雷达图层使用更高 stacking context（明确 `position`/`z-index`），按用户要求允许覆盖普通 UI；仍不得遮住关闭/错误等关键操作控件。
- 增加用户可调缩放控件（推荐 range slider + `-`/`+`/重置按钮），上下限要能覆盖异常但有限：例如 `displayScale` 0.75..2.0，步进 0.05，默认 1。缩放只改变 SVG 绘图区半径/容器尺寸，不改变 raw、second-highest benchmark 或 score；键盘可操作并有 aria-label/当前百分比。
- 不能用浏览器缩放或 CSS transform 让文本失控；固定 viewBox 与响应式容器，检查 980px 和移动宽度无横向溢出。超出六边形的第一名应仍可读，标签与数据表同步。

### 5A.5 右侧排名/PK 展示

将当前 `.radar-insight` 的“读图基准”说明替换为右侧展示区域，保留警告但不要让警告挤掉主体：

- **仅一名玩家**：显示玩家名称、队伍/BOT 标识、当局美观排名（默认按 `LBRating 2.0` 降序；Rating 缺失时按有效轴得分总和/平均比例降序并明确“Rating 不可用”）。显示 `第 N / M 名`，并列使用稳定 key 作为次排序。前三名使用明显但不喧宾夺主的金/银/铜视觉层级，不能只靠颜色，需有“冠军/亚军/季军”文字或图标语义。
- **多名玩家**：显示美观的 PK 对比布局（每名玩家一列/一行、颜色线型与雷达 polygon 对应），每人显示排名、名称、LBRating、胜出轴数量或相对领先提示。最多三人沿用现有选择上限；禁止把 PK 排名误写为正式比赛名次。
- 排名必须基于完整有效参赛 roster 计算，而不是只在 selected players 内重新排名；展示区域可标注“本场表现排名”。
- 右侧区域在雷达超出时允许被雷达覆盖，但普通内容应通过 grid/flex 分区避免重叠；遮挡发生时以雷达可见性优先，并将右侧信息提升到前景或提供紧凑模式。所有名称、排名和数值必须在移动宽度换行，不得溢出。

### 5A.6 表格与可访问性

精确数据表的列改为：轴位（1-6）、维度、玩家、图形比例、原始值、全场第二高基准、数据质量。`benchmark` 文案写明“本场第二高”，不再写固定基准。SVG `title/desc` 说明竖立六边形、轴顺序、第二名满格和第一名允许越界；每个 polygon/PK 项有玩家名称、当局排名和模型版本的 aria-label。颜色、线型、排名文字三者同时表达状态。

## 6. 验证计划与证据

### 6.1 自动化

```powershell
npm test -- --run tests/post-match-scoreboard.spec.ts tests/release-blocker-recovery.spec.ts tests/theme-preference.spec.ts tests/appearance-preferences.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml simple_rating
```

若全量 lint/test 被工作树既有生成物阻塞，记录准确失败文件和命令，不删除或回退既有文件；至少运行与本任务直接相关的定向测试。

### 6.2 视觉/交互矩阵

用实际 Tauri/受控浏览器页面或项目已有截图工具验证独立战报（不是只看源码）：

| 设置 | 窗口尺寸 | 必查 |
|---|---:|---|
| 浅色 + default + default density | 1280x800 | 标题栏、比分、表格、footer、Rating 对比度 |
| 深色 + default + default density | 1280x800 | 同上，不能白屏/出现浅色文字丢失 |
| 浅色 + 高密度/compact | 980x640 | 无重叠，表格可滚动，footer 可见 |
| 深色 + loose 或非 default palette | 980x640 | 圆角/颜色/行高随设置，仍无水平溢出 |

至少保存每种主题一张截图和页面 `document.documentElement.dataset` 读回；截图中右下角必须可见 `LBRating 2.0` 和 `lb-rating-2.0`。不得用旧战报窗口或旧构建产物证明新代码。

### 6.3 真实数据门禁

使用当前 schema/metrics 均匹配的真实 Demo 或用户指定样本，确认：玩家 Rating 数值来自后端 LBRating 2.0，ADR 较高且 K/D/A 普通的玩家不会被旧 K/D 权重错误压低，排序/MVP 正确，缺失输入显示 `-- 不可用`。旧 simple-rating v1/v2 报告均应触发“报告版本过旧，请重新解析”。

## 6. 不纳入本次与停止条件

- 不恢复 simple-rating v1/v2 作为当前公式，不重做 performance radar，不新增 KAST/Swing/economy 指标。
- 不改独立窗口 label、尺寸、关闭协议、`scoreboard://load-report`、Tauri 命令或 Demo 解析流程。
- 不因为 UI 任务重建安装器、上传 GitHub/R2、部署网站或升版本。
- 如果外观偏好在独立窗口无法读取，先停止声称“已跟随设置”，记录入口错误和最小复现；不能退回暗色硬编码作为最终方案。
- 如果报告缓存仍大量为 simple-rating v1/v2，必须走现有重新解析/版本失效路径；不能批量直接改数据库 JSON 冒充 LBRating 2.0 结果。

## 7. 交付回传清单

实际执行 AI 应回传：修改文件绝对路径；初始化方式和支持的 `data-*` 设置；浅/深色及尺寸截图路径；页脚 `LBRating 2.0 · lb-rating-2.0` 读回；ADR 主导公式固定测试、排序证据和报告门禁结果；旧 simple-rating v1/v2 报告拒绝证据；未完成的真实 Tauri/CS2 用户验收；以及工作树中新增但未提交的构建/证据产物。
