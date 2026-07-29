# 0.5.5 战报死锁、安装恢复、Demo 目录与发布阻断修复交接方案

> 日期：2026-07-29
> 项目：`E:\CS2AS05`
> 目标版本：`0.5.5`
> 文档性质：给不共享上下文的“实际执行 AI”的独立实施说明
> 本轮边界：只调查和写方案；尚未修改运行时代码、构建、提交、推送或发布。

## 1. 最终目标与完成标准

当前候选在不打开独立战报窗口时基本正常；一旦打开“本局战报”，独立窗口白屏，主窗口关闭、版本彩蛋和其他交互一起失效。用户现场还出现两类恢复路径不清楚的问题：

1. 在“安装与诊断”选择了正确的 CS2 安装根目录后，因为插件尚未安装、`core.json` 不存在，概览页将它显示成 `[PANEL_IO]` 读取失败，让用户误以为目录选错。
2. 在“对局复盘”添加目录时，不知道应该选择安装根、`game\csgo` 还是 `replays`；选择安装根后，UI 没有明确显示“目录登记成功 / 发现多少 Demo / 解析是否成功”。

实际执行 AI 必须将当前工作树修成可发布的 0.5.5，但只有同时满足以下条件才算完成：

- 打开、切换、关闭独立战报窗口不会阻塞 Tauri 主线程；连续操作后主窗口仍可关闭、导航和彩蛋仍可用。
- 战报入口资源或 Vue 初始化失败时不展示不可关闭的纯白窗口；失败必须留在主窗口给出可操作反馈。
- 正确选择 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive` 后，未安装插件被识别为“待安装”，并能一步进入安装，不再显示 `[PANEL_IO]`。
- 已选 CS2 根自动关联正确 Demo 扫描目录；手动选错层级时能规范化或明确建议，不再让用户猜。
- `simple-rating-v1` 计分板保留且能展示；高级 KAST/经济同步数据不阻塞本版。
- Three.js 彩蛋单独和在战报打开/关闭后都能运行、关闭并释放资源。
- 完整自动化、真实 Windows Tauri、真实 Demo、签名安装和 0.5.4 升级门禁通过。
- 最终 EXE、`.sig`、manifest 来自同一次最终构建；生产 updater 能被真实 0.5.4 客户端验证。

自动化通过不等于实机通过。白屏、窗口关闭、WebView2、Three.js 和 updater 都必须保留真实 Windows 证据。

## 2. 当前基线，禁止回退

调查时仓库为：

```text
branch: main
HEAD:   1021a75
package/tauri version: 0.5.5
```

工作树已有大量未提交的 0.5.5 实现，包括但不限于：

```text
M  NOTICE.md
M  src-tauri/capabilities/default.json
M  src-tauri/src/{lib.rs,models/demo.rs,services/demo.rs,services/mod.rs}
M  src/components/AppShell.vue
M  src/views/DemoReviewView.vue
M  src/styles/main.css
M  src/types/demo.ts
M  vite.config.ts
?? scoreboard.html
?? src/scoreboard.ts
?? src/ScoreboardApp.vue
?? src/components/scoreboard/
?? src/styles/scoreboard.css
?? src-tauri/src/commands/scoreboard.rs
?? src-tauri/src/services/simple_rating.rs
?? tests/post-match-scoreboard.spec.ts
?? docs/*0.5.5*.md
```

执行前先保存 `git status --short`、`git diff --stat`、`git diff --name-status` 到新的证据目录。不要 `reset --hard`、不要 checkout 覆盖、不要从旧方案重新实现。当前改动通常允许随本版发布，但提交前仍要逐文件审查，禁止把 Demo、数据库、日志、`target`、密钥或 DPAPI 文件加入 Git。

## 3. 现场证据与根因结论

### 3.1 三张用户截图

- `3707c87234e4e0b0c0e5e2120fe97d3f.png`：对局复盘“添加 Demo 目录”对话框，用户询问选哪个文件夹。
- `2558dab60429cacd88e3caa5eb4e7d8f.png`：选择 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive` 后没有明确成功结果。
- `339bef5dc5a854cc9494e84a13a8ed16.png`：独立“本局战报”窗口已创建但内容完全白色；主窗口位于其后。

另一个任务 `019fabe5-d7c5-7231-a5a7-e655ae76c44f` 的调查确认：游戏安装根目录是正确的，失败时缺少的是：

```text
game\csgo\addons\counterstrikesharp\configs\core.json
```

这表示插件环境未安装完整，不表示用户选错目录。

### 3.2 白屏和全程序失控的决定性根因

当前 `src-tauri/src/commands/scoreboard.rs`：

```rust
#[tauri::command]
pub fn open_scoreboard(...) -> Result<(), String> {
    // ...
    WebviewWindowBuilder::new(...).build()
}
```

本机实际依赖 `tauri 2.10.3` 的源码：

```text
C:\Users\GOPtZ\.cargo\registry\src\...\tauri-2.10.3\src\webview\webview_window.rs:62-63
On Windows, this function deadlocks when used in a synchronous command and event handlers.
You should use async commands and separate threads when creating windows.
```

现有实现恰好从同步 Tauri command 调用 `.build()`。这能完整解释：WebView 只出现白色宿主、主窗口关闭请求不再处理、彩蛋和其他 invoke/交互都失效。此项是 P0 根因，不应先用 CSS、延时或重复 `show()` 掩盖。

### 3.3 独立窗口的次生缺陷

即使消除死锁，当前路径仍有以下风险：

- `scoreboard.html?reportId=...` 把 query 混入 `WebviewUrl::App`，生产资产解析没有独立实测证据。
- 新窗口事件监听建立前可能丢失 `demo://show-report`。
- `window.emit(...)` 目标语义不够明确，应使用 label 定向通信。
- `Number(null)` 得到 `0`，缺少 `reportId` 会错误请求报告 0。
- Vue mount 前没有 HTML fallback，入口 JS 加载失败时整窗纯白。
- `decorations(false)` 且关闭只依赖 Vue 按钮；JS 不运行时没有可见关闭逃生路径。
- `close()` 没有错误处理或 Rust 级 hide/destroy fallback。
- 主窗口从自动报告事件直接 fire-and-forget `invoke('open_scoreboard')`，错误不会 toast。

### 3.4 Demo 默认目录缺陷

`src/stores/cs2.ts::selectRoot()` 已调用 `ensureDefaultDemoRoot(status.rootPath)`，但 `src-tauri/src/services/demo.rs::ensure_default_root()` 保存的是 `cs2::normalize_root()` 返回的安装根。当前依靠扫描深度 5 从整个安装树碰巧找到 `.dem`，而不是明确保存 `game\csgo` 或 `game\csgo\replays`。

UI 又只显示路径和最后扫描时间，没有发现数、失败数、权限/遍历错误，也没有区分“添加成功”和“扫描成功”。截图中的“不成功”因此无法定位。

### 3.5 插件状态分类缺陷

`panel.refresh()` 无条件先调用 `initializePanelDefaults(root)`，随后读取 `core.json`。在 `environment.baseEnvironmentReady === false` 时，该文件不存在是正常“待安装”状态，却被转成 `[PANEL_IO]`。当前概览页没有把用户引导到“安装与诊断”。

### 3.6 `.sig` 现场事实

签名材料存在于：

```text
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key.pub
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater-password.dpapi
```

`updater.key.pub` 文件保存的是 Base64 文本；其 Trim 后与 `src-tauri/tauri.conf.json -> plugins.updater.pubkey` 逐字相同，长度均为 152。若把磁盘 Base64 文本与 Base64 解码后的两行 minisign 文本直接比较，会得到错误的 `False`，不要据此换 key。

现有 NSIS 目录被旧候选污染：

```text
0.5.5 EXE mtime: 2026-07-29 12:47:15
0.5.5 SIG mtime: 2026-07-28 21:46:57
```

二者绝不能作为一对发布。`.sig` 是 Tauri updater/minisign 签名，不是 Windows Authenticode；最终每次 EXE 内容变化都必须重新签名。

## 4. 实施顺序总览

严格按以下顺序执行，前一阶段未通过不要开始签名/发布：

1. 冻结基线并建立证据目录。
2. 消除同步动态建窗死锁，建立无竞态的战报协议和关闭逃生路径。
3. 修复插件未安装状态与安装引导。
4. 修复 Demo 自动目录规范化和扫描反馈。
5. 保留并验证简易 Rating；验证 Three.js 彩蛋没有独立回归。
6. 跑自动化和真实 Windows/Demo/CS2 验收。
7. 隔离旧候选，执行一次全新签名 bundle，验证安装与 updater。
8. 最后才审查、提交、一次 push、替换发布和生产回读。

## 5. P0：重构独立战报窗口

### 5.1 首选架构：启动时创建隐藏窗口，不在 command 中动态 build

最稳妥的 0.5.5 修复是在 `src-tauri/tauri.conf.json` 的 `app.windows` 中声明第二个窗口：

```json
{
  "label": "scoreboard",
  "title": "本局战报",
  "url": "scoreboard.html",
  "visible": false,
  "width": 1100,
  "height": 720,
  "minWidth": 860,
  "minHeight": 600,
  "resizable": true,
  "decorations": true
}
```

发布紧急修复优先保留原生标题栏，确保入口 JS 完全失败时仍能点击系统关闭按钮和使用 Alt+F4。窗口内部继续使用现有枪灰数据面板，不需要营销式页面。若最终坚持无装饰，必须先证明 JS 未加载时 Alt+F4 和 Rust `CloseRequested` 仍能隐藏窗口；否则不得关闭原生装饰。

`scoreboard.html` 不带 query。它在应用启动阶段隐藏加载，避开同步 command 动态创建 WebView 的 Windows 死锁。`capabilities/default.json` 已包含 `scoreboard`，保留并核对所需 window/event 权限。

只有当静态隐藏窗口被 Tauri 配置或安装包实测证明不可用时，才使用备选：把 `open_scoreboard` 改成 `async fn`，并严格按 Tauri 2.10.3 文档在独立线程创建窗口。不能只把函数签名机械加 `async` 后就宣称修复，仍须用 watchdog/实机证明主事件循环未阻塞。

### 5.2 Rust 管理权和握手协议

在 `src-tauri/src/commands/scoreboard.rs` 或独立 service 中添加托管状态，例如：

```rust
struct ScoreboardState {
    pending: Mutex<Option<PendingReport>>,
    frontend_ready: AtomicBool,
    sequence: AtomicU64,
}

struct PendingReport {
    report_id: i64,
    sequence: u64,
    requested_at_ms: i64,
}
```

建议 IPC：

```text
open_scoreboard(reportId) -> { accepted, sequence }
scoreboard_frontend_ready() -> pending report or null
scoreboard_present(reportId, sequence) -> show/unminimize/focus
hide_scoreboard() -> hide and clear transient UI state
report_scoreboard_boot_error(message) -> log + notify main
```

协议：

1. `open_scoreboard` 在 Rust 校验 report/schema 后只写入 latest pending，不创建 WebView、不等待前端渲染。
2. 若前端已 ready，使用 `app.emit_to("scoreboard", "scoreboard://load-report", payload)` 定向通知。
3. scoreboard 启动后先注册 listener，再调用 `scoreboard_frontend_ready()` 拉取 pending，消除“先 emit 后 listen”的丢事件。
4. 前端加载并成功渲染报告后调用 `scoreboard_present(report_id, sequence)`；Rust 只展示最新 sequence，旧请求不能抢回窗口。
5. 加载失败时保持窗口隐藏或保留上一份有效报告，在主窗口 toast“战报加载失败，可在对局报告中查看”，不得展示白窗。
6. 5 秒内没有 ready/present 时写 `runtime.log` 并通知 main；不要阻塞 command 等待 5 秒。

报告读取/JSON 解析若可能耗时，放入 async command 的 `spawn_blocking`，不要在 UI 线程做 SQLite/文件扫描。

### 5.3 关闭和应用退出

- scoreboard 的普通关闭语义为 `hide`，便于复用预加载窗口；Vue 关闭按钮调用受控 `hide_scoreboard` 并捕获错误。
- Escape 可关闭；按钮至少 44x44、带 tooltip/ARIA 和清晰 focus。
- 原生标题栏关闭或 Alt+F4 由 Rust `CloseRequested` 拦截并 hide，避免销毁预加载 WebView。
- 真正退出应用时不能拦截全局退出；main 的退出流程最终应 destroy/exit 全部窗口。
- main 有 pending updater 时会 `preventDefault()` 显示确认框。收到 main close-requested 时先隐藏 scoreboard，保证确认框可见且可操作。
- 自动报告弹窗失败只影响弹窗，不影响报告入库、主窗口导航或主程序退出。

### 5.4 HTML/JS 失败 fallback 与可观测性

在 `scoreboard.html` 内联最小非白背景和启动状态，Vue mount 成功后再替换：

```html
<body class="scoreboard-booting">
  <div id="scoreboard-app">
    <main class="boot-fallback" role="status">正在载入本局战报...</main>
  </div>
</body>
```

入口 `src/scoreboard.ts` 在 mount 前安装 `window.onerror` 和 `unhandledrejection`，以去敏后的错误消息调用 Rust bridge；禁止记录报告完整 JSON、Steam token 或密钥。Vue mount 后设置明确的 `data-scoreboard-mounted=true`，报告首屏完成后设置 `data-scoreboard-ready=true`，供真实窗口验收。

`ScoreboardApp.vue` 必须严格校验 `reportId > 0 && Number.isSafeInteger(reportId)`；不要再依赖 `Number(null)`。所有 `invoke/listen/hide` 都要捕获异常并呈现可恢复错误态。

`src/views/DemoReviewView.vue::popoutReport` 和 `AppShell.vue` 自动报告 listener 都要 await/catch，使用现有 toast 服务显示失败。自动路径不得强制切页，也不得因弹窗失败丢失已完成报告。

### 5.5 战报专项测试

新增/扩展测试：

- Rust：`open_scoreboard` 不调用 builder；pending/ready/present 状态机、latest-wins、非法 ID、旧 schema、窗口缺失、emit/show 失败。
- 前端：listener 先注册、ready 拉取、无 pending、有效报告、加载错误、close/hide 错误、sequence 过期。
- 构建：`dist/scoreboard.html` 存在且资产 URL 有效；生产安装包内能加载，不能只验证 Vite dev。
- 静态契约：同步 command 中不得出现 `WebviewWindowBuilder::build()`。
- 连续 20 次打开/隐藏、10 次切换两个 reportId，只存在一个 `scoreboard` label，无白屏、无事件泄漏。
- 操作期间每 250ms 记录主窗口 heartbeat；最大停顿建议不超过 500ms，打开后主窗口关闭请求必须在 1 秒内产生可见结果。

## 6. P0：修复“目录正确但插件未安装”

### 6.1 状态契约

将插件环境明确分成至少以下状态，不要用文件读取错误代替业务状态：

```text
noRoot          未选择目录
invalidRoot     不是有效 CS2 安装根
notInstalled    CS2 根有效，但插件/配置尚未完整安装
ready           插件和受管配置可读
corrupt         文件存在但 JSON/schema 损坏
permissionError 文件存在但无权读取/写入
```

可以在现有 `Cs2EnvironmentStatus` 上派生，也可以新增 `PanelAvailability` DTO；关键是 Rust/Pinia/UI 使用同一判定。不存在 `core.json` 且 `baseEnvironmentReady=false` 必须是 `notInstalled`，不是 `[PANEL_IO]`。

### 6.2 数据流修改

- `panel.refresh(root)` 先读取 `cs2.environment`/轻量 inspect。
- `notInstalled` 时不要调用 `initializePanelDefaults`、不要读取 `core.json`、不要创建半套配置；清空旧 root 的 snapshot。
- 只有 `ready` 才初始化缺省值和读取 panel snapshot。
- 文件存在但损坏时保持 `corrupt`，不得静默覆盖用户配置。
- 权限失败保留路径和 OS 错误码到诊断日志，UI 给“关闭 CS2/以有权限账户重试”等实际建议。

概览页显示安静、明确的内联恢复区：

```text
插件尚未安装
已识别 CS2：D:\...\Counter-Strike Global Offensive
[前往安装与诊断]
```

主导航状态由单一 view state 控制；点击按钮切换到 `install`，不要通过浏览器 URL、`window.open` 或 DOM 查询模拟点击。安装成功并回读 `baseEnvironmentReady=true` 后再刷新 panel。

### 6.3 验收样本

用截图中的真实路径按顺序验证：

1. 目录有效、插件完全不存在：显示“待安装”，无 `[PANEL_IO]`。
2. 点击 CTA 到安装页，目录保持不变，安装按钮可用。
3. 完成安装后 `core.json` 存在，概览/预设/物品正常。
4. 删除测试副本中的 `core.json`：状态回到待安装或安装不完整，不误报目录错误。
5. 写入损坏 JSON：显示 corrupt 并保护原文件，不归类为未安装。
6. 切换两个根目录时不能显示前一个 root 的 snapshot。

## 7. P1：Demo 目录规范化与结果反馈

### 7.1 自动路径

用户选择 CS2 安装根时，Rust 返回并持久化规范化安装根；Demo 服务从它显式派生：

```text
<CS2 root>\game\csgo
<CS2 root>\game\csgo\replays   （存在时作为重点来源）
```

推荐只登记 `game\csgo` 为 `origin=selected_cs2_root`，扫描深度 1 即可覆盖 `replays`，避免重复根和遍历整个游戏安装树。若希望分别登记 `replays`，必须 canonical dedupe，不能让同一 `.dem` 重复入库。

修改 `ensure_default_root()`：

- 先用 `cs2::normalize_root()` 得到安装根。
- 验证 `<root>\game\csgo` 存在。
- 将 canonical demo root 保存为 `<root>\game\csgo`，不是 `<root>`。
- 迁移/禁用旧 `origin=selected_cs2_root` 安装根记录，不删除用户手动 roots。
- root 切换后 watcher 和列表在同一事务/明确顺序刷新。

### 7.2 手动选择

“自动管理”是主路径，“添加其他 Demo 目录”降为高级动作。Rust 接受并规范化常见层级：

- 选安装根：建议/转换到 `game\csgo`。
- 选 `game`：转换到 `game\csgo`。
- 选 `game\csgo`：直接使用。
- 选 `game\csgo\replays`：直接使用。
- 选其他目录：只要可读仍允许作为手动 Demo 库，但明确显示它是自定义目录。

对话框标题和旁边的短说明写清“通常无需手动选择；自动扫描 `game\csgo\replays`”。不要在应用里堆叠操作教程。

### 7.3 扫描结果契约

扩展 `DemoScanResult`/root 状态，至少回传并显示：

```text
rootPath, scannedDirectories, discoveredDemFiles,
imported, cacheHits, parsed, failed, permissionErrors,
lastScanAt, lastErrorCode, lastErrorDetail
```

一次手动添加后立即对该 root 做窄扫描并 toast：

- “目录已添加，发现 3 个 Demo，解析 3 个。”
- “目录已添加，但未发现 .dem；自动录像通常位于 ...\game\csgo\replays。”
- “目录无法读取：拒绝访问”，而不是列表静默为空。

截图真实路径的验收要求：自动 root 最终显示 `...\game\csgo`，能发现已知样本：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260729-0958-de_dust2-advent.dem
SHA256 C74963C77651D40A63EE835E713386EFB0CF67EEE8B3618163593DE063E485D5
```

## 8. 简易 Rating 与 Three.js 彩蛋

### 8.1 简易 Rating

当前工作树已将不可完成的完整 OpenRating 收敛为 `simple-rating-v1`，使用已有 K/D/A、damage、completed rounds。继续按 `docs/simple-rating-release-handoff-0.5.5-20260729.md` 的公式、schema 4、缓存迁移和固定测试向量执行。

本方案覆盖其窗口、目录、签名和发布结论。不要重新引入 OpenRating 名称，也不要用 `0` 填充缺失指标。高级 KAST、trade、经济、swing 和逐 tick 全员状态移到后续版本，不阻塞 0.5.5。

必须验证当前真实 Demo 的双方玩家、K/D/A、ADR、HS%、简易 Rating、排序和 MVP，而不是只确认组件存在。

### 8.2 彩蛋

现象“战报打开后彩蛋不能玩”由 Tauri 主线程死锁充分解释，先修窗口，不要先重写 Three.js。死锁修复后分别验证：

1. 冷启动后连续点击版本号触发，倒计时、playing、finished、关闭正常。
2. 打开并关闭战报后再次触发，结果相同。
3. 彩蛋运行时打开/隐藏战报，两边仍响应；主窗口 close 正常。
4. 连续打开/关闭彩蛋 10 次，RAF、ResizeObserver、WebGL renderer 都按 session 销毁。
5. 用 frame counter 和 canvas 像素/位置变化证明动画在动；“组件 mounted”或单张非空截图不算通过。
6. `prefers-reduced-motion`、WebGL context lost、factory reject 都有非白 fallback。

保留 `tests/three-stage-loop.spec.ts`、`tests/easter-egg-close.spec.ts` 和版本触发测试。若脱离战报仍失败，再按 `docs/version-0.5.5-threejs-runtime-freeze-and-supporter-showcase-fix-plan-20260728.md` 定位 RAF 生命周期，不要把两个故障混为一谈。

## 9. 自动化验证

按顺序运行并保存命令、stdout/stderr、exit code、耗时：

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo test --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets
git diff --check
```

vendored demoparser 的既有 warnings 要单独记录；项目自身新增 warning 必须修复，禁止为了过门禁大改 vendor。

真实 Demo ignored test：

```powershell
$env:CS2AS_DEMO='D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260729-0958-de_dust2-advent.dem'
try {
  cargo test --manifest-path .\src-tauri\Cargo.toml real_demo_scoreboard -- --ignored --nocapture
  if ($LASTEXITCODE -ne 0) { throw "real demo test failed: $LASTEXITCODE" }
} finally {
  Remove-Item Env:CS2AS_DEMO -ErrorAction SilentlyContinue
}
```

证据放入新的、不可覆盖旧调查的目录：

```text
workspace/release-evidence/0.5.5-release-blockers-<timestamp>/
```

该目录默认不提交。

## 10. 真实 Windows/Tauri/CS2 验收

实际执行 AI 准备候选，用户负责真实游戏和桌面交互确认。验收要尽量短，但不能省略决定性步骤。

### 10.1 窗口矩阵

在 dev 和生产 bundle 各做一次关键路径：

1. 主窗口冷启动，关闭开屏，确认导航/关闭可用。
2. 从录像库打开已知 dust2 report，窗口首帧不是白色，计分板完成展示。
3. 关闭/再开 20 次；在最小化后再打开；两个报告交替 10 次。
4. 战报加载中立即点 main 关闭；有/无 pending updater 两种状态都给出正确结果。
5. 模拟错误 reportId、旧 schema、入口 JS 报错；主程序不死锁，错误可见，窗口可关闭。
6. 打开战报后运行彩蛋并关闭，再关闭主程序。
7. 任务管理器观察无持续高 CPU；窗口 label 不累积。

保存至少：成功计分板截图、故障 fallback 截图、runtime log、窗口 heartbeat 摘要。

### 10.2 目录与安装矩阵

1. 在未安装插件的测试基线选择截图路径，出现待安装 CTA。
2. 安装后真实回读 `core.json`、两套 gameinfo backup 和受管 cfg，不能只看 UI。
3. 对局复盘自动显示 `game\csgo`，扫描已知 Demo 并展示数量。
4. 分别手动选择安装根、`game`、`game\csgo`、`replays`，验证规范化和去重。
5. 用无 `.dem` 空目录和无权限目录验证明确结果。

### 10.3 新一局 BOT

用助手启动 BOT/本地对局，至少完成 3 回合并正常退出：

- 新 Demo 被观察器识别、稳定后解析。
- 报告先可靠入库，再尝试弹窗；弹窗失败不丢报告。
- 自动出现的独立窗口可关闭，主窗口仍响应。
- 新 reportId、双方、回合和简易 Rating 合理。

若 watcher 没弹但手动扫描可解析，仍是 0.5.5 发布阻塞，不能推给未来 session state machine。

## 11. 签名构建和安装门禁

### 11.1 隔离旧候选

将现有同名 0.5.5 EXE、`.sig`、`.sha256`、旧 manifest 移到本次 evidence 的 `stale-artifacts`，保留审计但避免脚本按文件名误取。不要删除 0.5.3/0.5.4 正式产物。移动前后记录路径、size、mtime、SHA256。

### 11.2 安全恢复签名环境并构建

只在同一个 PowerShell 进程内解密 DPAPI 密码，不打印私钥或密码：

```powershell
$ErrorActionPreference = 'Stop'
$keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
$encrypted = (Get-Content (Join-Path $keyDir 'updater-password.dpapi') -Raw).Trim()
$secure = ConvertTo-SecureString $encrypted
$credential = [PSCredential]::new('tauri-updater', $secure)
$env:TAURI_SIGNING_PRIVATE_KEY = (Get-Content (Join-Path $keyDir 'updater.key') -Raw).Trim()
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = $credential.GetNetworkCredential().Password
try {
  npm run bundle:desktop
  if ($LASTEXITCODE -ne 0) { throw "bundle failed: $LASTEXITCODE" }
  $env:RELEASE_CHANNEL = 'prod'
  npm run release:manifest
  if ($LASTEXITCODE -ne 0) { throw "manifest failed: $LASTEXITCODE" }
} finally {
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
  Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
  Remove-Item Env:RELEASE_CHANNEL -ErrorAction SilentlyContinue
}
```

安装器构建最多 5 次。网络慢时等待，不要通过重复点击并发构建。

最终必须生成且相互对应：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe.sig
E:\CS2AS05\dist-release\cs2-bot-improver\updater-prod.json
```

验证：

- EXE/`.sig` 都在本次 bundle 后生成，mtime 相邻，size 非零。
- 记录 EXE SHA256；manifest 的 version、path、size、hash、signature、pub_date 与本次构建一致。
- 使用项目实际 CLI 的 `tauri signer verify --help` 确认当前语法后离线验签；不要猜参数。
- 从真实 0.5.4 客户端执行 updater 下载/验签/被动安装/重启，这是最终权威验证。
- `Get-AuthenticodeSignature` 若为 `NotSigned` 如实记录；它与 updater `.sig` 是两套机制。

### 11.3 安装矩阵

依次完成：

1. 全新安装 0.5.5，启动、选目录、打开已知战报、关闭。
2. 从正式 0.5.4 覆盖安装，确认配置、Demo roots、SQLite 报告库保留并迁移。
3. 卸载/重装，确认没有依赖开发目录资产。
4. 正式 0.5.4 通过生产候选 updater 升级到最终 0.5.5，签名通过并重启。

## 12. GitHub、生产更新与回读

发布前重新采集，不得沿用旧文档的远端状态：

```powershell
git status --short
git rev-parse HEAD
git fetch origin --tags
git rev-list --left-right --count HEAD...origin/main
git rev-parse 'v0.5.5^{}'
gh release view v0.5.5 --repo YuGeYu/CS2AS05 --json url,isDraft,publishedAt,assets
```

同时在 `E:\cs2as` 回读生产 R2/D1、自定义更新 API 和 Tauri feed。只有以下全部成立才允许替换同版本：

- 旧 GitHub 0.5.5 资产仍为零下载，且证据已保存。
- 生产 updater 尚未向 0.5.4 用户启用旧 0.5.5。
- 本方案全部功能/实机/签名/安装 Gate 通过。
- staged diff 已逐文件审查，无 evidence、Demo、DB、日志、密钥、DPAPI、target 或未知二进制。

提交当前经审查的完整 0.5.5 工作树和本方案。旧调查文档可以保留为历史证据，但应在顶部标注被本方案覆盖的具体章节，防止后续执行 AI误用“同步动态建窗”设计。

项目要求 GitHub push 最多一次；上传/Release 也只做一次完整尝试，失败后停止并报告，不循环重试。Quark 分享链接由用户生成并提供。生产发布先 dry-run，再只执行一次 `--remote`，之后必须 GET 回读 R2 exact hash/size、D1 记录和 updater feed；中间服务成功不等于客户端最终升级成功。

## 13. 明确禁止项

- 禁止继续从同步 command/event handler 调用 `WebviewWindowBuilder::build()`。
- 禁止用延时、重复 show/focus、CSS 背景把死锁伪装成已修。
- 禁止让用户反复换目录来掩盖插件未安装。
- 禁止不存在的 `core.json` 被归类为损坏或通用 I/O 错误。
- 禁止扫描整个 SteamLibrary 作为默认 Demo 路径。
- 禁止把组件 mount、单张截图或自动化当成 Three.js/窗口实机通过。
- 禁止复用旧 `.sig`、生成新 updater key、打印密码或提交签名材料。
- 禁止在 Gate 未通过时提交 tag、上传 GitHub、启用 R2/D1 updater。
- 用户说“停止操作”时立即停止所有状态变更。

## 14. 实际执行 AI 的最终报告格式

完成后向用户按以下结构报告：

1. **结果**：是否已达到可发布/已发布；若未达到，明确唯一阻塞 Gate。
2. **关键修复**：同步动态建窗如何移除；握手/关闭/fallback；插件和 Demo 目录恢复路径。
3. **自动化证据**：命令、exit code、证据目录。
4. **实机证据**：Windows 战报、主窗关闭、彩蛋、真实 Demo/BOT、安装/updater。
5. **发布产物**：EXE/`.sig`/manifest 的绝对路径、size、SHA256、mtime。
6. **Git/线上状态**：commit/tag/Release、R2/D1/feed 回读；没有执行的动作必须明确说没有执行。
7. **剩余约束**：高级 KAST/经济/逐 tick 指标属于后续版本，不冒充本版能力。

只有从干净安装或正式 0.5.4 基线能最小步骤复现成功，才能把 0.5.5 标记为完成。
