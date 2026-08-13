# CS2 Demo 功能：执行 AI 预验收收口方案（2026-08-03）

> 交接对象：下一位“实际执行 AI”。
>
> **负责人结论：下一步仍由执行 AI 完成，不需要用户现在介入。** 当前剩余项均是最终 debug EXE 的真实 Tauri 回归、受控窗口取证和本机性能测量。只有本文全部 Gate 通过后，执行 AI 才能请求用户进行一次真实 BOT/CS2 验收。
>
> 本文是 `demo-platform-execution-ai-preflight-plan-20260803.md` 的收口续作。已完成的地图导入、SQLite v7、真实雷达热力图、Canvas 单元测试和 Web fixture 不得重做；以 `demo-platform-execution-ai-preflight-report-20260803.md` 为当前事实基线。

## 1. 本轮唯一目标与结论边界

本轮只关闭以下七类阻断项：

1. 重新构建并启动**最终 hash 对应**的 debug EXE，完成真实 Tauri 回归。
2. 精确设置并验证四种真实窗口尺寸：主窗口 `1440x900`、`1100x700`；独立战报窗口 `1280x800`、`980x640`。
3. 测量 viewer 播放、暂停、切离空间页三种状态的 CPU，并证明 tick/绘制生命周期符合状态。
4. 重复进入/离开 viewer 20 次，测量 WorkingSet、PrivateMemory 和 HandleCount 趋势。
5. 测量 cold start 到“可交互”的真实时间。
6. 测量真实 Tauri viewer FPS。
7. 测量真实 Tauri `get_round_positions` 完整 invoke、Rust decode 和 IPC 的 P95，并重新完成无歧义的 live DB 静止性取证。

本轮不新增产品功能、不扩展 Demo 指标、不重做 UI、不改默认窗口大小、不改变用户数据。结果只能二选一：

- `AI 预验收未完成`
- `AI 预验收通过，可以请求用户 BOT 验收`

即使通过，也不能称为“功能候选”“安装候选”“发布候选”。这些名称仍分别受用户 BOT/CS2 验收、安装覆盖验收和发布授权约束。

## 2. 权威基线与执行纪律

### 2.1 开始前必须回读

```powershell
Set-Location E:\CS2AS05
git status --short
git rev-parse HEAD
git diff --stat
Get-Content -Raw .\docs\demo-platform-execution-ai-preflight-report-20260803.md
Get-FileHash .\src-tauri\target\debug\ai_pc_fac.exe -Algorithm SHA256
```

当前报告记录的旧候选仅是对照，不是本轮可复用的最终候选：

- HEAD：`8552b554993fec66866d2133d315342be3e0cbca`
- tracked diff fingerprint：`ac7d4b9ecec2396faf9e6b969f56f533222fc5c9`
- EXE：`E:\CS2AS05\src-tauri\target\debug\ai_pc_fac.exe`
- 旧 EXE SHA-256：`5371234979525F1A3159BDD6C67991A44AD3D5A91E844BC95778BE697D866848`
- 隔离 DB：`C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover.preflight\demo-review\demo-review-v1.sqlite3`
- 既有 evidence：`E:\CS2AS05\workspace\release-evidence\demo-preflight-20260803-20260803-045833`

如果加入任何诊断代码，旧 EXE hash 立即失效。必须重新 build、计算新 hash、启动新 EXE并从头完成本轮真实 Tauri 证据，禁止把不同 hash 的截图或性能数据拼成一个候选。

### 2.2 工作树和进程纪律

- 保留现有大量未提交改动；禁止 `git reset`、`git restore`、`git clean`、新开分支或覆盖用户文件。
- 不触碰来源不明的历史 PID；只管理本轮启动且已记录 PID/start time/path 的进程。
- 启动前确认没有本轮遗留的 `ai_pc_fac`、Vite 或 fixture 浏览器；端口冲突时换端口，不杀未知进程。
- 不删除或原地修改历史 DB、Demo、迁移备份和旧 evidence；本轮 evidence 使用新目录。
- 不 commit、push、release、deploy，不构建 installer，不签名，不启动真实 CS2。
- 命令失败必须保留 stdout、stderr、exit code 和时间；不能删失败日志后只报最终成功。

