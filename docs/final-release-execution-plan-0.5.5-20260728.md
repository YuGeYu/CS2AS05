# CS2 人机增强助手 0.5.5 最终发布执行交接方案

> 文档日期：2026-07-28（Asia/Shanghai）
> 交接对象：下一位【实际执行 AI】
> 主仓库：`E:\CS2AS05`
> 官网/更新服务仓库：`E:\cs2as`
> 本文可独立执行，不依赖此前聊天。方案制定阶段未提交、未推送、未上传、未写生产 D1/R2，也未正式发布。

## 1. 目标、权限与完成定义

目标是把当前工作树中的全部 `0.5.5` 功能收敛为可正式发布版本，并把整套桌面 UI 统一升级为大气、帅气、古风且充满科技感的产品视觉；随后生成由现有 Tauri updater 私钥签名的 NSIS 安装器和全新 manifest，完成自动化、真实 Demo、安装/升级/卸载、用户真实 BOT 新 Demo、GitHub、R2/D1 和线上接口验收。

除非用户明确停止，实际执行 AI 应从修复阻塞一直推进到发布后回读；但以下任一关键闸门失败时必须停止发布，不得把候选版称为正式版。不要新建分支，不要使用 `git reset`、`checkout`、`restore`、`clean`，不要回退当前已有改动。不要结束归属不明的 Steam、CS2、助手或开发进程。所有版本继续保持 `0.5.5`，不得临时改成 `0.5.6`。

正式完成必须同时满足：

1. 产品自动化退出码全部为 0，Clippy 主程序错误清零；当前 CS2 新生成 Demo 的记分板回归测试通过。
2. 最终 EXE 由一次干净的签名 bundle 生成，同目录存在与该 EXE 配对的 `.sig`；fresh manifest 的 size/hash/signature 与它完全一致。
3. 全新安装、覆盖安装、`0.5.4 -> 0.5.5` 自更新、正常退出和卸载均通过。
4. 用户已经完成真实 CS2 BOT 实测：打到第 5 回合后新 `.dem` 正常生成，对局复盘记分板正常。后续不修改 Demo parser/记分板逻辑时不要求用户重复验收。
5. `main` 的发布提交和 `v0.5.5` 标签已用一次 push 推送，GitHub Release 已发布；R2 回读、D1、custom API 和 Tauri feed 的版本/hash/size/signature 一致。
6. 主界面、弹窗、状态反馈和数据表格已经统一到同一套古风科技设计系统，并在 `720x620 / 960x700 / 1280x800`、亮暗主题、键盘和 reduced-motion 下通过验收；视觉升级不能破坏任何主要任务。

## 2. 制定方案时的权威现场

### 2.1 Git 与工作树

- `E:\CS2AS05`：`main...origin/main`，HEAD `6f770257a36034b27e276cd8576b2c0fa97c26f1`，tag `v0.5.4`，`HEAD...origin/main = 0 0`。
- 当前所有 `0.5.5` 功能仍是大量 modified/untracked 文件；HEAD 仍是公开的 `0.5.4`。提交前必须完整审阅 `git status --short` 和 untracked 内容，不能只提交已跟踪文件。
- `workspace/` 是本地运行/验收数据，默认不得加入 Git。两个 `startup-demo-scoreboard-evidence-*` 目录目前只有 `demo-review-v1.sqlite3`，不是完整发布证据。
- `E:\cs2as` 当前在 `codex/all-command-library...origin/main`，HEAD `5eb0263`，存在用户改动：`src/worker.ts`、`tests/supporters-cors-worker.spec.ts`、`.active-*-online-evidence` 和 `release-evidence/`。不得清理、回退或顺手提交这些文件；本次发布元数据通过既有脚本写 R2/D1，不要求推送官网仓库。

实际执行开始时重新记录：

```powershell
Set-Location E:\CS2AS05
git status --short --branch
git rev-parse HEAD
git rev-list --left-right --count HEAD...origin/main
git tag --list 'v0.5.5'
```

如果开始执行后发现无法归属的新改动，先保留现场并调查；不要用 Git 清理命令解决。

### 2.2 已实现的 0.5.5 行为

当前源码已包含以下关键契约，修阻塞时不得回退：

