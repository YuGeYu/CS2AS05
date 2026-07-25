# 概览启动体验、目录推荐与 NadeAudit 移除实施方案

状态：待实际执行 AI 实施
方案日期：2026-07-25
当前基线：`main` / `031c157e5b9bf50338ffea713365ac72d0ffdc10`（0.5.3，尚未推送）
远端基线：`origin/main` / `13b721839494fb1bdeef9bdd70e8b3d20a4b732d`（0.5.2）
目标发布版本：`0.5.3`（继续补完尚未推送的 0.5.3，不升版）

## 1. 给实际执行 AI 的任务定义

在当前 `main` 直接继续开发，不新开分支，不回退 `031c157` 已完成的 0.5.3 Panel 融合。交付以下三项改动：

1. 取消 0.5.2 定制 NadeSystem 新增的 `[NadeAudit]` 游戏控制台日志，但保持投掷物硬上限、节奏、选择、计数、经济和失败回滚逻辑完全不变。
2. 把概览页“启动 CS2”做成更醒目的桌面主操作；点击后立即显示启动特效，直到检测到 `cs2.exe`、用户点击程序任意位置或 30 秒到期。
3. 在概览页“CS2 游戏目录”旁增加“猜你想选”，执行可取消的深度候选扫描；只在找到 3 个唯一候选、达到 10 秒或用户点击停止时结束，并让用户明确选择候选目录。

这是实际执行任务，不要再次只输出方案。完成后应构建并给出 `0.5.3` NSIS 安装程序。真实 CS2 游戏内验证由用户执行，实际执行 AI 负责提供最短、可复现的验证步骤和需要观察的结果。

当前 `main` 比 `origin/main` 领先 1 个尚未推送的 0.5.3 提交。本次仍发布 0.5.3：实施并验证后，将新改动合并进现有 0.5.3 发布提交（推荐使用非交互式 `git commit --amend`），保持 `origin/main..HEAD` 只有一个完整的 0.5.3 提交。最终只允许一次普通推送；推送前必须再次显示 `git status -sb` 和 `git log origin/main..HEAD --oneline`，禁止 force push。

## 2. 调查结论与不可误解的边界

### 2.1 控制台日志的准确范围

目标是删除 0.5.2 新增的 `[NadeAudit]` 输出链：

- `NadeSystem.cs` 三个成功提交路径调用 `WriteNadeAudit(...)`。
- `NadeSystem.cs` 内 `WriteNadeAudit` 构造记录并调用 `Server.PrintToConsole(record.Format())`。
- `NadeAudit.cs`、测试入口和文档提供的原始控制台日志分析工具。

不能删除以下内容：

- 上游原有 `[NadeSystem]` 加载、错误、命令用法和实体创建失败诊断。
- 助手写入 `%LOCALAPPDATA%\CS2人机增强助手\logs\runtime.log` 的安装/扫描诊断。
- BOT 模式的 `-console -condebug` 启动参数。它们属于上游 Panel v1.4.2 已冻结启动契约，不等同于 0.5.2 新增的插件审计输出。
- `NadePacingPolicy.Commit` 的计数提交本身；不能因为它曾返回审计快照而改变提交时机或顺序。

当前可确认的新增输出只有 `[NadeAudit]`。不要对所有 `Server.PrintToConsole` 做批量删除。若执行时认为还应删除其他行，必须先用固定上游源码或旧构建证明确属 0.5.2 新增，再单独列差分。

### 2.2 当前启动实现

- UI 位于 `src/views/OverviewView.vue`，当前是普通 `primary-button`。
- `usePanelStore.launch` 仅设置 `mutationKey`，没有启动体验状态，也没有把启动错误写入 `lastError`。
- Rust `services/panel.rs::launch_cs2` 先切模式，再启动 `steam.exe -applaunch 730`；BOT 模式附加 `-insecure -console -condebug`。
- 全局进程轮询周期为 10 秒，Panel 聚合快照为可见时 2 秒一次。仅依靠二者会让启动特效消失不够及时。

### 2.3 当前目录扫描不足

`services/cs2.rs::discover_cs2_roots` 目前只检查每个盘符的：

```text
<drive>:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive
```

以及该位置 `libraryfolders.vdf` 中按引号拆行得到的路径。现状缺少：

