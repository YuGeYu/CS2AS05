# 表现雷达排名、缩放语义与赞助模态框修复交接方案

日期：2026-08-31  
工作区：`E:\CS2AS05`  
角色：方案制定 AI 交给实际执行 AI  
状态：已调查，待执行；本轮只新增方案，不代表代码或真实 Demo 已修复。

## 1. 用户现场与完成标准

### 1.1 表现雷达排名没有遵循 Rating

目标文件：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260826-1404-de_dust2-advent.dem
```

用户截图 `C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-08-28 203235.png` 显示表现雷达把 `advent` 排为“冠军 1 / 10”。这里的目标不是把 `advent` 判为观战者，也不是修复某一个 Demo 的特殊身份；`advent` 在该 Demo 中是正常参赛者。真正的问题是：前面的 Rating 计算已经表明 `advent` 不是 Rating 第一，但雷达右侧排名仍以另一套分数/维度总和把他排成冠军。助手必须从逻辑上保证 Rating 越高，表现排名越高，并且不区分 BOT 与真人。

完成标准：

- 指定 `.dem` 只作为回归样本，保留 `advent` 为正常参赛者；不得根据名字、是否 BOT 或真人身份做特殊排除；
- 表现排名的主排序键必须是既有最终 Rating（当前产品模型为 `lb-rating-2.0` 的 `DemoPlayer.rating.rating`）；Rating 越高排名序号越靠前；
- Rating 缺失时才允许使用明确标注“Rating 不可用”的降级排序，并保持稳定次序；不得用雷达轴总和覆盖一个存在的 Rating；
- BOT 与真人使用同一排名规则、同一 roster 和同一奖项语义；
- 对该 Demo，执行 AI 必须回读所有玩家的 Rating 与排名，证明排序单调一致：任意两名均有 Rating 的玩家，Rating 高者不能排在 Rating 低者之后。

### 1.2 缩放按钮语义错误

用户截图 `C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-08-29 105729.png` 显示雷达的 `-`、range、`+`、百分比控件。当前实现 `PerformanceRadarChart.vue` 通过 `svg :style="{ width: `${displayScale * 100}%` }"` 改变整个 SVG 宽度，导致控件改变图形在布局中的尺寸/位置，用户需要的是“雷达大小”而不是把雷达拖动或让布局发生偏移。

完成标准：

- 缩放只改变六边形绘图区半径/图形大小，中心点和轴位保持固定，父级布局、右侧排名区、控件位置不随缩放值漂移；
- raw、benchmark、score、排名和数据请求完全不因缩放改变；
- 保留可访问 `range`、减小、增大、重置和百分比读数，范围 `0.75..2.0`、步进 `0.05`、默认 `1.0`；
- 允许 score > 100 的 polygon 向外显示，SVG 及必要父级 `overflow: visible`，不得被容器裁切；
- 980x640 与移动宽度下控件和图形仍可用，不借浏览器页面缩放实现。

### 1.3 安装与诊断赞助入口

在安装与诊断页增加“赞助开发/支持维护”按钮。用户点击后打开覆盖全屏的模态框，展示官网项目中的微信赞赏码，模态框可通过关闭按钮、Esc 和背景点击关闭；赞赏不影响安装、诊断、更新、登录或任何功能。

## 2. 已调查代码事实与根因方向

### 2.1 雷达当前实现

- 类型：`src/types/demo.ts` 仍定义旧六轴 `firepower/damage/survival/participation/teamwork/opening`，模型版本 `performance-radar-v1`。
- 前端：`src/features/demo/components/MatchPerformanceRadar.vue` 的 `rankValue()` 读取名为 `rating` 的雷达维度 raw，找不到时才把各维度 score 求和；这会把雷达维度中的 raw/缺失值当成最终 Rating，且没有验证 `modelVersion`、Rating 可用性或与报告级玩家 Rating 的一致性。
- 当前实现没有把“Rating 排名”作为不可覆盖的主排序契约，也没有测试“Rating 高者必在前”；因此 `advent` 可能因火力/协同等雷达轴总和较高而错误夺冠。
- 本轮不修改 BOT/真人分类，不把 `advent` 改为 observer；仍需保留正常 CT/T roster 和现有 observer/unknown 处理，但排名规则对所有可排名玩家统一。
- `src/features/demo/components/PerformanceRadarChart.vue` 用改变 SVG `width` 的方式缩放，天然会改变 grid 布局占位和视觉位置；应改为固定 viewBox/容器内半径或有界 `transform`，中心不得移动。
- `src-tauri/src/services/demo.rs` 的报告聚合曾以 `team_number` 判断 eligible；当前必须复核指定 Demo 的 controller props、`player_team` 事件、正式回合 kills/deaths/damage 和 observer 证据，不能只沿用 `team_number`。

### 2.2 官网赞赏码来源

已在官网项目找到现成资源：

```text
E:\cs2as\public\assets\wechat-reward.png
```

官网 `E:\cs2as\src\pages\HomePage.vue` 已以 `<img class="reward-code" src="/assets/wechat-reward.png" alt="微信赞赏码" />` 展示该图片。执行 AI 应将该公开项目资源复制到本项目受管静态资源目录（建议 `public/assets/wechat-reward.png`），记录源路径、文件大小和 SHA-256；不要运行时依赖官网页面、二维码 URL、localStorage 或浏览器下载。

## 3. 实际执行方案

### P0：先修 Rating 主导的统一排名

1. 对指定 Demo 做只读基线：记录文件大小、SHA-256、header、解析结果、报告 JSON hash、数据库 demo id；确认 `advent` 是正常参赛者，不修改原始 `.dem`。
2. 在后端 DTO/服务层明确唯一排名字段：`rankingRating` 必须直接来自该玩家最终 `DemoPlayer.rating.rating`，且 `modelVersion === "lb-rating-2.0"`。不要从六轴 raw、雷达 score 或 UI 文本重新计算 Rating。
3. 实现共享纯函数 `rankPerformancePlayers(players)`：
   - 先过滤现有合法参赛 roster，但不区分 `isBot`；
   - 对有有效 LBRating 的玩家按 `rating` 降序；
   - Rating 相同按稳定 `stableKey` 次排序；
   - 只有 Rating 缺失的玩家才进入明确的降级组，按有效雷达轴比例平均排序，并标记“Rating 不可用”；
   - 任何情况下不能让降级分数超过一个存在的 Rating，也不能让低 Rating 排到高 Rating 前面。
4. `MatchPerformanceRadar.vue` 的 `rankValue()` 改为消费后端/报告提供的最终 Rating 字段，不再查找雷达 dimensions 的 `rating` raw；若 DTO 尚无字段，先补齐类型和映射，再接线。
5. 完整 roster benchmark 计算仍与 selected 展示分离；selected 只决定 polygon/表格展示，不改变排名。BOT 和真人同样进入 roster、同样参与排名和前三名语义。
6. 用指定 Demo 重新解析或失效旧缓存，回读所有玩家的 `rating`、`modelVersion`、`rankingRating` 和 `rank`，证明 `advent` 不再因雷达轴总和夺冠；最终冠军必须是 Rating 最高者。若 Rating 数据缺失或模型版本不一致，显示“排名不可用/Rating 不可用”，不得猜测。

### P0：把缩放改成只改图形大小

1. 在 `src/features/demo/radar/performance-radar.ts` 保留纯函数几何契约：六轴顺序、顶部/底部、顺时针、score>100 不截断。
2. 在 `PerformanceRadarChart.vue` 使用固定 `viewBox="0 0 640 640"`、固定外层绘图区和中心 `(320,320)`；将 `displayScale` 传入 `radarPoint/radarGrid/radarPolygon` 的半径，或只对内部 radar layer 做以中心为原点的有界 `transform`。禁止再绑定 SVG 百分比 `width` 造成 grid item 改位。
3. 轴标签、控件、figcaption 保持固定布局；图形放大可覆盖普通信息但不得遮挡关闭、错误和键盘焦点控件。必要时为雷达建立 stacking context，并将右侧排名区置于可读层。
4. range 的 `input`、`-`、`+`、重置按钮均更新同一个 `displayScale`，不触发 `getMatchPerformanceRadar`；显示 `75%..200%`，边界和键盘操作有 aria-label/aria-valuenow。

### P1：安装与诊断赞助全屏模态框

1. 入口组件：`src/components/SupportActions.vue` 位于 `src/views/InstallView.vue` 的安装与诊断页面；增加 Lucide `Heart`/`HandHeart` 图标按钮，文案如“赞助开发”。不新增重复导航。
2. 新增可复用 `src/components/DonateModal.vue`（或在 `SupportActions.vue` 内保持清晰边界）：使用 Teleport 到 body、`role="dialog"`、`aria-modal="true"`、标题“支持助手维护”、图片 `src="/assets/wechat-reward.png"`、`alt="微信赞赏码"`、简短友好说明“不赞助不影响任何功能”。
3. 全屏模态框使用现有 `.modal-backdrop` 语义和主题变量，内容区域不嵌套多层卡片；图片设置稳定最大尺寸、`object-fit:contain`、移动端不溢出；亮色/深色均可读。
4. 关闭契约：右上角图标按钮（`aria-label="关闭赞助窗口"`）、Esc、背景点击；打开时将焦点放入标题或关闭按钮，关闭后恢复触发按钮焦点；禁止 body 背景滚动。不得写入 localStorage。
5. 资源复制后核对与官网源文件大小/SHA-256 一致；资源缺失时显示可读错误占位，不渲染破图，也不能阻塞安装页。

## 4. 测试与验证

### 4.1 雷达/指定 Demo

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/performance-radar.spec.ts tests/match-performance-radar.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml performance_radar demo
```

