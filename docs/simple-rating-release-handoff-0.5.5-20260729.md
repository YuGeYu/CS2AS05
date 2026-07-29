# 0.5.5 简易 Rating 与正式发布完整交接方案

> **执行顺序更新：** 本文的简易 Rating 公式、schema 和缓存迁移仍有效；战报白屏/Windows 死锁、插件未安装、Demo 目录、签名污染和最终发布 Gate 请以 `docs/release-blocker-recovery-handoff-0.5.5-20260729.md` 为准，并先完成该方案的 P0 修复。

> 面向下一个实际执行 AI。本文是独立交接文档，不假设能读取本次对话。执行时以实时工作树、运行结果和线上回读为准。

## 0. 目标、授权与最终结论

用户已经明确决定：不再等待 parser 提供逐 death tick 的全员同步经济快照；使用当前 Demo 已能稳定得到的数据实现一个简单、透明、可复现的 Rating，并把当前全部候选改动收敛为可正式发布的 `0.5.5`。

因此，旧方案 `docs/post-match-scoreboard-window-openrating-release-plan-0.5.5-20260729.md` 的 Gate A **从本方案开始废止为发布门禁**。该 Gate 的调查结论仍然有效：`economyAdjustment`、完整 KAST、trade、round swing 等高级指标目前不可可靠计算；这些字段继续为 `null`、UI 显示 `--`，不得填伪造的 `0`。但它们不再是简易 Rating 的输入，也不再阻止发布。

本次必须交付：

1. 独立 Tauri 战报窗口正常显示当前新 Demo 的双方 10 名玩家。
2. 每名具备 `kills/deaths/assists/damage/completedRounds` 的玩家均显示“简易 Rating”，而不是 `--`。
3. 公式只使用已有可靠数据，并在代码、测试、UI 和发布说明中明确它不是 HLTV Rating，也不是完整 OpenRating。
4. 自动化、真实 Tauri、当前 Demo、签名安装包、安装/升级以及线上回读达到发布标准。
5. 保持 npm、Cargo、Tauri、资源 marker 等版本均为 `0.5.5`，不得升到 `0.5.6`，除非下文同版本替换闸门失败并由用户重新决定。

本方案授权实际执行 AI 修改源码、测试和文档，并在全部门禁通过后执行一次 Git push、更新 GitHub Release、构建签名安装包和部署 R2/D1。不得提交真实 Demo、SQLite、日志、`workspace/` 证据、密钥或 DPAPI 文件。

## 1. 2026-07-29 已调查基线

### 1.1 Git 与公开发布现状

- 工作目录：`E:\CS2AS05`
- 分支：`main`
- 当前 HEAD：`1021a755d2234a8e0f89392b6403ea0739fe70f5`
- `HEAD...origin/main` 实测为 `0 0`。
- GitHub 已存在公开 Release：`https://github.com/YuGeYu/CS2AS05/releases/tag/v0.5.5`。
- 该 Release 有 EXE、`.sig`、`.sha256` 三个资产；2026-07-29 调查时三者 `downloadCount` 均为 `0`。
- 现有 `v0.5.5` 是 annotated tag；执行前必须同时记录 tag object 和 peeled commit，不能只看 `targetCommitish=main`。

这不是“首次发布”。它只能作为**零下载条件下的受控同版本替换**。实际执行前必须重新联网回读 downloadCount、线上 updater 状态和 R2/D1 状态；任何资产下载数大于 0，或生产 updater 已经向用户提供旧的 `0.5.5`，立即停止同版本替换，保留现场并报告用户，需要改发 `0.5.6`。

### 1.2 当前新 Demo 与 parser 已有能力

