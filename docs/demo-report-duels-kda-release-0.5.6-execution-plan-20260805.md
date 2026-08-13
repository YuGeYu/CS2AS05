# Demo 报告双向对枪矩阵、KDA Rating 与 0.5.6 发布候选执行方案

> 交接对象：下一位“实际执行 AI”  
> 工作区：`E:\CS2AS05`  
> 日期：2026-08-05  
> 状态：仅方案，尚未修改业务代码、构建、提交、推送或发布。

## 1. 目标与范围冻结

本次只处理三个用户可见目标：

1. 报告中的“对枪”页改为类似参考图片的双向击杀矩阵，并且只统计真实击杀。
2. Rating 改为接近 `(K + A / 2) / D` 的可解释公式，BOT 与真人使用同一规则。
3. 版本从 `0.5.5` 升至 `0.5.6`，完成本地发布候选准备；本方案不授权实际 GitHub、R2、D1、Updater 或用户发布。

本轮明确不做：比分算法、回合边界、观战阵营识别、BOT watcher、地图回放、热力图、道具/经济指标、parser 重写、上游升级、数据库大迁移。当前已经验证正确的 `CT 16 : 12 T / 28 completed rounds` 必须作为回归基线保留。

开始前必须记录 `git status --short` 和当前 HEAD。禁止 `git reset`、`git restore`、`git checkout`、`git clean`，不得覆盖或删除已有用户改动及 `workspace/release-evidence` 证据。

## 2. 当前事实与根因

### 2.1 对枪页当前缺少参考图所需的双向矩阵

当前入口为：

- 前端：`src/views/DemoReviewView.vue` 的 `reportTab === 'duels'`。
- 类型：`src/types/demo.ts` 的 `MatchDuelRow`。
- IPC：`src/services/tauri/demo.ts:getMatchDuels`、`src-tauri/src/commands/demo.rs:get_match_duels`。
- 查询：`src-tauri/src/services/demo.rs:match_duels`。

后端 SQL 虽然限定 `e.kind='player_death'`，但当前只返回“攻击者/目标/击杀数/爆头数”的扁平聚合行，前端又把它渲染成普通表格。因此缺少参考图片要求的“两队为横纵轴、每个交叉格展示双向击杀数”的矩阵布局。修复重点是补齐队伍分组和双向聚合，不重新解析 Demo，也不需要逐条展示 tick、回合或武器。

### 2.2 Rating 名称与实际算法不一致

`src-tauri/src/services/simple_rating.rs` 当前 `MODEL_VERSION` 为 `simple-rating-kda-v1`，但实际是 KDA、伤害、生存、助攻四项加权：`0.70/0.15/0.10/0.05`。这不是用户要求的 `(K+A/2)/D` 主导模型，必须升级模型版本并让测试、报告和 UI 一致。

## 3. 对枪页实现契约：双向击杀矩阵

### 3.1 数据语义与矩阵模型

对枪页唯一允许的统计来源是规范化事件表中的 `kind = 'player_death'`。后端先按 `killer_key -> victim_key` 聚合，再按双方队伍把玩家拆成两个轴。建议新增明确类型 `MatchDuelMatrix`，而不是继续把扁平行直接交给模板：

```ts
interface MatchDuelMatrix {
  teamX: { number: number | null; label: string; players: MatchDuelPlayer[] }
  teamY: { number: number | null; label: string; players: MatchDuelPlayer[] }
  cells: MatchDuelCell[]
}
interface MatchDuelPlayer { key: string; name: string | null; isBot: boolean }
interface MatchDuelCell {
  xPlayerKey: string
  yPlayerKey: string
  xKillsY: number
  yKillsX: number
}
```

字段来源应优先使用规范化列和 `payload_json` 中已经存在的 actor/target 以及玩家身份/队伍映射；不要从文件名或展示文本猜身份。只有 killer 和 victim 都能解析到稳定玩家、且双方属于两个可识别队伍时，才进入矩阵。世界击杀、自杀、缺 attacker、缺 victim、缺队伍的事件不进入玩家对枪矩阵，但必须计入诊断计数或质量警告，不能静默伪装成普通玩家。BOT 不得过滤，BOT 与真人一样出现在轴和格子中。

矩阵语义必须固定为：

- X 轴：队伍 X 的所有玩家；Y 轴：队伍 Y 的所有玩家。
- 每个格子代表一对玩家，不是一个方向的单值。
- 格子主值显示 `X 玩家击杀 Y 玩家` 的数量，第二个值显示 `Y 玩家击杀 X 玩家` 的数量，例如 `3 : 1`；aria-label/tooltip 必须写清楚方向。
- 队伍先按 team number/规范化队伍名，玩家再按击杀总数降序、稳定 key 升序；无交手格子显示 `0 : 0`，不能省略。
- 只统计 `player_death`，不把助攻、交易、爆头率、伤害、道具或经济混入格子。