- Windows 注册表中的 Steam 位置。
- 正在运行的 `steam.exe` 所在目录。
- 32/64 位注册表视图与 App 730 卸载项。
- 非默认 Steam 根目录、迁移盘和多个 `steamapps`。
- `appmanifest_730.acf` 的 `installdir`。
- 常见历史目录布局与有限深度兜底搜索。
- 结构化 VDF/ACF 解析、进度事件、10 秒截止、3 个候选截止和手动取消。

原有快速扫描可继续用于应用启动；“猜你想选”必须走新的扩展扫描，不能让启动自动扫描阻塞 10 秒。

## 3. 总体实现结构

建议新增和调整：

```text
src-tauri/src/
  commands/cs2.rs                     # 新增 guess/stop commands
  models/cs2.rs                       # 新增候选、事件、摘要、结束原因 DTO
  services/
    cs2.rs                            # 保留现有安装、检查和快速发现
    cs2_discovery.rs                  # 新的可取消扫描器和 Steam/VDF 解析
  lib.rs                              # manage ScanCoordinator + 注册 commands

src/
  components/
    LaunchExperience.vue              # 启动特效覆盖层
    Cs2RootSuggestionsDialog.vue      # 候选扫描与选择对话框
  composables/
    useCs2LaunchExperience.ts         # 启动状态机、探测、计时和清理
  services/tauri/cs2.ts               # Channel 扫描和停止接口
  stores/cs2.ts                       # 推荐扫描状态、候选合并、选择动作
  views/OverviewView.vue              # 两个新入口及华丽启动带
  styles/main.css                     # 对应双主题、响应式和 reduced-motion

tests/
  overview-view.spec.ts
  launch-experience.spec.ts
  cs2-root-suggestions.spec.ts
```

Rust 的扫描器单独成文件，避免继续扩大当前已包含安装、ZIP、进程与日志逻辑的 `services/cs2.rs`。

## 4. 移除 `[NadeAudit]` 的实施步骤

### 4.1 源码改动

在 `third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/` 内执行：

1. 三处 `var audit = _pacingPolicy.Commit(...)` 改为直接调用 `_pacingPolicy.Commit(...)`，保留调用相对扣钱、冷却、队伍计数的原顺序。
2. 删除三处 `WriteNadeAudit(...)` 调用。
3. 删除 `NadeSystem.cs::WriteNadeAudit` 方法。
4. 删除 `NadeAudit.cs` 和 `tests/Program.cs` 中日志解析/摘要输出入口、round-trip/parser 专用测试；保留纯策略、硬上限、节奏和提交时机测试。
5. `NadePacingPolicy` 内用于策略测试的状态快照可以保留，不要为清理命名而重构核心算法。
6. 更新 `NadeSystem.csproj` 和测试 csproj，确保生产构建及测试都不再编译 `NadeAudit.cs`。
7. 从固定上游 `97fd57d2` 重新生成 `nades-per-bot-round-limit.patch`，不要手工留下与源码不一致的旧 patch。
8. 更新 `UPSTREAM.txt`：把“0.5.2 balance and audit customization”改成只描述 balance；删除日志分析命令，并明确“不再生成 `[NadeAudit]` 控制台行”。

不要删除 `_pacingPolicy.Commit` 的返回结构，除非编译器要求且单元测试证明完全等价。最小改动优先，目标是取消输出，不是重新设计投掷策略。

### 4.2 编译与策略测试

在该目录运行：

```powershell
dotnet restore .\NadeSystem.csproj
dotnet build .\NadeSystem.csproj -c Release
dotnet run --project .\tests\NadePacingPolicy.Tests.csproj -c Release
```

增加静态/二进制断言：

- `NadeSystem.cs` 不含 `WriteNadeAudit` 和 `NadeAuditRecord`。
- 新 DLL 的 ASCII 与 UTF-16 字符串均不含 `[NadeAudit]`。
- DLL 仍引用/保留上游必要的 `[NadeSystem]` 错误诊断，不以“零 PrintToConsole”为通过标准。
- 原有策略断言数量可变化，但硬上限、开局预算、BOT/队伍间隔、烟雾上限、失败取消与提交后计数的断言必须全部保留。

### 4.3 替换内置 ZIP

只替换：

```text
addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll
```