权威样本：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260729-0958-de_dust2-advent.dem
Size:   32,630,115 bytes
SHA256: C74963C77651D40A63EE835E713386EFB0CF67EEE8B3618163593DE063E485D5
Map:    de_dust2
Patch:  14173
```

已有证据：`workspace/release-evidence/0.5.5-scoreboard-20260729-gate-a/gate-a-summary.txt`。该目录被 `.gitignore` 排除，只用于本机证据，不提交。

已确认可用：10 名玩家身份、SteamID/userid/controller、T/CT 队伍、`player_death`、`player_hurt`、回合事件，以及玩家最终 `kills/deaths/assists/damage/headshots`。controller totals 优先；缺失时已有 event aggregate 回退。不可用的是击杀瞬间其余队员的同步经济快照，而不是 K/D/A/damage。

### 1.3 当前候选代码状态

工作树已有未提交实现：独立 `scoreboard.html` 入口、`scoreboard` Tauri 窗口、战报 UI、Rating DTO、schema/cache 升级、灰色科技样式、Local-Arena provenance，以及 `demo_spike.rs` 调查增强。不要 reset 或回退这些改动，基于它们收敛。

关键问题位于 `src-tauri/src/services/demo.rs`：当前在构建 `rounds` 之前调用 `build_scoreboard()`，所以每个玩家的 `rounds_played` 仍为 `None`；随后 `open_rating::calculate()` 又要求 survival/KAST/multi/swing/economy 全部存在，导致所有 Rating 必然 unavailable。

## 2. 简易 Rating v1 的唯一公式

### 2.1 名称与模型版本

产品名统一为：**简易 Rating**。

模型版本统一为：

```text
simple-rating-v1
```

禁止继续把最终值称为 `OpenRating`、`HLTV Rating` 或“完整 Rating 3.0”。现有 Local-Arena 来源记录可以保留为早期战报契约/信息架构参考，但最终公式必须明确为本项目独立、简化、确定性的 v1 公式。同步修正 `NOTICE.md` 和 `docs/local-arena-openrating-port-0.5.5.md`，避免声称最终公式仍是 Local-Arena OpenRating 的完整移植。

### 2.2 输入

仅使用下列已有字段：

```text
K = kills
D = deaths
A = assists
DMG = damage
R = completed rounds in this report
```

资格条件：五个输入均存在，`R > 0`。任何输入缺失则该玩家 `ratingStatus=unavailable`、`rating=null`；不得用 0 补缺失值。

`R` 是报告级已完成回合数，不是假装精确的逐玩家出场回合。当前本地/BOT 完整对局中，把同一 `R` 注入双方有效玩家。UI 展开详情写“报告回合”；不要写“精确参赛回合”。

### 2.3 公式

```text
KPR = K / R
DPR = D / R
APR = A / R
ADR = DMG / R

killComponent     = clamp(KPR / 0.70, 0.0, 2.5)
damageComponent   = clamp(ADR / 82.0, 0.0, 2.5)
survivalComponent = clamp((1.0 - clamp(DPR, 0.0, 1.0)) / 0.68, 0.0, 2.5)
assistComponent   = clamp(APR / 0.20, 0.0, 2.5)

rating = clamp(
  0.45 * killComponent
  + 0.30 * damageComponent
  + 0.15 * survivalComponent
  + 0.10 * assistComponent,
  0.0,
  3.0
)
```

Rust 内部使用 `f64`；DTO 输出各 component 和最终 rating 均四舍五入到四位小数；UI 最终值显示两位，ADR 一位。`headshotPercent = headshots / kills * 100` 沿用现有逻辑，但不进入 Rating。

说明：survivalComponent 只是基于 `deaths/rounds` 的报告级生存代理，不等于逐回合 survival；这就是为什么 UI 只展示最终简易 Rating，不新增“生存率”列。

### 2.4 固定测试向量

必须将以下向量写成 Rust 单元测试，不要只测“有值”：

```text
average: K=14,D=6,A=4,DMG=1640,R=20
KPR=.70, DPR=.30, APR=.20, ADR=82
components = 1,1,(.70/.68),1
rating = 1.0044（允许误差 0.0001）

zero: K=0,D=20,A=0,DMG=0,R=20
rating = 0.0000