建议保留 `get_match_duels` 命令名以减少 IPC 破坏，但把返回类型改为单个 `MatchDuelMatrix`；前端必须按两轴渲染矩阵，不能把 cells 再渲染成普通五列表格。不能同时把旧矩阵和新矩阵混在同一响应中。

### 3.2 前端视觉要求

先执行一次 UI/UX Pro Max 定向检索：

```powershell
python C:\Users\GOPtZ\.agents\skills\ui-ux-pro-max\scripts\search.py "CS2 kill feed tactical dashboard blue SaaS data dense" --design-system -p "CS2 Demo 0.5.6"
```

参考用户图片逐项核对，不凭空声称颜色和布局已一致。整体继续使用通用蓝色 SaaS、数据密集、宽屏大气风格；对枪页本身必须是二维交叉表，不是连续事件流，也不是普通五列表格：

- 顶部保留页面标题和双方总击杀数；不展示 KAST、经济、伤害、交易、胜率、爆头率等非本页内容。
- 表头显示一方玩家，首列显示另一方玩家；表头使用队伍色带和队伍标签，格子居中显示双向数值 `xKillsY : yKillsX`。
- 格子可用轻微底色/强度表达交手次数，但数字必须始终可读；颜色不能是唯一信息表达方式。
- 轴上的玩家名称过长时截断并保留 `title`；BOT 可用小图标或文字 `BOT` 标识，但不得使用另一套统计规则。
- 玩家数较多时允许矩阵区域内部横向滚动，页面整体不能横向溢出；小屏保持首列 sticky，表头可横向滚动，不能把格子压到不可读。
- 使用现有图标库（Lucide，如 Skull/Crosshair/Shield 等），不要手绘 SVG；图标按钮提供 tooltip、`aria-label` 和 focus-visible。
- 150-300ms 的矩阵进入/筛选微动画；支持 `prefers-reduced-motion`，卸载时清理定时器/监听器。
- 空、加载、错误、队伍缺失状态必须稳定；没有可识别双方时显示明确空状态，不能伪造队伍或玩家。
- 不嵌套卡片，不把矩阵包进多层浮动卡；保持表头、轴、格子层级清晰。

### 3.3 对枪页测试

至少新增/更新：

- 后端 Rust：只聚合 `player_death`；双向计数正确；同一对玩家两个方向分别统计；队伍/玩家排序稳定；无 attacker/victim/队伍策略固定；BOT 保留；非击杀事件、assist、trade、bomb 不进入 cells。
- 前端 Vitest：验证 X/Y 两轴、`3 : 1` 双向显示、`0 : 0` 空格、BOT 轴玩家、方向 aria-label；断言页面不出现“助攻、交易、经济、KAST、爆头率、武器/Tick/回合明细”等非矩阵字段；加载、空、错误和窄宽度状态。
- 当前正确 Demo 回归：`CT 16 : 12 T`、28 个 completed rounds 不变；矩阵两个方向的总和分别等于 canonical death event 的两个方向聚合，不得用单向覆盖或错误合并。
- Tauri 真实窗口至少复验 1440x900 和一个窄尺寸；若 UI 改动导致四尺寸门禁，必须按现有 preflight 流程重新取证。

## 4. Rating 新规则

### 4.1 公式与边界

新模型建议命名为 `simple-rating-kda-v2`，不要覆盖历史 `simple-rating-kda-v1` 的解释。核心原始值：

```text
kda_ratio = (kills + assists * 0.5) / effective_deaths
```

其中 `kills/assists/deaths` 必须来自可靠的 canonical scoreboard/事件统计；缺任一字段或 `rounds_played == 0` 时 `ratingStatus=unavailable`，不能用缺失值补零。为避免零死亡无限大，`effective_deaths = max(deaths, 1)`，并对最终 rating 设有限上限（建议 0.00-3.00，具体归一化常数由执行 AI依据现有 UI 量程固定并写测试）。推荐简单、可解释的实现：原始 `kda_ratio` 作为主值，做有限 clamp/round；不要继续混入伤害、生存、经济或其它隐藏权重。若保留 DTO 的 `damageComponent/survivalComponent/assistComponent` 字段，必须明确它们为兼容字段并置为 0/对应可解释值；更稳妥是扩展 DTO 为 `kdaRatio` 并同步前端类型。

BOT 与真人输入相同则输出完全相同。observer 不参与 Rating；观战阵营或身份不可靠的玩家显示 unavailable。历史报告读取必须兼容旧模型字符串，不能在读取旧 JSON 时错误地按 v2 重算。