推荐新增 `scripts/replace-nadesystem.ps1`，使用 `System.IO.Compression.ZipArchive` 在临时副本上按 entry 精确替换，成功验证后再原子移动到 `src-tauri/resources/CS2BotImprover.zip`。不能直接覆盖原 ZIP 后再验证。

替换前后为 ZIP 内全部 entry 建立 `{name,size,sha256}` 清单；断言除目标 DLL 外所有 entry 的内容 SHA256 完全一致，Panel EXE 仍满足当前大小和摘要。然后更新：

- `src-tauri/src/services/cs2.rs::CUSTOM_ZIP_SHA256`。
- Rust 测试中的定制 `NadeSystem.dll` SHA256。
- `README.md` 的 ZIP SHA256 和功能说明，删除 `[NadeAudit]` bullet。
- `tests/fixtures/panel-v1.4.2/manifest.json` 的 bundle SHA256。
- `NOTICE.md` 中“审计工具”表述；如果审计工具已经删除，不可继续声称随附。
- `UPSTREAM.txt` 的新 DLL SHA256、ZIP SHA256和变更说明。

ZIP 大小变化不是失败条件，非目标 entry 内容变化才是失败。

### 4.4 用户实机验收步骤

实际执行 AI 最终请用户：

1. 备份并安装新包，BOT 模式启动 CS2。
2. 进行能触发计划投掷、特殊投掷和反击投掷的回合。
3. 在游戏控制台和 `game/csgo/console.log` 搜索 `[NadeAudit]`，应为 0 行。
4. 确认上游加载/错误日志仍可见，且道具硬上限和节奏没有明显失效。

程序构建验证不能替代这一步，但用户未执行实机步骤不应伪报“游戏内已确认”。

## 5. 启动按钮与启动特效

### 5.1 UI 视觉

UI/UX Pro Max 建议强调进度反馈、控制运动数量并尊重 reduced motion。结合当前产品，不采用其外部字体或营销式 hero，而采用现有双主题和紧凑工具布局：

- `launch-band` 升级为概览页最强视觉层级，但不做超大 hero、不嵌套卡片。
- 启动按钮固定最小宽度约 190px、高度 52px，使用 `Play` 或 `Rocket` Lucide 图标、清晰边框、主色阴影和按压反馈。
- 按钮静止时不持续闪烁；hover 只做 160-220ms 的位移/阴影变化。
- 启动特效覆盖整个应用内容区并可接收点击，标题栏可保持可见。中心使用一个稳定尺寸的 CS2 启动标记、最多两个运动层（分段环和扫描线）及一条 30 秒时间进度条。
- 文案只显示状态，例如“正在启动 Counter-Strike 2”和当前在线/BOT 模式，不显示功能说明或教程文字。
- 不使用外部图片、网络字体、渐变光球或大量粒子；沿用现有 `--primary`、`--success`、`--warning`、surface tokens，并同时兼容 light/dark。
- `prefers-reduced-motion: reduce` 下停止旋转、扫描和缩放，只保留静态状态、进度与相同的三个消失条件。

### 5.2 状态机

`useCs2LaunchExperience` 使用明确状态：

```text
idle -> requesting -> waitingForProcess -> idle
```

启动流程：

1. 用户点击按钮后立即进入 `requesting` 并显示特效，30 秒总计时从这一刻开始。
2. 调用现有 `panel.launch(root, mode)`。
3. command 成功返回只代表 Steam 进程已接受启动请求，进入 `waitingForProcess`，不能当作 CS2 已启动。
4. 使用现有 `check_cs2_process` 进行 500-750ms 的串行非重叠探测；上一次完成后再安排下一次，不能 `setInterval` 堆积 invoke。
5. 探测到 `cs2.exe` 后立即更新 `cs2ProcessState='running'`，在不超过 200ms 的退出过渡后销毁特效。
6. 用户在程序任意位置点击时销毁特效，但不终止 Steam/CS2 启动。为避免最初的启动按钮 click 立即触发销毁，点击监听应在该事件完成后的 microtask/下一帧注册。
7. 30 秒到期销毁特效；这不是“启动失败”，只恢复普通 UI，之后全局轮询仍可检测 CS2。
8. 启动 command 明确失败时立即销毁特效，并通过 `panel.lastError`/结构化 Toast 显示错误。这是异常退出，不需要等 30 秒。
9. 组件卸载、窗口刷新、根目录变化或再次开始启动前，统一清理点击监听、timeout 和探测任务。

