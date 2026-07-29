# 0.5.5 对局结束独立计分板、OpenRating 与灰色科技 UI 发布实施方案

> **历史文档：** 战报窗口、目录、签名和发布阻断修复请以 `docs/release-blocker-recovery-handoff-0.5.5-20260729.md` 为准。本文中的同步 command 动态创建 `WebviewWindowBuilder` 设计会在 Windows/Tauri 2.10.3 死锁，禁止实施。

> **已被 `docs/simple-rating-release-handoff-0.5.5-20260729.md` superseded。当前发布采用 `simple-rating-v1`，本文件仅保留为历史 Gate A 调查记录。

> **已被替代：** 用户已决定 0.5.5 使用现有可靠数据实现简易 Rating，不再以完整经济/KAST 输入作为发布 Gate。实际执行请以 `docs/simple-rating-release-handoff-0.5.5-20260729.md` 为唯一后续方案；本文仅保留为 Gate A 调查记录。

日期：2026-07-29
角色：交接给下一位实际执行 AI
仓库：`E:\CS2AS05`
实施范围：只保证当前 CS2 生成的新 `.dem`；旧 Demo 不测试、不修复、不作为发布阻塞条件。

## 0. 执行结论和硬边界

本次不是把主窗口切换到“对局复盘”，而是修复完整链路：助手启动 CS2 后记录本局基线，CS2 正常退出后识别本局新生成且已写入稳定的 Demo，解析完成后创建或唤起一个独立的 Tauri 计分板窗口。该窗口展示可靠的本局记分板和从 Demo 完整数据计算的 Rating。

用户明确要求最终版本仍为 **0.5.5**，并确认尚未让玩家从 `0.5.4` 更新。当前 GitHub 虽已有公开 `v0.5.5`，但 API 显示三个 release 资产下载数均为 `0`；本方案将它视为未投放候选并在发布阶段受控替换。不得升为 `0.5.6`，也不得混用旧的 0.5.5 EXE、`.sig`、hash 或 manifest。

不做以下事情：

- 不兼容或回归旧 Demo，不根据旧 Demo 的失败修改当前解析器。
- 不以缺字段填 `0` 的方式伪造完整 Rating；无法完整提取当前新 Demo 所需字段即阻塞发布。
- 不在 WebView 中使用浏览器文件 API、浏览器轮询或文件路径作为 IPC 输入；文件观察、会话选择、解析和 DB 访问都在 Rust。
- 不整体合并 Local-Arena；仅移植/改写明确的统计模型和结果 UI 信息架构。
- 不在用户可见 UI 中把 Local-Arena 作为品牌或合作方展示；来源只保留在源码、NOTICE 和文档。
- 本文是方案，不在本次方案制定阶段提交、推送、tag、上传或发布。

## 1. 已调查事实（实施前复核一次）

### 1.1 当前基线和同版本发布事实

| 项目 | 事实 |
|---|---|
| 当前 HEAD / main / origin/main | `1021a755d2234a8e0f89392b6403ea0739fe70f5`，调查时 ahead/behind 为 `0/0` |
| 当前版本 | npm、Cargo、Tauri 均为 `0.5.5` |
| 当前 tag | annotated `v0.5.5` 指向该基线 |
| GitHub Release | `https://github.com/YuGeYu/CS2AS05/releases/tag/v0.5.5`，2026-07-28 发布，非 draft |
| 旧公开 EXE | 72,314,460 bytes，SHA-256 `A84716B6C27A66770DE36E8855D6D434CC3C382CD0D170218D28B951337DF095` |
| 旧公开 .sig | 436 bytes，SHA-256 `AB53F82B3FEDB5D75B4A5105BEF3BD12C136097E979B7A18E5F63E752863AE57` |
| 资产下载 | 调查时 EXE / .sig / .sha256 的 `downloadCount` 全为 0 |
| 本地安装器 | `src-tauri/target/release/bundle/nsis/` 已有多个版本，构建时不能模糊取错 |

