# CS2 客户端故障、强制关闭与备份策略交接方案

日期：2026-08-28  
工作区：`E:\CS2AS05`  
角色：方案制定 AI 交给下一位实际执行 AI  
状态：仅调查和方案，业务代码尚未按本文实施

## 1. 目标与完成标准

本轮只处理四件互相关联但必须分别验收的事情：

1. 通过侧边栏 Codex 内置浏览器调查并记录两个客户端故障的可复现证据；修复导致 CS2 状态错误、界面不响应或状态刷新竞态的客户端问题。
2. 在助手中新增“关闭 CS2”按钮。按钮只针对明确识别出的 `cs2.exe` 进程，支持正常关闭失败后的受控强制关闭，并在最终状态确认后才提示成功。
3. 修复 CS2 未启动、Steam 未启动时仍显示“CS2 运行中”的误判。Steam 是否运行不能作为 CS2 运行依据。
4. 将助手主动产生的写前 `.backup-*` 文件改为默认不保留；只有用户在当前操作前明确勾选“本次操作保留备份”时，才保留本次生成的备份。助手启动、写入成功和操作结束时清理其余助手拥有的旧备份。

完成条件：自动化测试、受控进程测试、真实 Tauri 窗口截图和用户电脑上“CS2/Steam 均关闭”回读均通过。只完成源码或单元测试不能声明修复完成。

## 2. 已调查的真实基线

### 2.0 内置浏览器现场核对（2026-08-28）

已在侧边栏 Codex 内置浏览器打开 `https://cs2as.600318.xyz/mm-259`，页面为“管理后台 · CS2人机增强助手”，可见两张相关工单：

- 工单 `CS2-MTBB874X-A246`（v0.5.9）描述“cs没有打开，甚至steam都没打开，但是一直显示cs2运行中”。其脱敏诊断摘要却写着“游戏运行中：是”，日志只显示扫描到 0 个候选目录及目录检查，未提供 PID、进程路径或状态采集时间。
- 工单 `CS2-MTB1JT80-7E51`（v0.5.9）描述大量 backup 文件；摘要写着“游戏运行中：否”，并包含多次安装/卸载/启动 Panel 记录，说明备份文件是在实际写入流程中产生的，不能通过删除整个 `backup` 目录解决。

这两个网页工单是脱敏、可能滞后的客户端上报，不是当前电脑实时状态。但第一张工单已经确认一个必须修复的产品故障现象：用户实际观察到 CS2 和 Steam 都未运行，而助手仍对外显示“CS2 运行中”。工单不能单独确认具体根因是旧状态缓存、进程枚举错误还是采样竞态；因此修复目标不是改文案，而是让状态来源、采集时间和最终回读一致。方案后续的结构化快照、精确 PID/映像匹配、双采样、错误时清空旧 running 状态和关闭后的最终确认，正是针对该已确认误判现象的修复链路。浏览器页面本身没有能力验证 Windows 进程，也没有证明“关闭按钮”已存在，因此最终门禁仍必须在真实 Tauri/Windows 环境完成。

### 2.1 进程状态

- `src-tauri/src/services/cs2.rs` 的 `check_cs2_process()` 在 Windows 使用 ToolHelp `Process32FirstW/Process32NextW`，当前仅按可执行文件名匹配 `cs2`/`cs2.exe`，不返回 PID、路径或启动时间。
- `src-tauri/src/commands/cs2.rs` 只暴露 `check_cs2_process`、安装、卸载、目录检查等命令，没有关闭 CS2 命令。
- `src/services/tauri/cs2.ts` 和 `src/stores/cs2.ts` 只把布尔值映射成 `running/stopped/unknown`；`AppShell.vue` 每 2 秒刷新 Panel，退出 CS2 时可能与 Panel/Demo I/O 叠加。
- `src/components/StatusStrip.vue` 目前只显示状态，没有关闭按钮。
- `docs/assistant-unresponsive-after-cs2-exit-20260816.md` 记录了退出瞬间轮询、`initialize_panel_defaults`、`get_panel_snapshot` 和 Demo watcher 并发的风险，但没有已证实的死锁证据。因此执行 AI 必须先加耗时/开始结束日志再修改并发策略。

### 2.2 备份

- `src-tauri/src/services/panel.rs` 的 `atomic_write()` 对每一个已存在目标文件创建 `目标扩展名.backup-时间戳`，失败时回滚，成功后不删除；该函数被 Panel 配置、`gameinfo.gi`、覆盖文件等多个写路径调用。
- `src-tauri/src/services/cs2.rs` 另有安装事务目录和 `backup/Online`、`backup/WithBots` 运行时文件。这些是插件功能所需的受管文件，不得误删；本策略只针对助手生成的带明确命名模式的写前备份。
- 当前测试已检查 backup 目录和写前备份行为，执行 AI 必须更新断言而不是删除安全回滚能力。

### 2.3 内置浏览器的两个客户端故障调查边界