同一时间只允许一个启动状态机。特效被用户点击关闭后按钮仍保持 `mutationKey === 'launch'` 的真实禁用状态，直到 command 返回；不能因为视觉关闭就允许重复启动。

### 5.3 交互与可访问性

- 特效容器使用 `role="status"`、`aria-live="polite"`，不冒充确认对话框。
- 覆盖层点击区域需要明确 `cursor: pointer`，并提供可见的 `X` 图标按钮作为确定点击目标；点击背景同样关闭。
- 运动元素 `aria-hidden="true"`。
- 固定特效尺寸使用 `min/max/clamp` 约束，但字体大小不按 viewport 缩放。
- 720x620 最小窗口下按钮、文案、特效和退出按钮不能重叠；长模式文案允许换行。

### 5.4 Store 修正

调整 `usePanelStore.launch`：

- 清空旧 `lastError`。
- 捕获并规范化启动错误后重新抛出，行为与其他 mutate 一致。
- `finally` 只清理 `mutationKey`，不控制特效生命周期。

在 `useCs2Store` 增加受控的启动探测方法，或让 composable 调用 `refreshProcessStatus` 并读取 store；不要在组件里直接篡改私有 ref。

### 5.5 标题标识与版本文字

图中绿色方框对应 `src/components/AppTitlebar.vue` 的 `.titlebar-mark`，红色方框对应 `src/components/AppShell.vue` 侧栏品牌的 `<small>`。执行时不要改窗口拖动和控制按钮。

- 从已有的 `src-tauri/icons/icon.png` 生成或复制一份前端可打包资源 `src/assets/app-icon.png`，保持与 Tauri 程序图标的像素内容和色彩一致。不要用文字 `CS2` 或新画的伪图标替代。
- `AppTitlebar.vue` 改为导入图片并显示固定宽高、`object-fit: contain`、圆角和替代文本；标题栏整体仍保持 `aria-hidden` 的品牌装饰，产品名文字仍保留。
- `AppShell.vue` 将 `Panel 0.5.3` 改为只显示 `0.5.3`，不要把 `Panel` 前缀留在侧栏。`InstallView.vue` 的资源包标题和按钮文案不因此需要改动。
- 图标需同时在 light/dark 主题、窗口 720x620 下不变形、不挤压标题和拖动区域。补充一个 Vitest 断言与一张桌面截图验证，不要只检查文件存在。

### 5.6 BOT 启动前插件版本门禁

目标是让 BOT 启动具备“当前或更高版本的本项目插件才继续”的保证，不是让“安装与诊断”页的安装按钮变成版本检查按钮。版本真值必须来自 Rust 包的 `env!("CARGO_PKG_VERSION")`，不要在 TypeScript 或启动函数里硬编码 `0.5.3`。

#### 版本元数据与可信校验

1. 在定制资源包中增加应用自有的标记文件，建议路径为 `game/csgo/addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json`。该 JSON 不是 CounterStrikeSharp 插件清单，仅是助手的安装标记，不能被游戏执行。
2. 标记至少包含 `schema`, `product`, `pluginId`, `version`, `payloadSha256`, `generatedFrom` 字段。`version` 由当次构建版本生成，`payloadSha256` 对所有应用所有的 DLL/config/data entry 按稳定路径、字节长度、内容排序计算，排除标记自身，避免循环。
3. 将标记写入 ZIP，同时在 `src-tauri` 测试 fixture 中保存其文件摘要。更新包体 SHA256 时必须验证除新标记和已预期的定制 DLL 外，其余 entry 不被无意改写。
4. 对已安装目录读取标记，校验 `product/pluginId/schema/version/payloadSha256`、必需文件和文件摘要。缺失、JSON 坏损、不匹配摘要、不支持的版本格式都是“不可信”，不能仅凭一个文件名判断已安装。

#### 比较和启动顺序