上述旧 0.5.5 数字仅用于归档和取证，不得发布为本次新代码的签名、hash 或下载文件。此前的 `docs/final-release-execution-plan-0.5.5-20260728.md` 是一次已完成发布历史，不再代表当前待实施功能。

### 1.2 功能缺失根因

`src-tauri/src/services/panel.rs::launch_cs2_inner` 在启动 Steam 后调用：

```rust
demo::observe_assistant_launch(app.clone(), chrono::Utc::now().timestamp_millis());
```

`src-tauri/src/services/demo.rs::observe_assistant_launch` 只轮询最多 360 秒，发现 CS2 退出后执行全量 `scan()`，再从最近 25 条报告中以 `mtime >= started_at - 5_000` 猜一条并广播 `demo://report-ready`。`src/components/AppShell.vue` 收到后只加载报告并将主窗口的 `current` 改为 `demoReview`。

所以现在只有“主窗口切页”，没有第二窗口。工程中没有 `WebviewWindowBuilder`、`WebviewWindow`、`emit_to("scoreboard", ...)`；Tauri 配置和 capability 都只有 `main`。

当前观察器也不可发布：长局超过 360 秒会静默失效；会话不持久；退出后没有可靠文件稳定状态机；仅按 mtime 会选错其他 Demo；错误被忽略；多次启动没有 session id 或幂等弹窗策略。

### 1.3 当前报告和 Rating 数据缺口

当前报告为 `schemaVersion=2`、`metricsVersion=scoreboard-v2`。每名 `DemoPlayer` 只有身份、队伍、K/D/A、伤害和爆头。用户已经实测当前新 Demo 的基础记分板正常，但现有 DTO/解析流程没有逐回合存活、KAST、交易、multi-kill、击杀瞬间双方存活/装备、round swing 和经济修正。

当前缓存命中只比较 schema/adapter，遗漏 `metrics_version`，新增 Rating 后可能复用旧 `report_json`，必须修复。

## 2. Local-Arena 来源、许可和模型真实性

固定研究副本：`E:\CS2AS05\workspace\reference-local-arena-fad7e4ab`
上游：`https://github.com/numakkiyu/Local-Arena`
固定提交：`fad7e4ab7441f6bb95ebe9f6186169dc424ff008`
许可证：`AGPL-3.0`；本项目为 `AGPL-3.0-or-later`。用户已获作者许可，但仍保留许可证和作者归属。

仅移植或改写：

- `addons/counterstrikesharp/shared/MatchCore/OpenRating.cs`
- `MatchStatistics.cs`、`TradeTracker.cs`
- `open-rating-3.0-proxy-v1.json`
- `Panel/src/panels/MatchResultView.tsx`
- `Panel/src/panels/MatchPanel.css` 中结果表格的信息层级

实际执行时新增 `docs/local-arena-openrating-port-0.5.5.md`，写明 URL、固定 commit、移植文件、落地文件和修改范围；新 Rust 统计文件加简短来源注释；`NOTICE.md` 增加同一来源和许可说明并保留既有作者归属。不要整体合并仓库，不要把 Local-Arena 显示为用户可见合作品牌。

模型必须称为 **OpenRating 娱乐代理模型**，不能称 HLTV Rating 3.0 或官方 Rating。其 JSON 明示 `calibrated_small_sample_entertainment`，样本只有 2 matches / 6 maps / 60 player-maps，50-map gate 未通过。

完整模型 `open-rating-3.0-proxy-v1`：