dominant: K=40,D=0,A=20,DMG=5000,R=20
components = 2.5,2.5,1.4706,2.5
rating = 2.3456

invalid: R=0 或任一输入 None
rating = None
```

测试中不要复制实现代码，分别断言 component 和最终值；另加一组极端输入只用于证明最终 `0..3` clamp 不会被绕过。

## 3. 后端实施步骤

### 3.1 收敛服务和 DTO

推荐将 `src-tauri/src/services/open_rating.rs` 重命名为 `simple_rating.rs`，并同步 `src-tauri/src/services/mod.rs` 调用。如果为减少机械改名而保留文件名，也必须将模块公开名、`MODEL_VERSION`、函数注释和 UI 名称改为 simple rating，不允许运行时继续出现 OpenRating。

把 `DemoRating` 改为与真实公式一致的契约：

```rust
pub struct DemoRating {
    pub model_version: String,
    pub kill_component: f64,
    pub damage_component: f64,
    pub survival_component: f64,
    pub assist_component: f64,
    pub rating: f64,
}
```

删除 `multi/swing/economy/kast/open_rating` 这类并未参与最终模型的 component 字段。`DemoPlayer` 中高级原始字段可以暂时保留为 `Option`，供未来扩展；本版仍保持 `null`。

前端 `src/types/demo.ts` 与 Rust camelCase 序列化必须一致。将 `ratingStatus` 联合类型改为 `complete | partial | unavailable` 仅用于报告级 data quality；玩家级只用 `complete | unavailable`，最好拆成两个类型别名，避免含义混淆。

### 3.2 先建回合，再算计分板

重排 `parse_report()`，不能继续在第一个回合事件尚未聚合时计算 Rating：

1. parser 得到 `output`。
2. header 从 `output.header.as_ref()` 克隆读取，避免消费后妨碍后续逻辑。
3. 遍历 `output.game_events` 的引用（或先 clone 所需事件）构建 `DemoRound`。
4. `completed_rounds` 只统计 `round_end`，不要把 `round_officially_ended` 再算一次，也不要使用当前样本中多出来的 `round_freeze_end` 数量。
5. 如 `round_end==0`，回退为具有 winner 且 end_tick 非空的逻辑回合数；仍为 0 则 Rating unavailable。
6. 调用 `build_scoreboard(&output, entity_status, completed_rounds)`。
7. 对 T/CT 且具有至少一个可靠总计来源的玩家写 `rounds_played=Some(completed_rounds)`，再算 ADR、HS% 和 Rating。
8. summary 的 `total_rounds` 也使用 `completed_rounds`；timeline 可保留额外的未完成/准备回合，但不能影响分母。

必须防止 `round_officially_ended` 覆盖下一回合：当前状态机收到 `round_end` 后没有清空 current，后续 official event 可更新同一回合，但不得生成新回合或双计数。

### 3.3 数据来源与质量状态

保持现有优先级：controller totals > event aggregate > unavailable。不要为了 Rating 改 vendored demoparser，也不要继续扩展全 tick entity 属性。

报告级状态：

- `complete`：全部展示的 T/CT 玩家都有 Rating。
- `partial`：至少一名有 Rating，但并非全部。
- `unavailable`：无人有 Rating。

`ratingWarnings` 使用准确且不阻塞发布的文案：

- complete：空数组，或一条非警告说明放到 UI footer，不放 warnings。
- partial：`部分玩家缺少 K/D/A、伤害或报告回合，未计算简易 Rating。`
- unavailable：`当前 Demo 缺少 K/D/A、伤害或已完成回合，无法计算简易 Rating。`

高级字段的说明独立为：`KAST、交易、Swing 与经济指标需要逐回合同步状态，本版本暂不提供。` 不得把它写成整个 Rating unavailable 的原因。

### 3.4 Schema、缓存与迁移

当前候选已经使用 schema 3 / `openrating-demo-v1`。最终改为：

```text
REPORT_SCHEMA_VERSION = 4
PARSER_ADAPTER_VERSION = "2"（解析适配未变可保持）
METRICS_VERSION = "simple-rating-v1"
SQLite PRAGMA user_version = 4
```

初始化时将 schema/metrics 不匹配的已有报告标记并异步重新解析。当前 cache hit 条件必须同时检查 report schema、parser adapter 和 metrics version。增加迁移测试，证明 schema 3 的 `openrating-demo-v1` 不会被直接复用。

不要修改数据库文件名 `demo-review-v1.sqlite3`；升级必须保留用户目录、roots、recording setting 和历史索引，仅重算 report JSON。

## 4. 前端与独立窗口实施

继续保留当前灰色科技、数据密集型桌面战报风格；UI/UX Pro Max 调查建议也是 Data-Dense Dashboard。不要重新做营销页、装饰性大卡片或浏览器原生窗口行为。

修改 `src/components/scoreboard/PostMatchScoreboard.vue`：

1. 表头 `OpenRating` 改为 `简易 Rating`。
2. 排序和 MVP 改读 `player.rating?.rating`。
3. Rating 单元格显示 `1.23 高` / `0.98 中` / `0.76 低` / `-- 不可用`，不只依赖颜色。
4. 建议阈值保持当前候选：`>=1.10 高`、`>=0.90 中`、否则低；这是展示档位，不改变公式。
5. KAST 与 Swing 列本版全部 `--`，信息密度不佳。为可发布且不误导，主表改为：玩家、K-D-A、ADR、HS%、简易 Rating，共 6 列；KAST/Swing/经济/交易放到展开详情的“暂不提供”说明，不显示一整列假空值。
6. 展开详情的“参赛回合”改为“报告回合”。
7. footer 改为 `简易 Rating · simple-rating-v1 · 非 HLTV 官方模型`。
8. 报告级 partial 不应让已有 Rating 消失；仅对应玩家显示 unavailable 原因。

保留窗口 label `scoreboard`、`1100x720`、最小 `860x600`、44px 标题栏、关闭按钮、show/unminimize/focus、`demo://show-report`、Vite 双入口、sticky 表头、键盘 Enter 展开和 `prefers-reduced-motion`。