- 新增 `inspect_bot_plugin_version(root)` 和 `ensure_bot_plugin_current(app, root)` 内部服务，并使用与执行相同的版本比较逻辑。可使用固定版本的 `semver` crate，或对 `major.minor.patch` 做严格数字解析；不得用字符串排序。
- 为了支持测试版，比较政策应按数字核心版本 `major.minor.patch` 判断：安装版本核心版本 `>=` 当前程序核心版本即可通过，允许同核心版本的 `-alpha/-beta/-test.N` 标记；版本前缀、构建元数据和非数字尾部不得让低版本逃过。若团队决定遵循完整 SemVer，必须在测试版命名规则中使用高于当前的数字核心，不要把 `0.5.3-beta` 误当成高于 `0.5.3` 。
- 修改 `src-tauri/src/commands/panel.rs::launch_panel_cs2` 接收 `AppHandle`，传入 `services/panel.rs::launch_cs2`。只有 `mode == "bots"` 才执行版本门禁；`online` 路径保持现有逻辑。
- BOT 启动顺序必须是：检测 `cs2.exe` 未运行 -> 读取并校验已安装标记 -> 若当前或更高且 payload 有效则跳过安装 -> 否则校验内置 ZIP 并原子安装当前包 -> 重新检测进程 -> 调用现有 `set_mode` -> 启动 Steam/CS2。安装失败时不应改写模式，也不应 spawn Steam。
- “原子安装”要求在解压到同目录的临时目录中先完成校验，保留本次安装前由助手管理的文件备份，写入标记最后一步。任一失败都要回滚到安装前状态；不能先删除 `addons` 再冒险解压。复用现有 `install_bot_package` 的摘要校验和路径安全检查，但不要让它变成版本门禁。
- 初始启动命令应返回 `pluginAction: "unchanged" | "installed"`、`pluginVersion` 和原有 `options/insecure`，供启动特效和 Toast 展示可追踪状态。异常使用稳定错误码，例如 `[BOT_PLUGIN_VERSION_INVALID]`、`[BOT_PLUGIN_AUTO_INSTALL_FAILED]`、`[BOT_PLUGIN_PAYLOAD_INVALID]`。
- 不得调用浏览器文件 API、不得启动游戏后再修改插件，不得向游戏控制台写入版本检查日志。所有诊断写现有 `runtime.log`，不影响游戏运行。

#### “安装与诊断”页的不变边界

- `src/views/InstallView.vue` 的 `store.install()`、“安装定制资源包”按钮和手动安装提示保持原样；不先调用版本检查，不因已有高版本而跳过。
- `install_bot_package` IPC 仍是显式手动安装命令；可以与 BOT 自动安装共享内部原子写入辅助函数，但不能改变手动命令的版本决策、文案和按钮禁用条件。

## 6. “猜你想选”扫描架构

### 6.1 IPC 契约

使用 Tauri 2 IPC `Channel` 传递进度和候选，不使用浏览器目录 API、`navigator.*` 或前端递归文件系统。

建议命令：

```text
guess_cs2_roots(onEvent: Channel<Cs2RootScanEvent>) -> Cs2RootScanSummary
stop_guess_cs2_roots() -> bool
```

`guess_cs2_roots` 是 async command，内部通过 `tauri::async_runtime::spawn_blocking` 执行磁盘/注册表扫描，不能阻塞 Tauri 主线程。`ScanCoordinator` 由 `Builder.manage(...)` 注册，内部只允许一个活动任务，并用 `Arc<AtomicBool>` 或等价 token 取消。第二次开始扫描时返回稳定错误 `[CS2_SCAN_ALREADY_RUNNING]`，前端聚焦已有对话框。

DTO 建议：

```ts
type Cs2SuggestedRoot = {
  path: string
  source: string
  confidence: 'verified' | 'likely'
  evidence: string[]
}

type Cs2RootScanEvent = {
  kind: 'progress' | 'candidate'
  elapsedMs: number
  checkedLocations: number
  currentLocation?: string
  candidate?: Cs2SuggestedRoot
}

type Cs2RootScanSummary = {
  candidates: Cs2SuggestedRoot[]
  elapsedMs: number
  checkedLocations: number
  stopReason: 'threeFound' | 'timeout' | 'userStopped'
  warnings: string[]
}
```

普通单路径权限/解析错误放入 `warnings` 并继续，不以第一个错误停止。只有扫描任务本身无法建立时 command 才失败。

### 6.2 严格停止语义