```text
kills    = clamp((kills / rounds) / 0.70, 0, 2.5)
damage   = clamp((damage / rounds) / 82.0, 0, 2.5)
survival = clamp((survivedRounds / rounds) / 0.68, 0, 2.5)
kast     = clamp((kastRounds / rounds) / 0.72, 0, 2.5)
multi    = clamp((sum((killsInRound - 1) * count) / rounds) / 0.32, 0, 2.5)
swing    = clamp(1 + roundSwing / rounds, 0, 2.5)
economy  = clamp(economyAdjustment / rounds, -0.20, 0.20)

rating = clamp(
  -0.7309958084287365
  + 0.3347209816908208 * kills
  + 0.47510339992828865 * damage
  + 0.3785772969943543 * survival
  + 0.2239626370653013 * kast
  + 0.136410832801464 * multi
  + 0.3956972806498023 * swing
  + 0.14666847859595622 * (1 + economy),
  0, 3)
```

内部四位小数，UI 两位。助攻伤害阈值 40；交易窗口为死亡后 5 秒且包含边界；KAST 为 kill / assist / survived / traded 任一成立；`multiValue = sum(max(0, killsInRound - 1) * roundCount)`。

Local-Arena 在实时插件内按击杀瞬间读取双方 alive count、全队装备值、bomb state、受害者 armor 和最高价值武器，然后计算 swing/economy。Demo 移植不能只拿最终 K/D/A。

## 3. 实施顺序

### 阶段 A：当前新 Demo 的 parser 可行性 Gate

先不做最终 UI。只使用当前 CS2 新生成、CS2 已退出且文件稳定的样本，扩展 `src-tauri/examples/demo_spike.rs` 或 ignored Rust test；Demo、SteamID、真实路径和原始输出只放 gitignore 的 `workspace/release-evidence/`。

```powershell
$env:CS2AS_DEMO = '<当前新 Demo 绝对路径>'
cargo run --manifest-path .\src-tauri\Cargo.toml --example demo_spike -- $env:CS2AS_DEMO
Remove-Item Env:CS2AS_DEMO -ErrorAction SilentlyContinue
```

必要时临时启用 `list_props`。vendored parser 已提供 `wanted_player_props`、`wanted_other_props`、`wanted_prop_states`、`wanted_ticks` 和 event fields；属性全名必须从该新 Demo 实测，不可猜 NetVar。

逐项证明可按 tick 和稳定 player key 关联：

- round start/freeze/end、player death/hurt、round MVP、bomb events；
- attacker/victim/assister userid、SteamID/slot/team、headshot、damage 和 tick；
- BOT 同名但 userid/slot 不同不会合并；
- 每个死亡 tick 的死亡前 alive/life state、armor、最高价值武器；
- 两队 alive count、装备总值和 bomb planted；
- 逐 tick 属性是否能重建击杀前后的 round swing。

若 parser API 缺实体武器/装备，优先在 `third_party/demoparser/` 做最小 Rust 适配，更新 `UPSTREAM.md`、adapter 版本和测试；禁止 Node/Python sidecar。

**Gate A：必须从当前新 Demo 重建七个模型输入。任一输入不可靠时，不准将缺值填 0，也不准私自改成 reduced 模型；先补 parser，否则阻塞 0.5.5。旧 Demo 缺字段不参与判断。**

### 阶段 B：Rust 统计契约和缓存升级

建议新增：

```text
src-tauri/src/services/open_rating.rs
src-tauri/src/services/demo_session.rs
src-tauri/src/services/scoreboard_window.rs
src-tauri/src/commands/scoreboard.rs
```

`open_rating.rs` 实现 `WeaponClass`、装备价值、`RoundSwingContext`、round swing delta、economy kill adjustment、`TradeTracker`、`RoundFlags`、`DemoMatchStatistics` 和 deterministic rating。计算不放 Vue，输入缺失使用 `Option`/状态而非 0。

扩展 Rust `DemoPlayer` 和 `src/types/demo.ts`：

```text
roundsPlayed, roundsSurvived, kastRounds, multiKills,
firstKills, firstDeaths, tradeKills, tradeDenials,
adr, kastPercent, headshotPercent, roundSwing, economyAdjustment,
ratingStatus: complete | unavailable,
rating: { modelVersion, kills, damage, survival, kast, multi,
          swing, economy, openRating } | null
```