### 4.2 Rating 测试矩阵

至少覆盖：

1. `(K=14,A=4,D=6)` 的原始值为 `14.3333/6`，并与最终显示值的归一化规则一致。
2. 固定 K、D 时 A 增加 2，原始贡献增加 1，证明助攻权重恰为 0.5。
3. K 增加使 Rating 严格上升；D 增加使 Rating 下降；同输入 BOT/真人相同。
4. D=0 有限且不为 NaN/Infinity；K=A=D=0 结果稳定（建议 0）。
5. 缺 K/D/A、rounds=0、observer 返回 unavailable。
6. 新模型字符串为 `simple-rating-kda-v2`，报告、SQLite、TypeScript 和 UI 一致。
7. 真实 de_nuke 样本回归：比分仍为 CT 16:T 12，28 回合；新 Rating 只改变 Rating 字段，不改变 kills/deaths/assists、比分或回合数。

## 5. 0.5.6 版本升级清单

只精确修改项目自身版本字段，禁止全局替换 `0.5.5`，禁止误改第三方依赖：

- `package.json` 顶层 `version`。
- `package-lock.json` 顶层项目版本（用 npm 生成/核对，不手工改依赖版本）。
- `src-tauri/Cargo.toml` 当前 package `version`。
- `src-tauri/Cargo.lock` 仅更新 `ai_pc_fac` 自身 package 版本，不能改依赖版本。
- `src-tauri/tauri.conf.json` 的 `version`。
- 任何由 `appConfig.appVersion`、插件 marker、release notes 或测试 fixture 读取的当前版本字段。

同步更新测试和文案：`tests/post-match-scoreboard.spec.ts` 中旧的 `simple-rating-v2`/`0.5.5` fixture 应逐项改为新契约；`tests/release-blocker-recovery.spec.ts`、`tests/installer-contract.spec.ts` 和版本彩蛋测试要区分“当前版本断言”与历史升级用例，不能盲目替换历史版本。运行 `scripts/release-manifest.mjs` 前必须保证它找到带 `0.5.6` 的已签名 NSIS；没有签名时只能停在候选，不能伪造 manifest。

## 6. 执行顺序与验收闸门

1. 记录工作区状态，阅读并锁定当前比分回归样本；完成数据契约和 UI/UX Pro Max 检索。
2. 先做后端 kill query/DTO/命令和 Rust 测试，再改前端 API、类型和 `DemoReviewView.vue`；每步保持编译通过。
3. 改 `simple_rating.rs`、模型版本和报告/SQLite 映射，补齐 KDA 边界测试。
4. 精确升至 0.5.6，更新测试 fixture、release notes 和版本自检。
5. 自动化门禁：

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test -- --pool=threads --maxWorkers=1
npm run build:web
cargo fmt -- --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml --lib
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
git diff --check
```

6. 真实/fixture 回归必须同时记录：比分 `CT 16:12 T`、completed rounds `28`、canonical kill 数、Rating model/version、四尺寸 viewer/报告页无横向溢出。报告字段缺失保持 `null`，不能用估算值填充。
7. 仅在自动化和真实 Tauri 回归通过后构建 0.5.6 release NSIS。核对 EXE、`.sig`、SHA-256、大小、版本、插件 ZIP marker；未签名构建明确标记为 unsigned candidate。
8. 安装/覆盖安装/卸载重装和 updater、GitHub、R2、D1、群公告、用户 BOT/CS2 实机验收均留给后续用户明确授权；本方案结束时不得 commit、push、tag、deploy 或 release。

## 7. 证据与停止条件

证据目录建议：`workspace/release-evidence/0.5.6-duels-kda-YYYYMMDD-HHmm/`，至少保存 `git-status.txt`、测试输出、版本扫描、真实 Demo JSON 摘要、截图/尺寸结果、installer hash 和未完成闸门清单。

遇到以下任一情况立即停在修复或候选阶段：canonical death 数与 UI 不一致；比分从 CT 16:12 变化；Rating 出现 NaN/Infinity 或模型字符串混用；旧报告无法读取；0.5.6 三处版本不一致；签名、manifest、安装器无法从同一最终 EXE 推导。不要通过隐藏错误、删除测试、重写历史证据或全局替换版本号来“通过”。

## 8. 交接完成标准

实际执行 AI 的回报必须分开列出：已修改文件、自动化通过项、真实 Tauri/真实 Demo 证据、未完成项和下一步需用户执行的动作。只有在用户后续明确授权并完成签名/安装/真实 BOT 验收后，才可把 0.5.6 称为正式发布；本文件本身只代表可执行方案，不代表功能已经实现。