- `package.json`、`package-lock.json` 顶层、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 主 package、`src-tauri/tauri.conf.json` 均为 `0.5.5`；Rust MSRV 是 `1.77.2`。
- 冷启动更新检查由 `src/App.vue` 启动 `src/features/software-updates/coordinator.ts`，自动请求必须恰好一次且 `manual=false`；`SupportActions.vue` 只共享状态和提供手动检查。
- 首次人机预设 Nades 为 `less`；BOT 物品 `profiles/agents/music/weapons/knives/gloves/stickers/charms` 八项默认全开。明确存在的旧用户配置，包括 `normal`、八项全关和空刀具，必须保留。
- 玩家选择并通过校验的 CS2 根目录会注册为对局复盘默认扫描根，`origin=selected_cs2_root`、`scan_depth=5`；更换选择时旧自动根禁用，手动根不被覆盖。
- Demo 报告为 `schemaVersion=2`、`parserAdapterVersion=2`、`metricsVersion=scoreboard-v2`，带 `scoreboardStatus`；支持 `force_reparse`，旧缓存不能继续短路。
- Demo parser 已启用实体/controller 记分板，保留 raw userid/userinfo，BOT 按 userid/controller 区分，真人只接受有效 SteamID64。旧 Demo 不在本次发布兼容范围内，不继续投入适配或验收。
- 已有本地 Demo 录像库、SQLite、扫描/导入、助手 BOT 自动录制、退出后识别新文件、对局报告和回合时间线。
- 自定义 Steam 安装位置启动链已扩展；开屏鸣谢与版本彩蛋使用 Three.js，官网数据由 Rust 原生 HTTPS 命令读取；窗口/游戏关闭和 Three.js 连续帧问题已有修复。
- 当前主工作区已经有 184px 侧栏、44px 标题栏、38px 状态条、24px 网格背景和统一 motion token，但视觉仍主要是普通浅蓝/灰白与深灰；古风科技表达集中在开屏和彩蛋，尚未贯穿概览、预设、BOT 物品、刀具、命令、对局复盘、安装诊断、弹窗和 toast。
- 定制 ZIP `src-tauri/resources/CS2BotImprover.zip`：size `67,811,702`，SHA256 `ACC5E0B73626A86F3C07ECDAE04B164F806F7D5A30DDC692C3C8C864FF73F4AB`。`docs/CS2BotImprover-marker-diff-0.5.5.json` 证明 payload 未变，仅 `CS2AS05.plugin.json` marker 升到 `0.5.5`。
- `docs/panel-v1.4.3-bot-items-contract.json` 已存在，记录 Panel SHA256、八项 key、默认值和 unknown field 保留契约。

### 2.3 当前自动化结果与明确阻塞

已经实跑：

- `npm run verify`：通过；workspace/typecheck/Oxlint/ESLint/Web build 均通过，Vitest `28 files / 103 tests` 全通过。
- Web build 只有既有 Three.js chunk 约 724 KB 超过 500 KB 的警告；不是当前功能失败，但必须写入残余风险。
- `cargo fmt --check`：通过。
- `cargo test`：`39 passed / 0 failed / 2 ignored`。两个 ignored 分别依赖 `CS2AS_DEMO` 和真实 CS2 目录。
- Nade policy：`51 assertions passed`。
- 最新实测 Demo `auto-20260728-1717-de_inferno-advent.dem` 已通过 ignored 集成测试：`scoreboardStatus=complete`、`entityParseStatus=strict`、10 名玩家、T/CT、SteamID64、K/D/A、伤害和爆头均有数据；报告为 11 回合、79 kills。用户也已确认真实 BOT 打到第 5 回合后新 Demo 正常生成，应用内记分板正常。

当前 **不能发布**，因为：

1. `cargo clippy --all-targets -- -D warnings` 退出码 1。`src-tauri/src/services/demo.rs` 有 5 个错误：三处 `.is_none_or(...)` 不兼容 MSRV 1.77.2；page size 链触发 `obfuscated_if_else`；`source.user_id.is_some()` 后 `unwrap()` 触发 `unnecessary_unwrap`。按 Clippy 提示等价改写并补/保留测试，不得提高 MSRV 逃避。
2. vendored `third_party/demoparser/parser` 当前有 10 条 warning。由于这是固定上游加小范围适配，可保留合理的 dependency lint 边界，但最终上述产品 Clippy 命令必须退出 0，报告中不得写成“零 warning”。不要大范围格式化或重构 vendor。
3. 整体 UI 尚未完成本方案要求的统一升级和桌面实机验收；当前开屏和彩蛋不能代替主工作区改造。
4. 当前 `0.5.5` NSIS 是未签名候选，没有同名 `.sig`；`dist-release/.../updater-prod.json` 指向另一份旧构建，严禁发布。

## 3. 阶段 A：收敛代码、整体 UI 与自动化

### 3.1 先修发布阻塞

先修 `src-tauri/src/services/demo.rs` 的 5 个 Clippy 错误，只做语义等价修改。建议：用 `map_or(true, ...)`/显式 `match` 替代 MSRV 不支持的 `Option::is_none_or`，用清晰 `if contains { page_size } else { 25 }`，用 `if let Some(user_id)` 消除 checked unwrap。不要在此步骤扩张 Demo parser 行为。