新 evidence 根目录：

```text
E:\CS2AS05\workspace\release-evidence\demo-preflight-closeout-20260803-<HHmmss>\
  baseline\
  automation\
  runtime\
  windows\
  screenshots\
  database\
  performance.json
  performance.csv
  execution-report.md
```

## 3. 先建立受控 QA 工具，不靠人工拖窗口

### 3.1 Win32 精确尺寸脚本

优先新增 `scripts\set-tauri-window-rect.ps1`，通过 PowerShell `Add-Type` 声明最小 Win32 API：

- `EnumWindows` 或 `Get-Process(...).MainWindowHandle` 获取 HWND；
- `GetWindowThreadProcessId` 反查 PID，防止命中同名旧窗口；
- `IsWindowVisible`、`GetWindowTextW` 校验窗口；
- `SetWindowPos` 设置**窗口外框**尺寸；
- `GetWindowRect` 回读实际外框；
- `GetClientRect` 和 `ClientToScreen` 记录客户区；
- `GetDpiForWindow` 记录 DPI，display scale 记为 `dpi / 96`；
- `DwmGetWindowAttribute(DWMWA_EXTENDED_FRAME_BOUNDS)` 可作为阴影/边框差异的辅助值。

脚本参数至少包含：`-ProcessId`、`-Width`、`-Height`、`-OutputJson`，可选 `-WindowTitle` 或 `-WindowKind main|scoreboard`。它必须：

1. 校验 PID 的 executable path 正是本轮 hash 对应 EXE。
2. 只操作属于该 PID 的 visible top-level HWND。
3. 调用后等待布局稳定，再回读 rect；允许 DWM 截图外沿因阴影产生少量差异，但 `GetWindowRect` 目标误差应为 `<=2 px`。
4. 输出 PID、process start、HWND、title、requested rect、window rect、client rect、DPI、scale、timestamp。
5. 找不到唯一窗口时失败，不通过标题模糊匹配随便选一个窗口。

不要再使用鼠标拖拽、系统菜单或修改生产默认窗口尺寸。窗口定位脚本是 QA 工具，不是产品功能。

### 3.2 截图和像素检查

新增或扩展只用于 evidence 的 PowerShell/Node QA 脚本，按 HWND 捕获真实 WebView2 顶层窗口。若 `PrintWindow` 对 WebView2 返回黑屏，使用 Windows Graphics Capture、现有 Computer Use 截图能力或可靠的屏幕区域捕获；无论采用哪种方式，都必须将 PNG 真正落盘并记录对应 HWND/rect，不能只保留会话中的视觉观察。

每张截图记录：

- 路径、SHA-256、PNG width/height；
- PID、HWND、EXE SHA-256、window/client rect、DPI/scale；
- 页面、Demo id、round、viewer tick/points 或 heatmap filters；
- 非空像素比例、唯一颜色数、Canvas 采样区域统计；
- 是否出现横向溢出、控件重叠、文字裁断、黑屏、透明区或 Canvas 纯色。

像素检查不得只以“颜色很多”判定成功。至少组合验证截图非空、Canvas 区域有雷达纹理、viewer 有位置标记或 heatmap 有热点，并把裁剪区域坐标写入 JSON。

## 4. Debug-only 诊断通道

现有自动化无法可靠测量 cold start、FPS 和完整 IPC P95。允许加入最小诊断，但必须满足：

- 前端仅在 `import.meta.env.DEV` 下编译/调用；Rust 端仅在 `#[cfg(debug_assertions)]` 下注册或写入。
- 不新增用户可见 overlay、按钮、路由、菜单或 production telemetry。
- 不依赖浏览器原生下载、`window.open`、File System Access API 或 DevTools 人工读数。
- 输出只写本轮明确传入或环境变量指定的 evidence 目录，路径做 allowlist/规范化；不写用户 Demo 内容、DB 内容或密钥。
- 每条事件为结构化 JSON/JSONL，含 monotonic timestamp、wall clock、PID、session id、event name 和必要维度。
- 文件写入失败要明确报错；不能静默退回猜测值。