使用 `std::time::Instant` 计算 10 秒，不用系统墙钟。每次准备访问新路径、完成一次文件读取以及发 event 前都检查：

1. 已有 3 个 canonical 后唯一候选 -> `threeFound`。
2. `elapsed >= 10s` -> `timeout`。
3. cancel token 为 true -> `userStopped`。

不能因为找到第 1 个候选、某个 Steam 配置读完或某个盘不存在而提前返回。已知来源耗尽但未到 10 秒时，继续处理有界兜底队列；若队列确实为空，则做可取消的短等待直到 10 秒，保证业务结束原因仍是三个规定条件之一。

候选按 Windows 大小写不敏感的 canonical path 去重。符号链接/junction 不重复计数。发送 candidate event 前再次检查上限，结果永远不超过 3 个。

### 6.3 扫描顺序

按高命中、低成本到低命中、有界成本执行：

1. 当前已选目录及现有快速扫描候选，用于去重和证据补全，不自动选中。
2. Windows 注册表 32/64 位视图：
   - `HKCU\Software\Valve\Steam` 的 `SteamPath`/`SteamExe`。
   - `HKLM\Software\Valve\Steam` 和 Wow6432Node 的 `InstallPath`。
   - HKLM/HKCU 卸载项 `Steam App 730` 的 `InstallLocation`（存在时）。
3. 正在运行的 `steam.exe` 可访问的 executable path，并从其父目录找 `steamapps`。
4. 每个已知 Steam 根下用结构化 VDF parser 读取 `steamapps/libraryfolders.vdf`；优先包含 App 730 的库。
5. 每个库读取 `steamapps/appmanifest_730.acf` 的 `installdir`，拼接 `steamapps/common/<installdir>`。
6. 仅对 Windows 固定本地盘检查常见布局：
   - `SteamLibrary`、`Steam`、`Program Files (x86)/Steam`、`Program Files/Steam`。
   - `Games/SteamLibrary`、`Games/Steam`、`Steam Games` 等历史常见根。
7. 最后在固定盘根的上述游戏/Steam 容器内做有限深度、有限目录数搜索；跳过 `Windows`、`ProgramData`、用户 profile、回收站、系统卷信息、node_modules、junction 和网络/可移动盘。禁止全盘无界递归。

不要继续使用 `split('"').nth(3)` 解析 Valve 文件。增加一个与 Rust 1.77.2 兼容并固定版本的 KeyValues/VDF parser；`libraryfolders.vdf` 和 `appmanifest_730.acf` 共用同一解析模块，并为旧版/新版 VDF 结构建立 fixture。

### 6.4 候选验证与排序

推荐目录最终统一为 `Counter-Strike Global Offensive` 根目录。接受输入变体后复用 `normalize_root`，但候选验证至少需要：

- `game/csgo/gameinfo.gi` 是文件。
- `game/csgo` 是目录。

置信度：

- `verified`：同时存在 `game/bin/win64/cs2.exe`，或 App 730 manifest 明确指向该目录。
- `likely`：基础目录和 `gameinfo.gi` 有效，但未找到 executable/manifest 证据。

排序先 `verified`，再按来源优先级，最后按 canonical path 稳定排序。不要把仅名称相似、只有旧 CS:GO 文件或只有空 `game/csgo` 的目录提供给用户。

### 6.5 前端交互

在 `OverviewView.vue` 的目录带右侧放两个并列命令：

- `猜你想选`：`ScanSearch`/`WandSparkles` Lucide 图标 + 文字。
- `选择目录`：保留现有手动目录对话框。

点击“猜你想选”打开应用内 `Cs2RootSuggestionsDialog`：

- 扫描中显示 0-10 秒确定时间进度、已检查位置数、最多 3 个候选列表。
- 扫描中主命令为带 `Square` 图标的“停止扫描”；点击后立刻禁用按钮并调用 stop command，等待 summary 收敛。
- 候选每行显示路径、来源、置信度和选择按钮；列表行不是嵌套卡片。
- 用户选中候选时先停止仍在运行的扫描，再调用现有 `cs2.selectRoot(path)` 和 `panel.refresh(path)`，成功后关闭对话框。
- 用户关闭对话框等同点击停止，不能留下后台扫描。
- 达到 3 个自动停止；10 秒到期显示“扫描完成”及实际候选数。0 个候选时保留手动“选择目录”入口。
- 不自动选择第一个候选，也不覆盖 localStorage 中当前根目录。
- 扫描候选合并进 `cs2.candidates` 时按 canonical Windows path 去重，不能只用区分大小写字符串比较。