`DemoDataQuality` 增加 `ratingStatus` 和按字段的 `ratingWarnings`。只有所有模型输入完整才返回 rating；否则 UI 显示 `--` 和具体原因，基础 K/D/A 仍可用。

事件聚合规则：

1. 真人 key 优先 SteamID64，BOT 使用 userid/slot，绝不按昵称合并。
2. 有效 round 开始时初始化参赛者 flags/alive；预热、观战、重复事件不算比赛回合。
3. hurt 只累计敌方实际 health damage；death 先读死亡前快照，计算 before/after win probability，再更新击杀、首杀、交易、经济修正和 alive。
4. round end 结算 roundsPlayed/survival/KAST/multi；保留指标到原始回合/事件的追踪关系。
5. 最终固定模型计算并在队内按 Rating 降序，保持同分稳定排序。

将 `REPORT_SCHEMA_VERSION` 升 3，`PARSER_ADAPTER_VERSION` 升真实新版本，`METRICS_VERSION` 升 `openrating-demo-v1`；DB `user_version` 升 3。缓存命中必须同时满足：

```text
status=done AND size/mtime unchanged
AND report_schema_version >= current
AND parser_adapter_version == current
AND metrics_version == current
```

旧缓存仅标记待重解析；历史文件不可访问或旧格式失败时不能后台无限刷错，本次不要求旧 Demo 解析成功。

### 阶段 C：可恢复的 CS2 -> 新 Demo 会话状态机

新增 SQLite `demo_sessions`：

```text
id, state, cs2_root, started_at_ms, seen_running_at_ms, stopped_at_ms,
baseline_json, selected_path, selected_fingerprint, report_id,
error_code, error_detail, popup_delivered_at_ms, updated_at_ms
```

状态：`armed -> waiting_for_cs2 -> running -> waiting_for_demo -> stabilizing -> parsing -> ready -> shown`；异常为 `no_cs2_seen`、`no_demo_found`、`demo_unstable_timeout`、`parse_failed`、`popup_failed`、`cancelled`。每次转换写 DB 和 runtime log，应用启动恢复非终态会话。

在 `launch_cs2_inner` 启动 Steam **之前** arm：

1. 对已启用 root 记录每个 `.dem` 的 canonical path、size、mtime、轻量 fingerprint（size + mtime + SHA-256 首尾各 64 KiB）。
2. Steam spawn 成功后协调器等待真实 `false -> true -> false` CS2 生命周期；移除 360 秒静默超时。以 12 小时持久诊断上限和重启恢复防止永久线程。
3. 退出后 notify 触发、定时扫描兜底。候选必须在 session root 内，相对 baseline 新增或 fingerprint 改变，mtime 在 session 窗口内。
4. 多候选按 mtime/变更时间排序；只有唯一最新者才自动选择，无法唯一判断时主窗口提示用户选择，不弹错误报告。
5. 每秒取 metadata，连续 3 次 size/mtime 不变才 parse；90 秒仍不稳定则保留等待状态并退避/notify 重试，不能把半写文件导入 error。
6. session ready/shown 幂等；第二局建立新 session；旧 session 不得覆盖新弹窗。

移除或改造原 `observe_assistant_launch`，不能保留两套并发逻辑。一般录像库 `scan()` 保留，但 session 不再从最近 25 条中猜报告。

### 阶段 D：独立 Tauri 窗口

新增独立 Vite entry，避免第二窗口加载 `App.vue` 后重复开屏、更新检查和主导航：

```text
scoreboard.html
src/scoreboard.ts
src/ScoreboardApp.vue
src/components/scoreboard/PostMatchScoreboard.vue
src/styles/scoreboard.css
src/services/tauri/scoreboard.ts
```