建议统一使用 `CS2AS_DEMO_PREFLIGHT_EVIDENCE_DIR` 启用诊断。未设置时诊断完全不工作。避免创建多个互不兼容的测量格式；最终由现有 `scripts\measure-demo-preflight.ps1` 汇总为 `performance.json` 和 `performance.csv`。

### 4.1 Cold start 定义

Rust 在进程入口尽早记录 `process_started`。前端必须在以下条件全部满足后发出一次 `interactive_ready`：

- Vue 已 mount；
- 主工作台已渲染；
- 首次数据库/library 请求已完成；
- 录像库可点击，loading/阻塞层已消失；
- 至少下一次 `requestAnimationFrame` 已提交到画面。

`coldStartInteractiveMs = interactive_ready.monotonic - process_started.monotonic`。窗口句柄出现、DOMContentLoaded 或 Vue mount 单独发生都不能冒充可交互。至少做 5 次真正冷启动：每次关闭进程、确认退出、间隔 2 秒再启动；报告每次值、median、P95。不得清系统缓存或破坏用户环境来制造“冷”。目标仍为 `<=2500 ms`；超标如实阻断。

### 4.2 Viewer FPS 与状态事件

在 viewer 已加载真实 round positions 后，以现有 RAF loop 旁路计数，不另起装饰动画。输出：

- `viewer_mounted` / `viewer_unmounted`；
- `viewer_play_started` / `viewer_paused`；
- `viewer_tab_hidden` / `viewer_tab_visible`；
- 每个测量窗口的 RAF callback count、elapsed、FPS、tick start/end、points、viewport、DPI。

FPS 仅统计 10 秒稳定播放窗口，排除首帧加载；记录平均值、1 秒分桶最小值和 dropped/long-frame 数。目标为接近 60 FPS；稳定 `>=30 FPS` 可通过，但必须记录真实 viewport、points 和 DPI。暂停和切页期间应由既有组件测试证明 RAF=0，真实运行诊断也必须显示无 viewer frame callback；不能仅凭进程 CPU 下降判定 cleanup。

### 4.3 Round positions decode P95

在真实 Tauri 前端对同一真实 Demo/round 连续调用 `get_round_positions` 30 次：

1. 每次调用前记录 `performance.now()`。
2. 等待 Tauri `invoke` promise 完整 resolve。
3. 记录完整 frontend invoke + Rust 读取/decode + IPC + JS 反序列化时间、返回点数和 round id。
4. 前 3 次作为 warm-up，不进入 30 次正式样本。

最终 `roundPositionsDecodeP95Ms` 使用正式 30 次端到端样本，目标 `<=100 ms`。Rust 内部 decode timing 可以作为分解证据，但不能替代端到端 P95；Web mock、SQLite CLI 或历史 spatial job elapsed 也不能替代。

## 5. 最终候选构建和身份锁定

完成 QA/诊断实现及相关测试后，先跑针对性测试，再构建：

```powershell
Set-Location E:\CS2AS05
npm run typecheck
npm test -- --pool=threads --maxWorkers=1
cargo test --manifest-path .\src-tauri\Cargo.toml --lib
cargo build --manifest-path .\src-tauri\Cargo.toml
```

将以下内容写入 `baseline\candidate.json`：

- HEAD、`git status --short`、`git diff --stat`；
- `git diff --no-ext-diff | git hash-object --stdin` 的 tracked diff fingerprint；
- EXE absolute path、bytes、mtime UTC、SHA-256；
- build command、exit code、start/end/duration；
- package identifier，必须继续使用隔离的 `com.aipc.cs2botimprover.preflight`；
- DB absolute path、启动前 DB hash/size/mtime、WAL/SHM 状态。