### 3.2 整体 UI 设计方向

必须使用 UI/UX Pro Max 的产品级设计流程。它给本项目的基础建议是实时运营工具、深色高对比、科技字体气质、最小发光与清晰 focus；实际落地不能照搬纯黑+绿色模板，而要结合 CS2 桌面工具和现有 Three.js 场景，形成 **“玄铁机括 · 青玉灵脉 · 赤金铭文”** 设计系统。

这是工作型桌面应用，不做落地页或巨型 Hero，不牺牲信息密度。所谓“大气”来自清晰的空间骨架、克制的材质、统一的层级和可靠的动效，不是把标题放大、堆满发光卡片或让装饰压过功能。

UI 改造的非回归边界：不得改动冷启动更新协调器、CS2/Steam 启动和关闭、Panel 写盘事务、首次 `less`、BOT 八项全开、用户旧配置保留、默认 Demo root 深度 5、自动录制/退出后报告、记分板数据聚合、Tauri updater、安装/卸载白名单。组件拆分可以进行，但 IPC 参数、store 状态机、按钮可用条件、错误恢复和数据表格字段必须保持。任何视觉方案与这些主要任务冲突时，以主要任务为准。

视觉关键词：玄铁、冷玉、赤金、朱砂、剑匣、机关阵盘、数据铭文。禁止：紫蓝渐变统治全屏、米黄/棕橙单色古风、渐变光球、bokeh、装饰性漂浮卡片、卡片套卡片、每个元素都发光、手绘 SVG 图标、Emoji、负 letter-spacing、浏览器原生目录/文件能力。

### 3.3 双主题色彩与 token

先重构 `src/styles/main.css` 顶部 token，不在各组件散落新十六进制颜色。建议基线如下，实际执行 AI 必须用对比度工具校准到 WCAG AA：

```text
Dark / 玄铁夜色
app-bg          #090E10
surface         #11191B
surface-muted   #172225
surface-raised  #1C292C
text            #EDF2F0
text-muted      #AAB7B3
border          #30413F
primary/jade    #45C7B3
primary-hover   #68D8C4
bronze          #D3A95B
cinnabar        #D85E59

Light / 冷宣昼色
app-bg          #EEF3F1
surface         #FFFFFF
surface-muted   #E7EEEB
surface-raised  #F7FAF9
text            #172320
text-muted      #586963
border          #C8D6D1
primary/jade    #147E70
primary-hover   #0E695E
bronze          #88641F
cinnabar        #B63D3D
```

保留 `success/warning/danger/focus/titlebar/code/motion/z-index` 语义 token，新增 `--jade-* / --bronze-* / --cinnabar-* / --ink-* / --engraving-line / --panel-shadow`，但组件只引用语义层。主色青玉负责选择和主操作，赤金只用于品牌铭文、重要计数与少量强调，朱砂只用于危险/告警/印章式状态；成功、警告、错误必须同时有 Lucide 图标或文字，不能只靠颜色。

字体必须离线可靠：正文使用 `"Microsoft YaHei UI", "Segoe UI", sans-serif`；数字/SteamID/Tick 使用 `Consolas, "Cascadia Mono", monospace` 并启用 tabular nums；`STKaiti/KaiTi` 最多用于品牌短铭文或开屏标题，缺失时自然回退，不能用于表格和正文。所有 letter spacing 为 0。

### 3.4 布局与全局外壳

优先改 token、全局 primitive 和少量语义 class，保留现有 Vue 状态/IPC/业务结构，避免为换皮重写页面：

- `src/App.vue` 与自定义标题栏：保持 44px 稳定高度和 Windows 拖动/最小化/最大化/关闭行为。标题栏改为玄铁横梁风格，品牌图标、应用名和窗口按钮边界清晰；关闭按钮仍只在 hover/focus 呈危险色。
- `src/components/AppShell.vue`：侧栏桌面宽度保持约 184px，不做可折叠炫技。品牌区用现有应用图标、版本按钮和一条赤金铭文线建立第一视觉信号；导航继续用 Lucide，选中态采用青玉左侧能量轨+轻表面变化，不能只靠荧光。
- `StatusStrip.vue`：作为“机括状态横梁”，固定高度、不随文案跳动；CS2、Panel、路径、更新/扫描状态有图标+文字，长路径省略并保留 title。
- `.tool-view/.view-heading`：保持紧凑工具页，不做 Hero。页面 H1 约 22-26px，附短 overline 和一条赤金/青玉分段铭线；内容最大宽度一致，避免每页漂移。
- 概览页只突出一个主操作“启动 CS2”。目录、模式、难度、录制按任务流排列；启动带可做未嵌套的全宽“剑匣控制台”，不再用普通白卡堆叠。
- 预设/BOT 物品/刀具/命令页继续高密度、可扫描。二元设置用 Toggle，模式用 SegmentedControl，颜色用 swatch（若出现），工具按钮用 Lucide 图标和 tooltip。
- 对局复盘是数据主角：摘要使用一条无嵌套的指标带，T/CT 分组在同一表格内用青玉/赤金细线和文本区分；SteamID、K/D/A、伤害使用等宽数字，表头 sticky，横向滚动只发生在表格容器。
- 安装与诊断页取消“大白色安装卡”观感，改成连续的工作台分区；状态、主安装动作、兼容入口和诊断层级明确。弹窗、toast、目录建议框与更新弹窗统一玄铁/冷宣表面与 1px 铭线。