增加前端测试，至少覆盖：公式 DTO 渲染、两位小数、档位文字、不可用玩家、MVP/排序、报告回合文案、无 OpenRating/HLTV 冒充文案、独立入口构建。可用组件测试，不要只做字符串 grep。

## 5. 不纳入 0.5.5 的工作

以下全部移至后续版本，不阻塞本次发布：

- death tick 全员同步装备快照、两队装备总值和 bomb-state economy adjustment。
- 精确逐玩家 survival、KAST、5 秒 inclusive trade、trade denial、multi-kill、first kill/death 和 round swing。
- vendored parser 的 tick snapshot adapter。
- 用持久化 `demo_sessions` 状态机替换现有 360 秒观察器。

不过现有“CS2 退出后识别新 Demo 并弹战报”仍需做真实运行验收；不能因状态机未来再做而跳过本版现有路径的验收。

## 6. 必须补齐的测试

### 6.1 Rust

- `simple_rating` 四个固定向量和所有 None/rounds=0 分支。
- round parser：6 个 `round_end` + 10 个 `round_officially_ended` 仍得到 `R=6`。
- 所有 T/CT 玩家得到同一报告回合数，spectator/unknown 不参与 complete 判定。
- controller total 和 event aggregate 两条路径都能计算；缺 damage 的单个玩家为 unavailable 且报告为 partial。
- schema 3 / old metrics cache 必须重新解析；schema 4 / current metrics 才命中。
- serde JSON snapshot/结构化断言包含 `modelVersion=simple-rating-v1` 与新 component 名，不含旧 `openRating/economy/multi/swing` component。
- 当前真实 Demo ignored test 增强为：map dust2、completed rounds 6、T/CT 共 10 人、每人 K/D/A/damage/R/rating 非空、Rating 0..3、report rating complete。