随后只通过记录的绝对路径启动 EXE，核对进程 executable path 和 SHA-256，再记录 PID/start time。若任何源码、资源、配置或 lockfile 此后变化，候选身份作废，必须重新 build/hash/relaunch，并废弃旧候选的运行证据。

## 6. 真实 Tauri 四尺寸验收矩阵

### 6.1 页面和窗口责任边界

主 Demo 工作台只验证：

| 窗口 | 尺寸 | 必须取证 |
| --- | --- | --- |
| main | `1440x900` | library、overview、rounds、viewer、heatmap |
| main | `1100x700` | library、overview、rounds、viewer、heatmap |

独立战报窗口只验证：

| 窗口 | 尺寸 | 必须取证 |
| --- | --- | --- |
| scoreboard | `1280x800` | scoreboard 主表、滚动/固定区域、窗口返回/关闭 |
| scoreboard | `980x640` | scoreboard 主表、滚动/固定区域、窗口返回/关闭 |

不要错误要求独立 scoreboard 窗口展示 viewer 或 heatmap。若 economy、duels、utility、events 属于主战报页且能稳定导航，可在两个 main 尺寸补证，但不是用它们替代上述必需页面。

### 6.2 每个尺寸的操作顺序

1. 用 PID/HWND 设置精确外框尺寸并回读 JSON。
2. 打开录像库，确认 4 个历史 Demo，选择报告中 dust2 demo id 4。
3. 验证 overview 为 5 completed rounds、10 players，未知 Rating 仍显示 unavailable/`--`，不伪造评分。
4. 打开 viewer，等待真实 8208 points/对应 round 加载；截图静止帧，再播放观察 tick 推进，暂停后确认 tick 稳定。
5. 打开 heatmap，确认真实 dust2 雷达和橙红热点非空；筛选控件可见且不会挤压 Canvas。
6. 独立打开 scoreboard 窗口，以其 PID/HWND 分别设置两个尺寸并截图。
7. 每次导航后检查 document/client 横向溢出、焦点可见、键盘可达、loading/error 状态和 Canvas 容器稳定尺寸。

视觉验收沿用通用蓝色 SaaS / Data-Dense Dashboard，不在本轮改版。重点检查 `#1E40AF` 主色、`#3B82F6` 次色、`#D97706` 数据强调是否保持可读；focus ring 可见、正文对比度、150-300 ms 状态过渡和 `prefers-reduced-motion` 必须保持。禁止为截图添加装饰性持续动画。

阶段 Gate：四个真实 rect 全部达到目标，必需截图落盘且绑定同一 EXE hash，页面无重叠/横向溢出/黑屏/纯色 Canvas，viewer tick 和 heatmap 热点均有运行证据。

## 7. CPU、FPS 和 20 次生命周期测量

### 7.1 采样契约

使用同一最终 EXE、同一 dust2 demo/round、同一主窗口尺寸（优先 `1440x900`）和同一进程。每 `250-500 ms` 采样：

- timestamp；
- process total CPU time，并用相邻样本 delta / elapsed / logical processor count 计算规范化 CPU percent；
- WorkingSet64；
- PrivateMemorySize64；
- HandleCount；
- viewer state、tick、RAF count。

不要把任务管理器瞬时显示或累计 `Process.CPU` seconds 当作 CPU percent。每段先稳定 2 秒，再正式采样 10 秒：

1. viewer active/play；
2. viewer paused；
3. 切到 overview/rounds 等非空间页，即 tab-away。

输出三段 raw samples、mean、median、P95、peak。paused 和 tab-away 必须同时满足：tick 不推进、viewer RAF callback 为 0、CPU 相比 active 有明确回落；若进程存在其他后台任务，记录任务和绝对 CPU，不能硬设一个虚假固定降幅。

### 7.2 20 次进入/离开

保持进程不重启，完成 20 个完整 cycle：进入 viewer -> 等真实 positions/Canvas ready -> 离开到非空间页 -> 等待卸载稳定。每次在进入后和离开稳定后分别采样 WorkingSet64、PrivateMemorySize64、HandleCount，并记录 `viewer_mounted/unmounted` 计数。