窄屏下按钮换行但都保持 44px 目标；候选长路径必须省略显示并能通过 title/可选择文本查看完整值。

## 7. 文件级修改清单

### 必改

- `third_party/.../NadeSystem.cs`、`NadeAudit.cs`、两个测试项目、`UPSTREAM.txt`、patch。
- `src-tauri/resources/CS2BotImprover.zip`。
- `src-tauri/src/services/cs2.rs`：共享规范化、摘要常量和 DLL 测试；保留快速扫描。
- `src-tauri/src/commands/cs2.rs`、`models/cs2.rs`、`services/mod.rs`、`lib.rs`。
- `src/views/OverviewView.vue`、`src/stores/cs2.ts`、`src/stores/panel.ts`、`src/services/tauri/cs2.ts`、`src/styles/main.css`。
- `README.md`、`NOTICE.md`、`tests/fixtures/panel-v1.4.2/manifest.json`。
- 项目版本文件：核对 `package.json`、lock 顶层、`Cargo.toml` 当前 package、`tauri.conf.json` 和产品文案均保持 0.5.3；不得升版。

### 新增

- `src-tauri/src/services/cs2_discovery.rs`。
- `src/components/LaunchExperience.vue`。
- `src/components/Cs2RootSuggestionsDialog.vue`。
- `src/composables/useCs2LaunchExperience.ts`。
- 对应 Vue/Vitest 和 Rust fixture/unit tests。
- `scripts/replace-nadesystem.ps1` 与 ZIP entry manifest 比较工具。

### 不改

- Panel 模式、难度、Aim/Nades、Bot items、刀具和命令契约。
- 原版 Panel EXE 及其摘要。
- NadeSystem 投掷策略参数和提交语义。
- 软件更新服务端或其他不相关页面。

## 8. 自动化测试要求

### 8.1 Rust 扫描测试

通过依赖注入或临时目录 fixture 测试：

- 新旧两种 `libraryfolders.vdf`。
- `appmanifest_730.acf` 自定义 `installdir`。
- 多个 Steam root/library，跨盘迁移。
- 同一路径不同大小写、`game`/`game/csgo` 变体和 junction 去重。
- 第 3 个候选出现后不再访问后续路径。
- 虚拟时钟达到 10 秒返回 `timeout`，不要让测试真实等待 10 秒。
- stop token 返回 `userStopped`。
- 单个解析/权限错误记 warning 并继续。
- 同时开始第二个扫描返回 already-running。
- 不接受只有相似目录名的假阳性。

把时钟、候选源和 filesystem probe 抽象成可测试接口，避免单元测试访问真实 C/D/E 盘。

### 8.2 启动体验测试

Vitest 使用 fake timers 和 mock Tauri invoke：

- 点击后同步出现特效，不等待 launch promise。
- launch 成功但进程仍 false 时继续显示。
- 后续探测 true 后消失并清理所有 timer/listener。
- 覆盖层任意点击消失，且不会取消 launch promise。
- 初始启动按钮 click 不会立即把新特效关闭。
- 30,000ms 到期消失，29,999ms 仍显示。
- launch reject 立即消失并显示结构化错误。
- 重复点击被阻止。
- unmount/root change 完整清理。
- reduced-motion 下无动画 class/关键动画被 CSS 媒体查询禁用，但状态机相同。

### 8.3 推荐目录 UI 测试

- 事件流逐个加入候选并显示进度。
- 最多显示 3 个且 canonical 去重。
- 停止、关闭对话框、选择候选都会调用 stop；只调用一次。
- 选中后顺序为 stop -> `selectRoot` -> `panel.refresh` -> close。
- 0 候选和扫描 warning 有明确状态，手动选择仍可用。
- 720x620 下长路径不挤出按钮。

## 9. 验收矩阵