`vite.config.ts` 配置 `index.html` 与 `scoreboard.html` multi-page input。`scoreboard.ts` 只 mount ScoreboardApp。第二窗口只用 `reportId` invoke DB 报告，不接受任意路径。

Rust `open_scoreboard(report_id)`：

- 验证 report 存在、done、schema/metrics 当前。
- 新建固定 label `scoreboard`：`WebviewWindowBuilder` + `WebviewUrl::App("scoreboard.html?reportId=<id>")`，约 1100x720，最小 860x600，可调整大小。
- 已存在则 `emit_to("scoreboard", "demo://show-report", report_id)`，再 show/unminimize/focus；Scoreboard 监听事件重新 invoke。
- 用 ready handshake 避免首次创建时 event 丢失；query 仅用于首次 report id。
- 关闭 Scoreboard 只关闭/隐藏自身，不退出主程序；主程序退出一并销毁。
- create/emit/focus 失败写 `popup_failed`，主窗口 toast“报告已生成，可从录像库打开”，不能标记 shown。
- 录像库 done 条目增加 Lucide `MonitorUp`/同义图标的弹出按钮，复用同一 command；主窗口保留时间线但不再自动抢占切页。

`src-tauri/capabilities/default.json` windows 加 `scoreboard`，按 Tauri schema最小授权 window/event/core 权限。开发和打包都要验证 `scoreboard.html` 可加载。

### 阶段 E：灰色科技 UI

遵循 UI/UXPro Max：高对比、明确层级、44px 交互区、Lucide、键盘/ARIA、150--300ms 微交互、`prefers-reduced-motion`、固定表格列。字体离线 fallback 使用 `"Microsoft YaHei UI", "Segoe UI"`，数字使用 `"Cascadia Mono", Consolas`。

把 `src/styles/main.css` 暗色从青绿底改为中性枪灰/炭灰；全应用仍用语义 token，不能逐页大面积硬编码。建议：

```css
--app-bg: #111214; --app-bg-subtle: #181a1d;
--surface: #1d1f22; --surface-muted: #272a2e; --surface-raised: #30343a;
--text: #f1f3f5; --text-muted: #b2b8c0;
--border: #444950; --border-strong: #636a73;
--primary: #82d7d0; --primary-hover: #a6ebe4;
--tech-accent: #d7b46a; --danger: #ef817b;
```

青色仅焦点、数据和 CT；金色仅 MVP/高 Rating；红色仅错误/低 Rating；T 用低饱和铜色且带文字标识。不要蓝紫大渐变、发光球、满屏渐变或营销 Hero。古风仅用克制的铭文分隔线、金属刻度、地图坐标表达，不牺牲数据密度。

同时清理主工作区现有的装饰性品牌文案，范围必须精确：

- 在 `src/components/AppShell.vue` 的 `.sidebar-brand` 中删除 `<strong>人机增强</strong>` 与 `<small>玄铁机括</small>` 及其失去用途的包裹 `<div>`；保留左侧 `PackageCheck` 图标和现有 `0.5.5` 版本按钮。
- 在 `src/styles/main.css` 删除 `.titlebar-name::after` 生成的 `content: '青玉灵脉'`，并删除只服务于上述两行文案的 `.sidebar-brand > div/strong/small` 样式及对应响应式隐藏规则。
- 这里的“人机增强”仅指侧栏中位于“玄铁机括”上方的短文案；不要误删窗口标题、关于页或安装页中的正式产品名“CS2 人机增强助手”。
- 将 `.version-easter-egg` 调整为侧栏品牌区的主要文字信号：桌面建议 `20--22px`、`font-weight: 800`、tabular numbers，最小点击区仍不低于 44x44；窄窗口建议不低于 `18px`。版本号必须完整显示，不能折行、截断或挤压导航。
- 版本按钮现有功能保持原样：`title`、`aria-label`、点击计数、“别点我”提示和版本彩蛋触发逻辑全部保留；只改 DOM 排布和视觉尺寸，不改 `registerVersionClick`、触发次数、时序或键盘语义。
- 删除文案后重新收紧 `.sidebar-brand` 的 gap/padding/min-width，保持图标与放大的 `0.5.5` 水平对齐；hover/focus/active 不改变元素尺寸，避免侧栏抖动。