对每个指标输出：first、last、min、max、delta、线性回归 slope、后 5 次 median 相对前 5 次 median 的变化。判定原则：

- 不要求每次完全相同，也不因 GC/allocator 暂留一次峰值失败。
- viewer mount/unmount 必须各 20 次且成对，无残留 RAF/observer。
- HandleCount 不得随 cycle 持续近似单调上升。
- WorkingSet/PrivateMemory 不得呈持续线性增长且在后半程不趋稳。
- 出现明显增长时先再做 10 次确认；仍增长则阻断并保留 raw samples，不通过重启进程掩盖。

## 8. 扩展现有测量脚本，不创建竞争格式

扩展 `scripts\measure-demo-preflight.ps1`，保持当前参数和既有字段兼容，可增加例如 `-RuntimeJsonl`、`-WindowEvidenceDir`、`-DbStabilityJson` 等可选参数。最终 `performance.json` 至少完整包含：

- candidate HEAD/diff fingerprint/EXE hash；
- cold start 5 次 raw/median/P95；
- library 500 query P95 和 real heatmap query P95；
- round positions 30 次端到端 raw/P95/points；
- viewer FPS 和 1 秒分桶；
- active/paused/tab-away raw CPU/process samples；
- 20 次 cycle raw samples、first/last/max/slope；
- 四尺寸 screenshot/rect/DPI/hash 索引；
- DB 静止样本和副本诊断结果。

已有 library P95 `26.274 ms`、heatmap P95 `36.392 ms` 只作为历史对照。本轮脚本重跑后写新 evidence，不覆盖旧目录。确实无法测量的字段保持 JSON `null`，并在 `notes` 给出具体失败点；任何必需字段为 `null` 时 Gate 不通过。

为聚合和趋势计算新增纯函数测试：P95 小样本边界、CPU delta、线性 slope、缺样本保留 null、错误 EXE hash 拒绝合并、不同 session id 拒绝合并。CSV 是摘要，JSON/JSONL raw samples 是权威数据。

## 9. 无歧义的数据库退出稳定性

不要在 live DB 的 10 秒静止采样期间运行 `sqlite3`、应用查询或任何 reader；reader 自己可能创建 SHM。

严格按以下顺序：

1. 正常关闭本轮启动的 Tauri、Vite 和 fixture/browser；记录 exit time/code。
2. 以 PID/path/start time 确认本轮 writer 已退出；确认没有本轮 analysis job/lease 仍在运行。
3. 仅用文件系统 API 连续 10 秒、每 2 秒采样 6 次 live DB：size、mtime UTC、SHA-256、WAL existence/size、SHM existence/size。
4. 采样期间禁止打开 live SQLite；六次主 DB hash/size/mtime 必须稳定，WAL 不得含未提交内容。
5. 采样结束后，用文件系统复制主 DB及当时必要 sidecar 到 `database\diagnostic-copy\`。若 WAL 非零，不自行丢弃，先阻断并保留现场。
6. 只对 evidence 副本运行 `PRAGMA integrity_check`、`PRAGMA user_version`、`map_metadata`、jobs/leases 查询。
7. 报告分别列出 `liveFileStability` 和 `diagnosticCopySqliteReadback`，禁止把副本查询产生的 SHM 写回 live 状态时间线。

SHM 存在本身不等于 writer 活跃。权威判定是：无本轮活动进程/lease、live 主 DB 六次稳定、WAL 无未提交内容、诊断副本 `integrity_check=ok` 且 `user_version=7`、`map_metadata=44`。

## 10. 自动化与最终重跑

针对性测试通过、真实 Tauri 取证完成后，关闭进程并串行运行全套；Cargo 不并行运行：

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
cargo build --manifest-path .\src-tauri\Cargo.toml
git diff --check
git status --short
```

注意：最后一次 `cargo build` 若改变 EXE hash，则此前运行证据不再对应最终 EXE。执行 AI必须比较 build 前后 hash：