### 6.2 前端与构建

依次运行：

```powershell
npm run typecheck
npm run lint
npm test
npm run build:web
npm run workspace:check
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
git diff --check
```

`cargo clippy --all-targets -- -D warnings` 仍可能首先被 vendored demoparser 的既有 10 条 warning 阻塞。先普通运行 clippy 并保存输出；若 `-D warnings` 仅因 vendor 失败，明确记录 vendor 路径/警告，确认本项目 crate 无新增 warning，不为通过门禁而大改 vendor。若本项目代码有 warning，必须修复。

当前真实 Demo：

```powershell
$env:CS2AS_DEMO='D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260729-0958-de_dust2-advent.dem'
cargo test --manifest-path .\src-tauri\Cargo.toml real_demo_scoreboard -- --ignored --nocapture
Remove-Item Env:CS2AS_DEMO
```

将完整 stdout/stderr、exit code、耗时和结构化摘要保存在新的 `workspace/release-evidence/0.5.5-simple-rating-<timestamp>/`，不要覆盖旧 Gate A 证据。

## 7. 真实 Tauri 与 CS2 验收

实际执行 AI 启动开发版或候选版，用户负责真实游戏内效果确认；网站类回读可由内置浏览器完成。

### 7.1 当前 Demo 窗口

1. 从录像库对上述 Demo 点击弹出图标。
2. 独立窗口而非主 `App.vue` 打开；确认不出现开屏/更新协调器。
3. 10 名玩家 T/CT 分队正确，K/D/A、ADR、HS% 与报告一致。
4. 每名有效玩家显示数值 Rating，MVP 和排序随 Rating。
5. 重复点击复用同一 label，切换 report、show、unminimize、focus 正常。
6. 在 `1100x720` 和 `860x600` 检查无重叠/溢出；表头 sticky，必要时表格自身横向滚动。
7. Tab/Enter、可见 focus、44px 关闭区、reduced-motion 正常。

建议保存两张截图，并人工核对最长 Demo 文件名、中文玩家名/BOT 标签和最低窗口下 footer 不遮挡表格。

### 7.2 新一局 BOT

用助手启动一局 BOT/本地对局并至少完成 3 个回合，然后正常退出 CS2：

- 新 Demo 被现有观察器识别且稳定后解析。
- 主窗口不被强制切换页面；独立战报窗口自动出现或能从通知/录像库打开。
- 新报告不是旧 reportId，双方玩家、回合、Rating 合理。
- 若 watcher 没弹但手动扫描能解析，这仍是发布阻塞；修本版现有路径，不把问题推给未来 session coordinator。

## 8. 签名构建、安装与升级

沿用 `docs/final-release-execution-plan-0.5.5-20260728.md` 的密钥、安装、发布和回读纪律，但用本方案的功能门禁覆盖其旧 Rating 结论。私钥位于当前 Windows 用户的既有 release key 目录；绝不打印私钥或密码，不生成新 key。

构建前把同名旧 EXE/`.sig` 和 stale manifest 移到新 evidence 备份目录，避免 `release-manifest.mjs` 按文件名误取；不要删除 0.5.3/0.5.4 正式产物。然后在单个 PowerShell 进程读取 Trim 后的私钥文本和 DPAPI 密码，运行正式 bundle。

```powershell
$keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
$secure = Get-Content (Join-Path $keyDir 'updater-password.dpapi') -Raw | ConvertTo-SecureString
$credential = [PSCredential]::new('tauri-updater', $secure)
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content (Join-Path $keyDir 'updater.key') -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $credential.GetNetworkCredential().Password
try {
  npm run bundle:desktop
  $env:RELEASE_CHANNEL = 'prod'
  npm run release:manifest
} finally {
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
  Remove-Item Env:RELEASE_CHANNEL -ErrorAction SilentlyContinue
}
```