新增断言：

- 指定 Demo 中 `advent` 保持正常参赛者和可选玩家身份；其排名必须服从实际 `lb-rating-2.0` Rating，而不是被名字或真人/BOT 属性特殊处理；
- 任意有 Rating 的玩家对满足 `ratingA > ratingB` 时必须有 `rankA < rankB`；Rating 相同使用 stableKey 稳定排序；
- Rating 缺失只允许进入降级组并明确显示“Rating 不可用”，不得覆盖已有 Rating 排序；
- 完整 roster 排名独立于 selected；Rating 缺失降级排序和并列稳定；
- 缩放 0.75/1/2.0 边界、0.05 步进、键盘、aria；缩放前后中心点/轴位/控件 bounding rect 不变，只有 polygon/grid 半径改变；
- score 150 等越界仍可见、不被 SVG/父容器裁切。

实际 Demo 证据必须保存到 `E:\CS2AS05\artifacts\performance-radar-rating-ranking-fix-20260831\`，至少包含：只读基线 hash、解析 roster JSON、最终 radar JSON、Rating/排名对账输入、截图和命令输出。

### 4.2 赞助模态框

新增 `tests/donate-modal.spec.ts` 与 SupportActions 断言：

- 安装与诊断页存在“赞助开发”按钮；点击后出现全屏 `role=dialog` 和微信赞赏码；
- 关闭按钮、Esc、背景点击均关闭并恢复焦点；打开期间 `aria-modal=true`、图片 alt 正确；
- 亮色/深色与 720x620、980x640、375px 无溢出；资源路径不是外部网页，不使用 localStorage。

## 5. 真实界面验收

1. 使用真实 Tauri/release WebView 打开指定 Demo，回读每位玩家的 `rating`、`modelVersion`、`rankingRating`、排名和 roster；确认 `advent` 保持正常参赛者身份，且排名与 Rating 严格一致。
2. 单选 Rating 最高者、`advent` 以及两至三名玩家，确认右侧始终显示完整参赛阵容排名；取消/加入玩家不会改变 benchmark、Rating 排名或其他人的排名。
3. 点击 `-`、`+`、range 和重置：雷达大小变化，中心十字、六轴标签、右侧排名和控件位置不发生布局漂移；移动视口可滚动但不裁切关键内容。
4. 进入安装与诊断，点击“赞助开发”，确认全屏模态框、微信码清晰可见；Esc、关闭按钮、背景点击均有效；再打开/关闭安装、诊断、更新、故障提交流程确认无回归。

## 6. 停止条件与最终交付

出现以下任一情况必须停止并报告“部分实现/待验收”：

- 排名仍由雷达轴总和、raw 或 UI 临时值主导，或存在 `ratingA > ratingB` 但 `rankA >= rankB`；
- 通过名字、BOT/真人身份硬编码排除或优待 `advent`，或改变既有 participant/observer 业务语义；
- 缩放仍改变 SVG 外部布局、控件位置、数据请求或裁切越界 polygon；
- 赞赏码不是官网 `wechat-reward.png` 的可追溯副本、模态框不能关闭/焦点丢失/移动端溢出；
- 只有单元测试，没有指定 Demo 的最终 JSON 和真实界面截图。

执行 AI 最终应新增执行报告，包含：实际修改文件、指定 Demo 原始/最终 hash、所有玩家 Rating/modelVersion/ranking 对账、`advent` 正常参赛者证据、BOT/真人统一规则证据、缩放前后几何与 bounding rect、浅深色/多视口截图、赞赏码源文件 hash、自动化结果及未完成的用户实机验收项。不得仅凭“界面看起来正常”或单个 Demo 名次变化宣称 Rating 排名问题已解决。