页面 section 默认无外层浮卡。只有独立重复项、模态框、刀具项和真正需要框定的工具控件可使用 card；圆角统一 `4px/6px/8px`，不得超过 8px。阴影只分 `none / low / modal` 三档，深色主题主要靠边框和表面明度建立层级。

### 3.5 细节、动效和视觉资产

- 延续现有应用图标、刀具 PNG 与原创 Three.js 场景，品牌/产品/真实功能资产必须在首屏可见；不新增无关图库或纯气氛图。
- 工作区网格可保留为极低对比的机关刻度，但只能作为背景纹理；禁止大面积彩色渐变。用 1px 线、角标、刻度、局部 `box-shadow` 表达科技感。
- 所有按钮至少 44px 可点击区域，图标按钮保持稳定 `40/44px` 方形尺寸并有 `title + aria-label`。不使用文本圆角块替代熟悉的关闭、刷新、删除、前后翻页图标。
- hover/focus 150-220ms，页面切换不超过 300ms；只动画 `opacity/transform/clip-path`。主操作按压、导航轨移动、扫描进度和状态确认可有一次性反馈，禁止所有卡片循环发光。
- `prefers-reduced-motion` 下取消 Three.js 非必要运动、扫描线、导航滑动和 stagger；功能、计时、关闭按钮仍可用。动画必须可中断，开屏不能阻塞主程序初始化。
- `App.vue` 已用 `defineAsyncComponent` 动态加载 `StartupIntro/EasterEggGame`，两个组件又动态加载 Three.js scene。保留这条边界并核对最终 chunk：主工作区不能同步引入 Three.js；给全屏层预留固定空间和非空 fallback。现有约 724KB 警告应确认属于独立 Three chunk，不能为消除警告而产生开屏白屏、尺寸跳变或把 Three 重新并入主 chunk。

### 3.6 响应式、可访问性与防回归

桌面验收尺寸固定为 `720x620`（Tauri 最小）、`960x700`（默认）、`1280x800`（宽屏）。现有 `860/760/700px` media query 可以继续使用，但要统一职责：

- 960 及以上：184px 侧栏、双列控制区、完整状态条。
- 720-959：侧栏转顶部紧凑导航或保持现有横向导航，图标和当前项可读，不能挤压版本按钮；控制区改单列。
- 所有尺寸：页面本身无水平滚动；仅命令/记分板等固定格式工具允许内部滚动。长中文、SteamID、路径和最长按钮文案不得溢出或遮挡。

补充 `tests/ui-design-contract.spec.ts` 或等价守护测试，至少验证：全局语义 token 完整；主要视图仍是语义 `nav/main/button/table`；图标按钮有 accessible name；当前页 `aria-current`、tab `aria-selected`、busy/disabled 状态正确；所有主要页面没有浏览器原生文件选择 fallback；reduced-motion 规则存在。不要写依赖每个像素值的脆弱 snapshot。

实现后做一次 CSS 颜色扫描，确认组件内没有新增未解释 raw color、主题不是单一青色/深蓝/棕色、普通文本对比度至少 4.5:1、大文本/边界至少 3:1。键盘从标题栏到导航、主要操作、表格操作、弹窗关闭完整走一遍；focus 不能被 overflow 裁切。

### 3.7 最终自动化

UI 与 Clippy 修复全部完成后再执行完整验证，不能沿用改 UI 前的 `npm run verify` 结果：

```powershell
Set-Location E:\CS2AS05
npm run verify
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.3\nades-pacing\tests\NadePacingPolicy.Tests.csproj -c Release
git diff --check
```

保存完整 stdout/stderr、命令、时间、exit code 到新目录，例如：

```text
workspace/release-evidence/0.5.5-final-YYYYMMDD-HHmm/
```

不要只保存数据库。至少生成 `environment.txt`、`git-status-before.txt`、`npm-verify.txt`、`cargo-test.txt`、`cargo-clippy.txt`、`nade-tests.txt`、`diff-check.txt` 和后续 JSON 摘要。证据目录不提交 Git。