最终必须得到：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe.sig
E:\CS2AS05\dist-release\cs2-bot-improver\updater-prod.json
```

记录 EXE/`.sig` size、SHA256、mtime；结构化验证 manifest 的 version/channel/projectId/path/size/hash/signature/pub_date 全部对应这一次最终构建。Tauri `.sig` 与 Windows Authenticode 是两回事；`Get-AuthenticodeSignature` 若仍为 `NotSigned` 如实记录。

依次完成：全新安装、启动并打开当前 Demo、关闭；覆盖安装现有 0.5.4 并确认用户配置/Demo DB 保留；卸载/重装；最后用真实 0.5.4 客户端走应用内 updater 验证签名、被动安装和重启。安装器最多构建 5 次，超过即停止并报告根因。

## 9. 受控替换现有 v0.5.5

发布动作开始前再次执行并保存 JSON：

```powershell
git status --short
git rev-parse HEAD
git rev-parse v0.5.5
git rev-parse 'v0.5.5^{}'
git rev-list --left-right --count HEAD...origin/main
gh release view v0.5.5 --repo YuGeYu/CS2AS05 --json url,isDraft,publishedAt,assets
```

并从 `E:\cs2as` 回读生产 D1/R2、自定义更新 API、Tauri feed。满足以下全部条件才允许同版本替换：

- 旧 GitHub 三个资产 downloadCount 仍全为 0。
- 旧 0.5.5 未在生产 updater 启用、feed 未向 0.5.4 返回旧 0.5.5。
- 最终功能、真实 Demo、Tauri 窗口、签名安装和覆盖 0.5.4 均通过。
- staged diff 无 workspace、Demo、DB、日志、密钥、DPAPI、target 或未知二进制。

提交应包含全部经审阅的源码、测试、NOTICE/provenance 和本方案；旧 Gate A 文档可保留为历史调查，但在顶部追加醒目 superseded 链接，防止后续 AI误用。

由于远端 `v0.5.5` 已存在，推荐顺序：

1. 创建新的 release commit，但先不 push。
2. 删除旧 GitHub Release（先保存元数据和零下载证据）。
3. 删除远端旧 tag；这一步本身是一次远端写操作，因此受“GitHub push 最多 1 次”约束时，应优先使用 GitHub API 删除 tag ref，然后只进行一次最终 `git push origin main refs/tags/v0.5.5`。不要 force push main。
4. 本地删除并重新创建 annotated `v0.5.5` 指向新 release commit。
5. 一次 push main + tag，随后回读远端 commit 和 peeled tag。
6. 创建新的非 draft GitHub Release，上传最终 EXE、`.sig`、`.sha256` 和发布说明。

GitHub API 删除 tag ref 的精确形式为 `gh api -X DELETE repos/YuGeYu/CS2AS05/git/refs/tags/v0.5.5`；必须先确认旧 Release 已归档且零下载证据已保存。最终 push 命令只运行一次：`git push origin main refs/tags/v0.5.5`。创建 Release 前再次执行 `git ls-remote origin refs/heads/main refs/tags/v0.5.5` 并核对 commit。

若项目对“一次 push”的定义包含 API 删除 tag，或删除 tag/Release 任一步失败，不要循环尝试；停止并向用户报告。GitHub 上传只允许一次完整尝试，网络慢时耐心等待，不重复创建 Release。

## 10. R2/D1 与生产 updater

夸克最终分享链接仍由用户创建并提供；没有新链接时可以完成 GitHub 和本地签名候选，但不得声称生产 updater 正式发布完成。

在 `E:\cs2as` 保存发布前 D1/R2/API/feed 快照，不修改或提交该仓库现有脏文件。使用既有 `self-update:publish` 脚本先 dry-run，title/summary/items 必须提到“新增本地 Demo 独立战报窗口和简易 Rating”，不能称完整 OpenRating。

```powershell
Set-Location E:\cs2as
npm run self-update:publish -- `
  --installer 'E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe' `
  --sig 'E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe.sig' `
  --version 0.5.5 `
  --quark-url '<USER_PROVIDED_0.5.5_URL>' `
  --title 'CS2 人机增强助手 0.5.5' `
  --summary '<APPROVED_SUMMARY>' `
  --items '<ITEM_1>|<ITEM_2>|<ITEM_3>'
```