- hash 未变：记录一致性，可沿用本轮运行证据。
- hash 改变：必须以新 hash 重新启动，并至少重跑四尺寸、CPU/FPS、20 次 cycle、cold start 和 decode P95；不能只做一次冒烟。

将每条命令写入 `automation\results.json`，保存独立日志。vendored demoparser 固定 10 条 warning 可如实记录，但项目 crate 在 `-D warnings` 下必须通过；Three.js chunk warning单列，不声称零 warning。

## 11. Gate、停止条件和禁止替代

出现任一情况，结果必须为 `AI 预验收未完成`：

- 任一必需指标仍为 `null`、样本数不足或使用估算值。
- 截图来自 Web fixture、旧 EXE、不同 hash，或未落盘绑定 HWND/rect。
- 任一目标尺寸未达到，页面有重叠、横向溢出、黑屏或纯色 Canvas。
- cold start 只测到窗口出现，不是 interactive ready，或 P95 超过 2.5 秒。
- positions P95 不是 30 次真实 Tauri 端到端 invoke，或超过 100 ms。
- viewer 稳定 FPS 低于 30，暂停/切页仍有 viewer RAF/tick 推进。
- 20 次 cycle 显示持续内存/handle 增长或 mount/unmount 不成对。
- live DB 静止采样被 SQLite reader 污染、主 DB不稳定、WAL 未收敛或副本 integrity 失败。
- 最终自动化失败、项目 crate clippy 失败，或最终 EXE hash 没有对应完整运行证据。
- 为过 Gate 修改生产默认窗口、隐藏错误、清理用户工作树、启动 CS2或触发发布动作。
- 用户要求停止。

禁止以下替代：Web fixture 代替 Tauri、Rust benchmark 代替 IPC、历史 job elapsed 代替 decode、窗口句柄出现代替可交互、Computer Use 会话观察代替落盘截图、一次任务管理器读数代替 CPU 时间窗、重启进程代替泄漏趋势、live sqlite3 回读代替纯文件静止采样。

## 12. 本轮仍不执行的事项

- 用户 BOT 五回合录制和 watcher 闭环。
- 游戏内记分板与解析结果逐项人工对照。
- 真实启动 CS2 和 `launch_demo_at_tick` 跳转。
- 正式 installer、签名、覆盖安装、卸载/重装。
- commit、push、GitHub Release、R2/D1、生产 updater。
- 新页面、新指标、新网络服务或 UI 改版。

用户现阶段无需操作。只有所有 Gate 通过，下一轮才另写一份面向用户的最短 BOT/CS2 验收清单。

## 13. 执行 AI 最终报告模板

最终报告写到本轮 evidence 的 `execution-report.md`，并在 `docs` 新增对应报告索引。格式固定：

```text
结果：AI 预验收未完成 / AI 预验收通过，可以请求用户 BOT 验收

候选身份：HEAD、diff fingerprint、EXE path/bytes/hash、identifier、PID/session
真实 Tauri：四个 window rect/client rect/DPI、页面、截图 path/hash、像素检查
性能：5 次 cold start、30 次 positions P95、viewer FPS、三状态 CPU raw/摘要
生命周期：20 次 WorkingSet/PrivateMemory/Handle raw、first/last/max/slope、mount/unmount
数据库：live 六次纯文件样本、WAL/SHM、诊断副本 integrity/version/jobs/leases
自动化：逐命令 exit code、tests、warning 归属、最终 build hash一致性
未执行：BOT、真实 tick、installer、commit/push/release/deploy
阻断项：仅列仍未关闭项；没有则写 none
证据：绝对路径
```

只有四尺寸、cold start、positions P95、FPS/CPU、20 次 lifecycle、DB 静止性、自动化和最终 EXE hash 全部一致通过，才可使用第二种结论。通过后下一负责人切换为用户，但用户只负责真实 BOT/CS2 行为验收；执行 AI仍负责整理数据、分析差异和修复代码。