内置浏览器只能验证网页/UI 请求、渲染、导航和客户端接口返回，不能把网页状态当作真实 Tauri/Windows 进程证据。执行 AI 应分别记录：

- 故障 A：页面显示 CS2 运行中但任务管理器/PowerShell 无 `cs2.exe`；记录页面状态、同一时刻的 `check_cs2_process` 返回值、结构化进程快照（PID/映像路径/采集时间/采样次数）和 runtime.log。若只有历史工单摘要而没有同刻快照，结论只能写“待复现”，不能写“误判已证实”。
- 故障 B：CS2 退出后页面无响应或状态不刷新；记录退出前后的命令开始/结束日志、前端 heartbeat、窗口是否仍响应和 WebView 控制台错误。不能用浏览器 `window.close`、下载、File System API 或网页按钮代替 Tauri 命令验收。

## 3. 后端实施方案

### 3.1 统一、可审计的进程快照

在 `src-tauri/src/services/cs2.rs` 抽出 `Cs2ProcessInfo { pid, exe_path, parent_pid, start_time }` 和 `list_cs2_processes()`。Windows 首选 ToolHelp 枚举 PID，再使用只读进程查询取得映像路径；查询权限不足时保留 PID 并记录 `[CS2_PROCESS_PATH_UNAVAILABLE]`，不能因此把 Steam 或助手判成 CS2。名称匹配必须是精确、不区分大小写的 `cs2.exe`，排除 `.bak`、命令行字符串和 `steam.exe`。

`check_cs2_process()` 只调用该统一列表并返回 `!is_empty()`。增加 `get_cs2_process_snapshot`（或将现有检查返回结构化状态，需同步 TS DTO），包含 `observedAt`, `processes`, `confidence`。连续两次快照间隔约 150-300ms，只有两次均为空才发布 stopped；这样避免进程创建/退出瞬间的半状态。查询错误发布 `unknown`，绝不保留旧的 `running` 假象。

### 3.2 关闭命令

在 `src-tauri/src/commands/cs2.rs` 增加 `close_cs2(force: bool) -> OperationResult`，在 `src-tauri/src/lib.rs` 注册，并在 `src/services/tauri/cs2.ts`、`src/stores/cs2.ts` 暴露。

执行顺序：

1. 获取带 PID/路径的快照；为空则再次确认并返回“CS2 已关闭”，不触碰 Steam。
2. 默认 `force=false`：对每个精确 CS2 PID 发送 Windows `WM_CLOSE`/等价优雅关闭信号，等待最多 8 秒，每 250ms 重查快照。
3. 仍存在时，前端必须先显示一次明确确认（“CS2 未响应，强制关闭可能丢失未保存内容”）；用户确认后再次调用 `close_cs2(true)`。后端只允许对快照中已验证的 CS2 PID 使用 `TerminateProcess`，必要时按父子关系结束该 CS2 进程树；禁止 `taskkill /IM *`、禁止结束 `steam.exe`、助手自身或未知 PID。
4. 强制结束后连续两次快照为空才返回成功；失败返回 PID、阶段和错误码，保留诊断日志。

无论成功或失败都调用一次轻量 `refresh_process_status`，并用命令序列号/取消令牌防止关闭期间旧轮询覆盖新状态。所有开始、发送信号、等待、最终快照写入 `runtime.log`，但不记录用户凭据。

### 3.3 退出竞态与响应性

在 `AppShell.vue`/相关 store 中加入单一进程刷新协调器：关闭命令、定时轮询、窗口恢复只共享一个 in-flight Promise；CS2 从 running 变 stopped 后，Panel 刷新延迟 500-1000ms 且取消上一次未开始的刷新，Demo 扫描使用现有去重机制。不要在前端设置无限超时；Rust command 对文件/进程等待提供明确超时并返回可诊断错误。为每个 Tauri command 记录耗时，先用日志证明故障 B 的阻塞点再决定是否调整锁粒度。

## 4. 前端实施方案

### 4.1 关闭按钮

在 `src/components/StatusStrip.vue` 或其同层全局状态工具区加入图标按钮（Lucide `Power`/`CircleStop`），文字“关闭 CS2”，最小点击区 44px，`aria-label`、`title`、可见 focus 和 `aria-live` 结果齐全。

- `checking/unknown`：按钮禁用并显示检测中/无法确认，不允许盲杀。
- `stopped`：显示“CS2 已关闭”，按钮禁用。
- `running`：先优雅关闭；只有后端返回仍存在且用户确认后才强制关闭。
- busy 状态下锁定按钮，防止重复请求；错误显示具体阶段而非“操作失败”。

按钮放在高于雷达/普通面板的 stacking context，确保异常 overlay 不遮挡；浅色、深色和已有 palette 均使用语义 token，不写死暗色。

### 4.2 状态误判修复