上述不带 `--remote` 是 dry-run；核对无误后仅在同一命令末尾增加一次 `--remote`。不要猜测脚本参数或改用手写 Wrangler/D1/R2 命令绕过其事务顺序。

确认 dry-run 的对象 key、size、SHA256、signature 和最终 manifest 一致后，只允许一次 `--remote`。脚本必须先上传 R2、GET 回读 exact hash/size、以 updater disabled 写 D1、确认 feed，再启用。不得用 HEAD 代替 GET，不得跳过签名/hash。

启用后验证：

- custom API：0.5.4 得到 `hasUpdate=true/latest=0.5.5`；0.5.5 得到无更新。
- Tauri feed：0.5.4 得到 200、最终 URL/signature；0.5.5 得到 204。
- feed URL GET 下载到新临时文件，size/SHA256 与最终 EXE 一致；CORS/cache 合理。
- 真实 0.5.4 客户端完成 updater 下载、签名校验、安装、重启到新 0.5.5。

线上失败立即 `updater_enabled=0`，必要时 `is_active=0` 回到 0.5.4；保留 R2 和 evidence，不覆盖同 key。官网部署最多 3 次；正常不需要 Worker deploy。

## 11. 发布说明建议

```text
CS2 人机增强助手 0.5.5

- 新增本地 Demo 录像库、自动录制与回合时间线。
- 新增独立对局战报窗口，展示双方计分板、ADR、爆头率和简易 Rating。
- 简易 Rating 基于 Demo 中的击杀、死亡、助攻、伤害和报告回合计算，仅用于本地娱乐比较，不是 HLTV 官方模型。
- 修复新 Demo 的真人/BOT 身份与分队统计，并改善自定义 Steam 目录启动、窗口关闭和灰色科技界面。

已知限制：KAST、交易、经济与 round swing 需要更完整的逐回合同步数据，本版显示为 --；Demo 仅在本机处理，不上传；未来 CS2 更新可能改变 Demo 格式；Windows 安装包具有 Tauri updater 签名，但本机若无代码签名证书则不具备 Authenticode。
```

## 12. 停止条件与最终报告

遇到以下任一项停止发布：Rating 输入被 0 静默补齐；当前 Demo 不是 10 人/6 completed rounds；任一有效玩家 Rating 空缺；窗口不弹/复用错误；UI 重叠或不可操作；自动化、本版 watcher、签名、manifest、安装/升级失败；旧 v0.5.5 下载数非零；旧 0.5.5 已被生产 updater 提供；staged 含敏感/本机产物；R2 GET/D1/feed 不一致；用户说停止。

最终报告必须分开写：

```text
实现：公式版本、字段、schema/metrics、窗口/UI、修改文件
自动化：每条命令 exit code、warning 归属、证据目录
真实 Demo：path/hash、map、completed rounds、玩家数、rating 范围/状态
真实 Tauri/CS2：当前 Demo 窗口、新 BOT 对局、用户确认项
安装：EXE/.sig/manifest path、size/hash、Authenticode、全新/覆盖/卸载/重装
Git：before/new commit、tag peeled commit、一次 push、Release URL、旧资产零下载证据
生产：夸克 URL、R2 GET、D1、两个 API/feed、真实 0.5.4 updater
结果：候选 / 签名候选 / 正式发布 / 已回退
残余限制：高级指标、watcher 状态机、Authenticode、Demo 格式兼容
```

只有自动化、当前真实 Demo、真实 Tauri/CS2、最终签名安装、受控同版本替换、R2/D1/API/feed 和真实 0.5.4 updater 全部通过，结果才写“正式发布”。若仅代码和测试通过，写“可发布候选”；若缺夸克或生产权限，写“签名候选”，不要把本地成功等同线上发布。