| 场景 | 必须结果 |
| --- | --- |
| 点击启动 | 同一帧/下一次 Vue commit 显示特效，按钮不重复提交 |
| Steam 接受但 CS2 未起 | 特效持续，非重叠探测继续 |
| 检测到 cs2.exe | 特效立即退出，状态条显示运行中 |
| 用户任意点击 | 特效退出，CS2 启动不被取消 |
| 30 秒 | 特效退出，不误报失败 |
| 启动错误 | 立即退出，显示可执行错误 |
| reduced motion | 无旋转/扫描运动，三个退出条件不变 |
| 猜你想选找到 1 个 | 不提前停止，继续到 3 个、10 秒或用户停止 |
| 找到 3 个 | 不返回第 4 个，stopReason 为 threeFound |
| 10 秒 | 不继续磁盘访问，stopReason 为 timeout |
| 用户停止/关闭 | 尽快停止，保留已发现候选 |
| 多 Steam/迁移盘 | 注册表、VDF、manifest 候选合并且去重 |
| 假历史目录 | 缺 gameinfo.gi 时不展示 |
| 新插件 DLL | 无 `[NadeAudit]` 字符串，策略测试全过 |
| ZIP | 只有 NadeSystem.dll entry 内容改变 |
| 游戏控制台 | 用户实机确认无 `[NadeAudit]`，其他诊断仍可用 |

## 10. 构建、打包与发布

依次运行：

```powershell
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.2\nades-per-bot-round-limit\tests\NadePacingPolicy.Tests.csproj -c Release
npm install
npm run verify
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

版本号必须保持 `0.5.3`。执行前核对项目自身的版本字段；已是 0.5.3 的不做无意义改写，也不得全局替换版本字符串，以免误改 Cargo.lock 第三方版本。构建打包最多允许 5 次，先跑分层检查再启动完整 bundle，避免用安装包构建充当编译诊断。

UI 实现后启动 `npm run dev:desktop`，验证 960x700、720x620、1280x800 的 light/dark 和 reduced-motion。不要用普通浏览器文件 API替代 Tauri 功能，也不能只凭截图宣称扫描/启动完成。

安装包交付必须报告：

- 绝对路径、字节数、SHA256、生成时间。
- 干净安装与从已发布 0.5.2 覆盖升级的结果。
- 持续启动 30 秒、页面切换和推荐扫描停止条件验证。
- 新 ZIP、新 NadeSystem DLL 摘要及 entry 差分报告路径。
- 自动测试结果和仍待用户执行的真实 CS2 控制台验证。

提交建议：

```text
Release 0.5.3 with integrated Panel and overview enhancements
```

在将本次改动 amend 进现有 0.5.3 提交并完成验证后，仅执行一次普通 `git push origin main`。若网络失败，不立即重复超过 1 次；保留本地 commit 和安装包并报告。

## 11. 回退与失败处理

- 新扫描器失败：保留原有快速扫描和手动选择，隐藏“猜你想选”，不破坏目录选择主流程。
- 启动特效异常：回退新组件/样式，保留原 `panel.launch`，不要回退已验证的 Rust 启动逻辑。
- NadeSystem 新 DLL 策略测试失败：恢复替换前 ZIP 和 DLL，不发布；不能为了去日志牺牲策略断言。
- ZIP 非目标 entry 发生变化：从备份重新执行结构化替换，禁止接受无法解释的资源变化。
- 实机仍出现 `[NadeAudit]`：先核对实际加载 DLL 的 SHA256 和安装目录，避免把旧插件残留误判为新构建失败。
- 所有回退只处理本次新建/修改文件，不重置 0.5.3 提交、用户配置或未知工作区内容。

## 12. 最终交付报告模板

```text
结果：0.5.3 是否完成
NadeAudit：源码/DLL/ZIP 静态验证结果；用户实机结果或待验证
启动体验：三个消失条件及错误/reduced-motion 结果
目录推荐：扫描来源、3 个/10 秒/手动停止结果
测试：dotnet、npm verify、fmt、clippy、cargo test
资源：ZIP 与 DLL 新 SHA256、entry 差分报告
UI：三个尺寸 light/dark/reduced-motion 截图路径
安装包：绝对路径、大小、SHA256、时间
安装验证：干净安装、从 0.5.2 覆盖升级、持续启动
Git：本地提交、单次 push 结果
限制：需要用户执行的真实 CS2 验证
```

只有自动化结果、资源摘要和安装包验证完整后才可报告程序开发完成；游戏内日志与效果只按用户实际反馈报告。