Scoreboard 信息结构：

1. 44px titlebar：本局战报、地图、关闭。
2. 顶部全宽战报带：地图、文件、完成时间、回合、比分；比分未验证就显示“--/未验证”，不造值。
3. 一行四项荣誉：MVP、最高 ADR、最高 HS%、最高 Kills。
4. CT/T 连续分组表格：`玩家 | K-D-A | ADR | KAST | HS% | Swing | OpenRating`。玩家列省略长名，数字列固定且不换行，表头 sticky，Rating 阈值 >=1.10 高、0.90--1.09 中、<0.90 低，并有文字/数值而非只靠颜色。
5. 点击行展开单层明细：身份、参赛回合、K/D/A、首杀/首死、交易、multi-kill、七项 breakdown；不能卡片套卡片。无 rating 时解释缺失字段。
6. 底部紧凑状态带：解析/模型状态、Demo 文件名；“在主窗口打开报告”“打开所在目录”用 Lucide 图标按钮 + tooltip/aria-label，目录操作走 Rust command。

最小窗口下允许表格容器横向滚动，不能挤坏数字列。动画只用 opacity/transform 约 200ms，reduced-motion 禁用；核心操作不可只在 hover 出现。

## 4. 测试和验收

### 4.1 Rust

- OpenRating 固定向量与 Local-Arena 四位结果一致；clamp/rounding、武器分类、经济修正、swing 正负。
- 交易 5 秒 inclusive、助攻 40 damage、KAST、multi-kill 边界。
- BOT 同名 userid 不同仍两人；真人 SteamID 优先；自杀/队伤/观战/预热不污染。
- 任何模型输入缺失时 `ratingStatus=unavailable` 且 `rating=null`，不出现假的 `0.00`。
- v2 -> v3 migration、metrics cache invalidation、不可访问历史文件不循环。
- baseline 不选旧文件、三次稳定才 parse、多候选不误选、重复 ready 只弹一次、重启恢复、长于 360 秒仍有效。
- window 新建/复用/失败状态；将 builder 决策抽成纯函数或 trait，测试不必真启动窗口。

### 4.2 前端

- `scoreboard.html` 独立入口，不 import `App.vue` 或 startup updater coordinator。
- DTO、分队/Rating 排序、两位格式、`--`、展开、长名、BOT、model disclaimer、aria-label。
- `demo://show-report` 更新同一窗口；弹出命令只传 report id。
- 灰色暗色 token、visible focus、reduced-motion、固定列、无布局抖动。
- DOM 与最终渲染中不再出现侧栏“人机增强”“玄铁机括”或 titlebar 伪元素“青玉灵脉”；正式产品名“CS2 人机增强助手”仍存在。
- 放大的 `0.5.5` 版本按钮在桌面/窄窗口都完整可见，点击与键盘激活仍触发原版本彩蛋测试，点击热区不小于 44x44。
- 回归现有冷启动一次更新检查、默认 Demo root depth 5、道具“较少”、BOT 物品默认全开，不能被新 entry 破坏。