前端优先采用结构化快照的 `confidence` 和 `observedAt`；如果暂时保留布尔 IPC，至少在 store 中实现双采样确认、错误时清空旧 running 状态，并在窗口重新聚焦时立即刷新。Steam 状态只作为可选诊断文本，不能参与 `cs2ProcessState`。

## 5. 备份默认策略与配置契约

### 5.1 用户可见选项

在执行安装/更新/Panel 写入操作的确认区域加入复选框“本次操作保留写前备份（默认关闭）”。这是一次性选项，不写入 localStorage；每次操作开始重置为 false。Tauri 调用显式传递 `keepBackup: boolean`，缺省值在 Rust 端仍按 false 处理，防止旧前端误传导致大量备份。

### 5.2 Rust 写入契约

将 `atomic_write(path, bytes)` 改为 `atomic_write(path, bytes, BackupPolicy)`，策略至少包含 `keep_backup` 与 `operation_id`。默认写入安全流程仍可创建临时备份用于失败回滚，但：

- 成功且 `keep_backup=false`：立即删除本次临时备份。
- 成功且 `keep_backup=true`：仅保留本次备份，并将绝对路径加入 `OperationResult`/日志，便于用户找回。
- 失败：回滚成功后删除临时备份；回滚失败才保留并在错误中给出路径。
- 使用唯一 operation id，不能用毫秒时间戳单独命名，以免并发覆盖。

在 `src-tauri/src/services/panel.rs` 集中改造所有 `atomic_write` 调用，不能只改一个入口。对于安装事务中的 `backup/Online`、`backup/WithBots` 和 rollback 目录，继续按现有事务语义保留/清理，不把它们纳入通配符删除。

### 5.3 旧备份清理

增加受限的 `cleanup_owned_backups(root)`：只扫描助手明确生成的文件名/目录模式（例如 `*.backup-<operation-id>`、助手事务目录），先校验 canonical path 位于当前 CS2 根目录或其 `game/csgo` 子目录，再按“非本次保留、非活动事务、非最近失败回滚”删除。不得递归删除名为 `backup` 的普通目录，不得删除用户手工命名文件，不得跨根路径。

触发时机：助手启动一次、每次写操作成功后一次、关闭/卸载前一次。清理应可重入、失败只告警并写日志，不阻塞主要成功结果。提供 `dry_run` 单元测试输出待删列表，真实清理只由 Rust 执行，禁止浏览器删除。

## 6. 测试与证据门禁

### 自动化

- Rust：精确进程名、Steam 与 cs2 区分、双采样、优雅关闭超时、强制关闭 PID 白名单、权限错误、备份保留/删除/回滚、路径越界保护、清理幂等。
- 前端：按钮在四种状态的禁用/文案、二次强制确认、重复点击锁定、`aria-label`、错误恢复；`keepBackup` 每次默认 false 且只影响当前调用。
- 回归命令：

```powershell
npm test -- --run tests/cs2-process-polling.spec.ts tests/bot-plugin-gate.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 panel
```

### 受控进程验收

使用一个受控的测试进程或测试桩验证：无 Steam/无 cs2、仅 Steam、真实 cs2 正常退出、真实 cs2 强制退出、同名 `.bak` 文件。记录 PID、路径、开始/结束时间和最终快照。不得用 `taskkill /IM cs2.exe` 作为唯一证据。

### 真实 Tauri/浏览器矩阵

在 Codex 内置浏览器复现两个故障并截图；再启动真实 Tauri（不是仅 Vite 页面）验证：

1. CS2 与 Steam 均未运行：状态稳定为“未运行”，关闭按钮不可用。
2. 只启动 Steam：仍为“未运行”。
3. 启动 CS2 后点击优雅关闭：8 秒内状态变为“未运行”。
4. 模拟无响应后确认强制关闭：只结束 CS2 PID，Steam 和助手仍存活。
5. 连续执行安装/更新 10 次：默认无新增 `.backup-*`；勾选一次只留下该次；下一次默认操作不再保留。
6. 浅色/深色、1280x800 与 980x640：按钮不被雷达或弹层遮挡，无横向溢出，焦点可见。

真实 CS2 关闭可能导致未保存内容丢失，必须由用户执行最终游戏环境验收；执行 AI 不得把受控桩测试冒充真实游戏证据。

## 7. 停止条件与交接产物

出现以下任一情况立即停止扩大改动并记录根因：无法区分目标 PID、强制关闭会影响 Steam/助手、路径 canonicalize 失败、回滚失败且无法安全清理、Tauri 与浏览器行为冲突、真实窗口仍持续显示旧 running。不得通过放宽进程匹配或删除整个 backup 目录“解决”。

实际执行 AI 完成后必须更新本文件或新建同目录执行报告，包含：变更文件清单、IPC/DTO 版本、日志片段、测试命令结果、截图绝对路径与 SHA-256、备份清理前后文件清单、真实 Tauri/用户验收状态。未通过真实矩阵前，状态写“部分实现/待验收”，不得声明客户端故障已彻底修复。