再专项确认这些守护测试仍通过：

- 冷启动更新恰好一次，快速重复调用共用 in-flight 请求，手动检查仍为 `manual=true`。
- 选择 canonical CS2 root 前先调用 `ensureDefaultDemoRoot`，默认深度 5。
- 首次默认 `bots / Low / mixed / less / 八项全开 / 507,508,515,519,525`。
- 旧用户显式 `normal / 八项全关 / 空刀具` 保留。
- BOT userid 不合并；旧 cache schema/adapter/metrics 会重解析；强制重试不能被 size/mtime cache 命中。
- UI 设计契约、键盘/ARIA、亮暗主题、reduced-motion 与 Three.js lazy chunk 守护测试通过。

## 4. 阶段 B：最新版真实 Demo 回归验收

本发布只验证当前 CS2 新生成的样本，不测试、不修复、不承诺旧 Demo。固定回归样本：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260728-1717-de_inferno-advent.dem
```

执行：

```powershell
$env:CS2AS_DEMO = '<absolute-dem-path>'
cargo test --manifest-path .\src-tauri\Cargo.toml real_demo_scoreboard -- --ignored --nocapture
Remove-Item Env:CS2AS_DEMO -ErrorAction SilentlyContinue
```

不得只依据 `players` 非空断言。把打印 JSON 结构化摘要为 `demo-current.json`，至少记录样本路径/size/mtime/hash、耗时、`schemaVersion`、三个 adapter 版本字段、`scoreboardStatus`、`entityParseStatus`、玩家数、真人/BOT 数、T/CT/未知数、唯一有效 SteamID64 数、K/D/A/伤害/爆头非空覆盖率、round/kills 和 parser warning/error。

验收标准：

- 最新样本必须保持 strict + complete，10 名非观战真人、两队、SteamID64 唯一，统计和时间线可用。
- 当前样本出现 crash、hang、unknown parser error、空记分板或身份错误合并均阻塞发布。若解析超过合理上限，先记录进程/CPU/文件状态并定位，不要并发重跑掩盖问题。
- 旧 Demo 的 partial/unavailable、身份缺失和历史格式差异不阻塞 `0.5.5`，不要继续运行旧样本或扩张兼容范围。

可保留 schema/adapter/metrics 的缓存升级自动化测试，但无需拿旧 Demo 做人工验收。

## 5. 阶段 C：签名材料与最终构建

### 5.1 已确认的密钥

签名材料在：

```text
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key.pub
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater-password.dpapi
```

私钥 348 bytes、公钥 152 bytes、DPAPI 文件 622 bytes。公钥内容已经确认与 `src-tauri/tauri.conf.json` updater `pubkey` 解码结果一致。DPAPI 密文只能由当前 Windows 用户解密。绝不打印、记录、提交或上传私钥和密码；不要生成新密钥替换既有 release line。Tauri updater 签名与 Windows Authenticode 是两件事，本机当前没有 Authenticode 证书证据。

发布前再次无明文核对公钥：

```powershell
$conf = Get-Content .\src-tauri\tauri.conf.json -Raw | ConvertFrom-Json
$embedded = [Text.Encoding]::UTF8.GetString([Convert]::FromBase64String($conf.plugins.updater.pubkey)).Trim()
$disk = (Get-Content 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key.pub' -Raw).Trim()
if ($embedded -cne $disk) { throw 'Updater public key mismatch' }
```

### 5.2 安全注入并构建

先删除/移走同版本旧 EXE、旧 `.sig` 和 stale manifest 到 evidence 备份目录，避免脚本按文件名误取；不要删除 0.5.3/0.5.4 正式产物。确认没有 release Cargo 正在占锁。只在当前 PowerShell 进程恢复密码：

```powershell
$keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
$secure = Get-Content (Join-Path $keyDir 'updater-password.dpapi') -Raw | ConvertTo-SecureString
$credential = [PSCredential]::new('tauri-updater', $secure)
$env:TAURI_SIGNING_PRIVATE_KEY_PATH = Join-Path $keyDir 'updater.key'
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $credential.GetNetworkCredential().Password
try {
  npm run bundle:desktop
  if ($LASTEXITCODE -ne 0) { throw "bundle failed: $LASTEXITCODE" }
} finally {
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PATH -ErrorAction SilentlyContinue
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
  $credential = $null
  $secure = $null
}
```

必须产生：

```text
src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.5_x64-setup.exe
src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.5_x64-setup.exe.sig
```

记录 EXE 和 `.sig` 的 size/SHA256/mtime；`.sig` 应是非空 minisign/Tauri updater signature。`Get-AuthenticodeSignature` 预计仍为 `NotSigned`，必须如实记录，不能把 `.sig` 称为 Authenticode。最终用真实 `0.5.4` 客户端完成 updater 安装才是签名链的权威端到端验证。

然后立即生成 fresh manifest：

```powershell
$env:RELEASE_CHANNEL = 'prod'
try { npm run release:manifest } finally { Remove-Item Env:RELEASE_CHANNEL -ErrorAction SilentlyContinue }
```

用结构化 JSON 读取 `dist-release/cs2-bot-improver/updater-prod.json`，断言 `version/channel/projectId` 为 `0.5.5/prod/cs2-bot-improver`，installer 绝对路径是最终 EXE，`size` 和 `sha256` 重新计算一致，`signature` 与最终 `.sig` trim 后逐字一致，`pub_date` 晚于最终 EXE mtime。禁止发布当前 stale 值 `size=69,367,405`、`SHA256=099730C...`，也禁止发布未签名候选 `size=72,390,537`、`SHA256=74C28D56...`。

## 6. 阶段 D：最终安装器本机/隔离验收

所有验证都针对阶段 C 的最终 EXE；一次失败后重建就必须更新 hash、manifest、证据和后续上传文件。

### 6.1 全新安装

1. 备份现有应用数据和安装目录，用干净 Windows 用户或隔离 app-data 做新装；不要删除用户真实 CS2 配置。
2. 启动后确认从标题栏、主外壳到各业务页已经统一为“玄铁机括 · 青玉灵脉 · 赤金铭文”双主题；开屏可跳过，网络失败也不阻塞，真实公开鸣谢可显示，彩蛋动画连续、X/Escape 均能关闭。
3. 通过日志/请求证据确认应用冷启动恰好执行一次 `manual=false` 更新检查，进页不产生第二个自动请求，手动按钮仍可重试。
4. 选择用户 CS2 游戏根，确认 Demo root 自动出现且深度为 5；更换根时行为符合 origin 契约。
5. 在首次缺失配置场景确认 BOT/Low/mixed/less、八项全开和五把默认刀；检查实际两个 cfg、`core.json` 和 state JSON，不只看 Vue。
6. 扫描/导入 Demo，确认记分板横向不溢出窗口、BOT 显示 `BOT` 而不是 `0`，complete/partial/unavailable 文案准确；T/CT 分组、数字列和 sticky 表头在双主题下清晰可读。
7. 正常关闭应用，确认无遗留助手进程；启动/关闭 CS2 按钮只操作明确目标，不误杀 Steam。

### 6.2 覆盖安装与卸载

- 在保留的 `0.5.4` 数据上直接运行最终 `0.5.5` 安装器，确认用户选择目录、显式预设、八项全关、空刀具、Demo DB 和录制开关保留；仅新增缺失字段。
- 卸载后分别记录程序文件删除与用户 app-data/数据库保留策略。残留用户数据如果符合设计不能笼统判成卸载失败；程序二进制和注册项残留则必须修复。
- 再安装并启动一次，证明卸载/重装路径没有破坏 updater 或 SQLite migration。

UI 验收按大项目简化，但至少对概览、预设、BOT 物品、刀具、命令、对局复盘、安装诊断、更新弹窗、目录弹窗、toast、开屏和彩蛋检查 `720x620`、`960x700`、`1280x800`。每个尺寸保存亮/暗主题关键截图，检查键盘 focus、reduced-motion、loading/error/empty/disabled、长路径与长中文，无重叠、裁切和页面级横向滚动。使用真实 Tauri 桌面窗口确认，不能只用 Web preview，也不用浏览器原生文件/目录能力替代。

## 7. 阶段 E：用户实机结论的使用边界

用户已明确确认：使用当前 `0.5.5` 代码实际启动 BOT，打到第 5 回合后新 Demo 正常生成，应用内记分板正常。实际执行 AI 直接把这项记为已完成，不再要求用户重复操作，也不再测试旧 Demo。

阶段 A 的 Clippy 修复必须是语义等价改写。如果此后只改 UI 样式/布局、发布文档、签名环境、manifest 或打包流程，沿用本次用户实测结论。如果又修改 `src-tauri/src/services/demo.rs` 的解析/聚合逻辑、`third_party/demoparser/`、Demo IPC/model、记分板数据映射或可见字段语义，才必须重新用当前 CS2 新生成 Demo 做一次回归；仍不测试旧 Demo。

## 8. 发布说明（不要沿用脚本默认的 Steam-only 文案）

建议 title：`CS2 人机增强助手 0.5.5`
建议 summary：`整体升级古风科技桌面 UI，新增本地 Demo 对局复盘与自动录制，修复当前 CS2 新生成 Demo 的记分板身份解析，并完善启动更新、自定义 Steam 路径和首次默认配置。`

建议 items，D1、GitHub Release 和用户公告保持同源：

1. 整体升级“玄铁机括 · 青玉灵脉 · 赤金铭文”古风科技 UI，统一主工作区、数据表格、弹窗、状态反馈和亮暗主题。
2. 新增本地 Demo 录像库、目录扫描、手动导入、SQLite 报告、回合时间线与分队记分板。
3. 助手启动的 BOT/本地对局可自动录制，CS2 退出后自动识别并解析本次新 Demo；官方匹配录制仍由服务器或平台决定。
4. 修复当前 CS2 新生成 Demo 的玩家身份链，记分板支持真人 SteamID64、BOT userid/controller 和统计聚合。
5. 玩家选择的 CS2 游戏目录会作为默认复盘扫描目录，默认深度 5。
6. 冷启动固定检查一次更新，并保留手动检查和签名自更新；修复自定义 Steam 安装目录启动。
7. 全新配置默认道具为“较少”，BOT 物品八项全开；升级不会覆盖用户明确选择。
8. 新增古风科技 Three.js 开屏鸣谢与版本彩蛋，并修复动画冻结、数据获取和关闭交互。
9. 优化游戏/窗口关闭、目录发现、状态反馈、键盘操作与 reduced-motion 体验。

已知限制必须写明：Demo 仅本地处理、不上传；旧 Demo 不属于本版本兼容范围；未来 CS2 更新可能改变 Demo 格式，无法可靠解析时会显示具体状态而不猜测；自动录制只保证助手启动的 BOT/本地托管对局；部分插件环境的探员模型/刀具开关可能写盘成功但游戏内受上游限制；Windows 安装器有 Tauri updater 签名但若无证书仍没有 Authenticode；若最终构建仍有 Three.js chunk 体积警告，应记录实际大小和 lazy-load 边界。

## 9. 阶段 F：提交、tag、一次 push 与 GitHub Release

用户已明确 `0.5.5` 准备正式发布，但仍须等所有前置闸门通过。提交前：

1. 审阅全部 modified/untracked；确保 Demo 源码、vendor parser、许可证、测试和本方案均包含。
2. 排除 `workspace/`、签名密钥、DPAPI、真实 Demo、数据库、日志、截图、`target/` 和临时 release evidence。
3. 再跑 `git diff --check`、秘密扫描和版本一致性检查。不要提交 `dist-release` stale manifest；若仓库惯例需要提交 manifest，只能提交阶段 C 的 fresh 文件并确认不含本机敏感路径，必要时先把生成器改成可发布的相对/URL 元数据再重跑测试。
4. 创建一个可追溯 release commit 和 annotated tag：

```powershell
git add <reviewed source/docs/tests only>
git diff --cached --check
git diff --cached --stat
git commit -m "release: CS2 Bot Improver 0.5.5"
git tag -a v0.5.5 -m "CS2 Bot Improver 0.5.5"
git push origin main refs/tags/v0.5.5
```

只执行一次 push，不 force push；push 失败先判断网络，禁止重复创建提交/tag。回读远端 commit/tag。随后用 `gh release create v0.5.5` 创建非 draft GitHub Release，附最终 EXE、`.sig`、SHA256 文件、完整发布说明、许可证/上游说明和已知限制。GitHub Release 上传不替代 R2 readback。

## 10. 阶段 G：夸克、R2/D1 与线上启用

夸克分享链接由用户从最终 EXE 创建并提供；不得沿用 `0.5.4` 链接 `https://pan.quark.cn/s/4b812bb3b416`。收到新链接前不能运行生产 publish 脚本。

生产前从 `E:\cs2as` 保存 D1 release/settings、R2 对象清单、custom API 和两个 feed 的快照。当前线上 latest 是 `0.5.4`；custom API 对 `0.5.4/0.5.5` 都无更新，两个 Tauri feed 都为 204。重新确认 Wrangler 登录和 R2/D1 权限，不修改/提交官网 dirty 文件。

先 dry-run 并核对 title/summary/items，禁止使用脚本默认的两条 self-update 文案：

```powershell
Set-Location E:\cs2as
npm run self-update:publish -- `
  --installer 'E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe' `
  --sig 'E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe.sig' `
  --version 0.5.5 --quark-url '<USER_PROVIDED_0.5.5_URL>' `
  --title 'CS2 人机增强助手 0.5.5' `
  --summary '<approved summary>' `
  --items '<item1>|<item2>|...'
```

确认 dry-run 的 key、size、SHA256 与 final manifest 一致后，仅一次加 `--remote` 执行。既有脚本会上传 R2、GET 回读 hash/size、先 upsert D1 且 `updater_enabled=0`、确认 feed 仍为 204，再启用该版本。任何步骤失败均不手工跳过 hash 检查；若使用 `--skip-upload`，必须提供并核对 `--verified-readback`，且只能用于已证明同一对象的恢复执行。

启用后逐项回读：

- custom API，current `0.5.4`：HTTP 200、`hasUpdate=true`、latest `0.5.5`、完整新说明、夸克新链接、`selfUpdate.available=true`、size/hash 等于最终 EXE。
- custom API，current `0.5.5`：HTTP 200、`hasUpdate=false`、latest 仍为 `0.5.5`。
- Tauri feed，current `0.5.4`：HTTP 200，version `0.5.5`，download URL 指向正确 R2 对象，signature 与 `.sig` 一致。
- Tauri feed，current `0.5.5`：HTTP 204。
- 对 feed 返回 URL 做 GET，下载到新临时文件，重新计算 size/SHA256；检查 CORS 和 `Cache-Control`，不能只做 HEAD。
- 从真实已安装 `0.5.4` 点击应用内更新，完成下载、签名校验、passive 安装、重启到 `0.5.5`；用户数据和 Demo DB 保留。此项是 updater 签名最终权威验证。

线上最多部署官网 3 次；本发布正常不需要 Worker deploy。GitHub push 最多一次。群公告、内测邀请由用户执行。

## 11. 回退与停止条件

- 自动化、UI 实机验收、当前新 Demo 回归（若触发重测）或安装/升级失败：不提交正式 tag、不上传生产；保留证据修复后从受影响阶段重跑。旧 Demo 失败不阻塞。
- UI 只完成开屏/彩蛋、主工作区仍是旧皮肤，或任一主要页面出现重叠、文字溢出、不可见 focus、主题低对比、关闭按钮失效：停止签名构建，先修 UI；不能用“功能能点”代替发布质量。
- 签名失败或 `.sig`/manifest/hash 不一致：隔离旧产物，重新 bundle；不手工复制旧 `.sig`，不生成新密钥。
- Git push 前出现密钥、DPAPI、真实 Demo/DB、workspace 证据或未知二进制被 staged：取消这些路径的 staged 状态并重新审阅，不清理原文件。
- R2 GET hash/size 不一致：保持/恢复 `updater_enabled=0`，不启用 release。
- 启用后 feed/custom API/真实 0.5.4 自更新失败：立即在 D1 将 `0.5.5 updater_enabled=0`；必要时将 `0.5.5 is_active=0` 使 custom latest 回到 `0.5.4`。保留不可变 R2 对象供取证，不覆盖同 key；修复后用新版本发布，不悄悄替换用户已见过的同版本二进制。
- GitHub Release 已公开但 updater 回退：在 Release 顶部标注暂停下载/撤回，保留审计信息；不要删除证据伪装未发布。
- 用户说“停止操作”：立即停止后续测试、提交、tag、push、上传和生产写入，原样保留现场。

## 12. 最终报告模板

```text
结果：0.5.5 候选 / 签名候选 / 正式发布 / 已回退
Git 基线：before HEAD、release commit、v0.5.5 tag、remote 回读、push 次数
版本：npm / Cargo / Tauri / Cargo.lock / ZIP marker
自动化：npm verify、fmt、test、Clippy、Nade、diff-check（exit code + evidence path）
UI：设计 token、主要改动文件、亮/暗主题、720/960/1280 截图、键盘/ARIA、reduced-motion、chunk 结果
当前 Demo：路径/hash、strict/complete、玩家/队伍/SteamID、统计、回合、耗时
最终 EXE：absolute path、size、SHA256、mtime、Authenticode
Updater：.sig path/hash、公钥匹配、fresh manifest 摘要、0.5.4 端到端更新
安装：全新、覆盖、卸载、重装、冷启动更新、默认值、默认 Demo root depth 5
用户真实 BOT：用户已确认第 5 回合生成新 Demo且记分板正常；是否因后续 parser 改动触发重测
发布说明：title、summary、items、known limitations
GitHub：release URL、附件 hash
夸克：用户提供的 0.5.5 URL
线上：R2 key/readback、D1 release、custom API 两版本、feed 两版本、CORS/cache
残余风险：Authenticode、Three.js chunk/显卡兼容、CS2 Demo 格式、上游插件限制
回退：是否触发、D1 状态、R2 保留、后续动作
证据根目录：workspace/release-evidence/0.5.5-final-...
```

只有最终签名产物、当前新 Demo 回归、真实 `0.5.4` 自更新和全部线上回读都完成后，结果才能写“正式发布”。用户的新 Demo 实机结论已经完成；旧 Demo 不计入发布条件。