执行：

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
git diff --check
```

未运行的命令必须记录原因，不能写通过。

### 4.3 当前新 Demo + 真实 Tauri（发布阻塞）

本次会改 parser 聚合/会话/窗口，用户此前“新 Demo 基础记分板正常”的结论不能覆盖新 Rating。最终候选上让用户：安装，启用自动录制，由助手启动 BOT，至少 5 回合，退出 CS2。

必须证明：

1. 不进入录像库也会自动弹独立“本局战报”；主窗口不被强制切页。
2. 新一局退出后复用 Scoreboard 并切到新 report，不显示旧 session。
3. 当前新 Demo 两队/身份/K-D-A/ADR/KAST/HS/Swing/Rating 完整，无空白、错误合并、未验证身份警告或假 0。
4. 860x600、1100x720、1280x800 检查亮暗、长名、Tab/Enter/Escape、关闭重开、reduced-motion；必须是真实 Tauri 窗口。
5. 解析/弹窗失败有 toast 和录像库恢复入口，应用不崩。

记录样本 size/mtime/hash、解析耗时、schema/adapter/metrics、玩家/队伍/Rating 覆盖；真实路径和 SteamID 只放 ignored evidence。旧 Demo 不运行、不阻塞。

## 5. 文件审阅表

| 类别 | 预期文件 |
|---|---|
| Rust | `models/demo.rs`、`services/demo.rs`、新增 open_rating/demo_session/scoreboard_window、commands/lib 注册 |
| Tauri | `tauri.conf.json`、`capabilities/default.json` |
| Vue/Vite | `vite.config.ts`、scoreboard entry/app/component/CSS、types/store/service、DemoReview/AppShell；AppShell 删除侧栏两行装饰文案但保留版本彩蛋 |
| 主题 | `src/styles/main.css` |
| 测试 | Rust tests、Vitest window/DTO/design contracts |
| 合规 | `NOTICE.md`、provenance 文档、本方案 |
| 版本 | 三处仍为 0.5.5；Cargo.lock 由 Cargo 更新，禁止批量替换第三方版本 |

不要提交 `workspace/`、Demo、DB、日志、截图、target/dist、密钥、DPAPI、release evidence。实施前后读 `git status --short`，不 reset 用户改动。

## 6. 构建、签名和安装

签名材料：

```text
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key.pub
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater-password.dpapi
```

私钥/密码/DPAPI 绝不打印、提交或上传。bundle 前比较 Tauri 内嵌 pubkey 与 `.pub`。将旧本地 0.5.5 产物归档到 ignored evidence 并记录 hash；不要删 0.5.3/0.5.4，确保 bundle 目录没有同名 stale 文件。

```powershell
$keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
$secure = Get-Content (Join-Path $keyDir 'updater-password.dpapi') -Raw | ConvertTo-SecureString
$credential = [PSCredential]::new('tauri-updater', $secure)
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content (Join-Path $keyDir 'updater.key') -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $credential.GetNetworkCredential().Password
try {
  npm run bundle:desktop
  if ($LASTEXITCODE -ne 0) { throw "bundle failed: $LASTEXITCODE" }
} finally {
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
  $credential = $null; $secure = $null
}
```

最终唯一 0.5.5 EXE/.sig/.sha256 要重新计算 size/hash/mtime。`.sig` 是 Tauri minisign updater 签名，不是 Authenticode；`Get-AuthenticodeSignature` 若 NotSigned 如实记录。

以 `RELEASE_CHANNEL=prod npm run release:manifest` 生成 fresh manifest，结构化断言 version/channel/projectId、installer path、size、sha256、signature、pub_date 都匹配最终产物。运行全新安装、覆盖 0.5.4、卸载重装；覆盖必须保留用户目录、Demo DB 和显式配置。

## 7. 必须保持 0.5.5 的受控重发

仅因用户明确要求且确认玩家仍在 0.5.4 执行。它是历史重写：实际功能、当前新 Demo、签名和 0.5.4 升级验证全部通过后才能开始。若旧 0.5.5 任一资产已有下载或线上 updater 已启用，立即停止，不能替换同版本。

发布前：

- 再用 `gh release view`/API 回读 downloadCount，必须全 0。
- 检查 R2/D1/custom API/Tauri feed：current 0.5.4 仍不能已被投放旧 0.5.5。
- 归档 Release JSON、旧 tag object/commit、附件 URL/size/hash 和旧本地产物到 ignored evidence。
- 新 release commit 包含代码、测试、NOTICE、provenance、本方案；staged diff/secret/version 检查通过。

唯一操作顺序：

1. 创建新 release commit：`git commit -m "release: CS2 Bot Improver 0.5.5"`。
2. 删除已归档的旧 GitHub Release：`gh release delete v0.5.5 --repo YuGeYu/CS2AS05 --yes`。
3. `git tag -d v0.5.5` 删除本地旧 tag；用 `gh api --method DELETE repos/YuGeYu/CS2AS05/git/refs/tags/v0.5.5` 删除已归档的远端旧 tag ref，不消耗正式 Git push 次数。
4. 对新 commit 创建 `git tag -a v0.5.5 -m "CS2 Bot Improver 0.5.5"`。
5. 只执行一次正式 push：`git push origin main refs/tags/v0.5.5`；失败先判断网络/权限，不制造重复提交。
6. 回读远端 tag/commit，重建 non-draft Release，只上传最终 EXE/.sig/.sha256。

最终报告明确记录 GitHub API 删除旧 tag ref 一次、Git push 一次。不要 force-push main。

Release notes 包含独立战报窗口、OpenRating 娱乐代理、灰色科技 UI、仅当前新 Demo、本地解析、不承诺旧 Demo、未来格式风险、Tauri 签名非 Authenticode。

夸克链接由用户为最终 EXE 创建，不能复用旧链接。在 `E:\cs2as` 保存 R2/D1/API/feed 快照，先 dry-run，再仅一次 `self-update:publish --remote`。启用后回读：

- custom API：current 0.5.4 -> update 0.5.5；current 0.5.5 -> 无更新。
- Tauri feed：current 0.5.4 -> 最终 URL/signature；current 0.5.5 -> 204。
- 对 feed URL 真 GET，size/hash 等于最终 EXE，不只 HEAD。
- 真实 0.5.4 完成下载、签名验证、passive 安装、重启到 0.5.5，并再次完成第 4.3 节新 Demo 验收。

## 8. 停止、回退、最终报告

停止条件：Gate A 无法完整取值；新 Demo 身份/分队/Rating 错；窗口不弹或弹错 session；灰色主题对比/布局不合格；签名/manifest/R2/0.5.4 updater 不一致；旧 v0.5.5 出现任何下载或已线上启用。

线上失败先将 D1 `updater_enabled=0`，必要时 `is_active=0` 退回 0.5.4；保留 R2/evidence，不再覆盖同版本。GitHub 新 Release 已公开但 updater 失败时在顶部标注暂停原因。用户说停止时立即停止测试、git、上传和生产写入。

最终报告：

```text
结果：候选 / 已签名 / 已发布 / 已回退
基线：before HEAD、release commit、新 v0.5.5 tag、push 次数、旧资产 downloadCount=0
功能：session id、新 Demo fingerprint、独立窗口创建/复用、恢复入口
新 Demo：hash、耗时、玩家/队伍/身份、完整 Rating 字段和 model status
自动化：每条命令与 exit code；未运行项及原因
UI：灰色 token、尺寸、亮暗截图、键盘/focus/reduced-motion
产物：EXE/.sig/.sha256 path/size/hash、签名/Authenticode、fresh manifest
安装：全新、覆盖、卸载重装、真实 0.5.4 自更新
GitHub：旧 Release 归档、新 URL/附件 hash；R2/D1/API/feed 回读
限制：OpenRating 娱乐代理、旧 Demo 不支持、未来 CS2 Demo 格式风险
证据：workspace/release-evidence/0.5.5-scoreboard-<timestamp>/
```

只有功能、当前新 Demo、真实窗口、签名安装、同版本重发前置检查和线上 updater 全部通过，结果才能写“可发布/正式发布”。
