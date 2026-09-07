# 0.5.12 人机强度工坊与窗口拖动调查、设计和执行计划

日期：2026-09-07

## 结论摘要

本轮先记录原因和执行计划，不直接修改功能代码。

1. 人机强度工坊的短暂无响应主要来自同步 Tauri command 中直接执行 VPKEdit CLI、轮询等待子进程和同步读取/哈希 VPK。打开、切换、创建、保存、应用都会经过这一类工作，耗时越长，窗口越容易表现为无响应。
2. 少数玩家的提取错误是“输出目标无法创建或写入”的失败，不是 BOT 数据本身必然损坏。反馈路径为旧版固定 `open-custom-...\\botprofile.db`，而当前源码已在 `extract_db()` 中追加独立 `extract-<timestamp>` 工作目录；这说明受影响用户需要先确认实际安装器版本，再区分旧实现残留、目标文件占用、杀毒/受控文件夹拦截和目录权限问题。
3. 当前标题栏拖动只绑定在 `.app-titlebar`。公告、更新、根目录、强度工坊等 Teleport 到 `body` 的 `position: fixed` backdrop 会覆盖标题栏，鼠标事件被 backdrop 截走；右侧抽屉虽从 `top: 44px` 开始，但其它 modal 没有统一保留标题栏区域。

## 证据定位

### 工坊卡顿

- `src-tauri/src/commands/bot_difficulty.rs` 的命令均为同步函数。
- `src-tauri/src/services/bot_difficulty.rs:42-80` 的 `run_cli()` 使用同步 `Command`、循环 `try_wait()`、25ms sleep 和最多 30 秒等待。
- `src-tauri/src/services/bot_difficulty.rs:341-400` 的 `list()` 会读取活动/内置 VPK、检查工具状态并扫描自定义档案。
- `src-tauri/src/services/bot_difficulty.rs:478-517` 的 `open()` 会先检查 CS2，再启动 VPKEdit 提取并读取整个 `botprofile.db`。
- `src/components/BotDifficultyWorkbench.vue:9-15` 的 `load()`、`selectProfile()`、`createFromBuiltin()`、`save()`、`apply()` 都在用户交互路径上直接等待 Tauri 调用完成；快速切换没有请求序列号或取消旧请求。
- `save()` 会执行 VPK 删除条目、添加条目、再次提取回读和全文件哈希，属于最重操作；`create()` 也会复制 VPK 后立即提取。

### 少数提取失败

- 当前 `extract_db()` 使用 `workspace/extract-<timestamp>/botprofile.db`，并通过全局互斥避免同时运行提取。
- 反馈中的目标路径没有 `extract-*` 子目录，优先怀疑旧安装器/旧代码路径，不能直接把当前源码的隔离逻辑当成玩家已获得的修复。
- 即使使用新路径，VPKEdit 仍可能因以下原因返回退出码 1：目标文件已存在且被旧进程或杀毒软件锁定；AppLocalData 目录被受控文件夹/安全软件阻止；用户目录或磁盘权限异常；磁盘空间不足；VPK 内部条目异常或输入 VPK 正被其它程序读取。
- 现在错误只返回 VPKEdit 的 stderr，没有记录输出目录是否可写、目标是否已存在、输入/输出所在磁盘、剩余空间、重试次数和工具版本，导致少数机器无法区分原因。

### 窗口拖动

- `src/components/AppTitlebar.vue` 只在 `.app-titlebar` 上监听 `@mousedown.left="startDragging"`。
- `.titlebar-controls` 使用 `@mousedown.stop` 排除按钮，这是正确的窗口控制保护。
- `src/styles/main.css:391` 的通用 `.modal-backdrop` 使用 `position: fixed; inset: 0; z-index: 10`。
- 公告 backdrop 为 z-index 48，BOT 工坊 backdrop 为 30，更新/根目录等 modal 使用通用 backdrop；这些元素会覆盖顶部 44px 标题栏。
- `AppearanceSettingsDrawer`、`PlayerDrawer` 已从标题栏下方开始，但没有形成所有浮层统一的标题栏保留契约。

## 设计方案

### A. 消除工坊卡顿

1. 将 VPKEdit 相关 Tauri command 改为异步入口，阻塞工作放到专用 blocking worker；UI 线程只负责发起任务、接收结果和更新状态。
2. 为每个工坊会话增加请求序列号/取消标记：新选择覆盖旧选择，旧结果不能回写当前档案；保存、应用、创建期间继续锁定交互，但不冻结窗口。
3. 将 `list` 拆成轻量元数据读取和后台工具状态检查，打开工坊先显示档案列表，工具检查/数据库读取在后台完成。
4. 保留全局 VPK 操作互斥，但把锁状态转换成明确的“另一个档案操作正在完成”提示，而不是让用户看到无响应。
5. `run_cli()` 改为异步子进程等待或 blocking task，保留 30 秒超时；增加阶段名、耗时、退出码和 stderr 摘要，禁止把完整路径/玩家敏感信息回传到公告或日志。

### B. 提取失败自愈

1. 每次提取使用新的随机目录，而不只依赖时间戳；开始前验证父目录可创建、可写和剩余空间。
2. 目标文件存在时先尝试删除；删除失败要明确返回“目标被占用/权限被拒绝”，不继续把它包装成泛化的 dependency invalid。
3. 对退出码 1 的纯输出目标失败做一次短延迟重试，重试仍失败才返回错误；不得对输入 VPK 做破坏性覆盖。
4. 记录并显示工具版本、工具 SHA、输入 VPK SHA、输出目录、是否重试、Win32 错误类别和磁盘剩余空间，方便少数用户反馈可诊断证据。
5. 对旧版本固定工作目录执行一次兼容清理：只清理应用自己的 `bot-difficulty-workshop/workspace/open-*` 临时文件，保留 profiles、backups 和用户自定义 VPK。
6. 增加权限/占用/失败重试 fixture；真实用户环境仍需覆盖 Defender/受控文件夹和非系统盘路径。

### C. 永久窗口拖动区域

1. 建立统一的标题栏保留区契约：所有应用 modal backdrop 的顶部从 `44px` 开始，窗口标题栏始终位于最高层；右侧抽屉继续从 `top: 44px` 开始。
2. 标题栏控制按钮继续阻止拖动；标题栏内品牌、上下文和空白区域始终可拖动，按钮/链接/输入框不触发拖动。
3. 在标题栏增加明确的拖动 surface 标记，并保留 Tauri `startDragging()` 作为兼容回退；避免依赖浮层是否 Teleport 到 body。
4. 将公告、更新、根目录、BOT 工坊、捐赠、地图轮换等浮层统一检查，不允许任何新的 `inset: 0` 覆盖标题栏。
5. 增加窗口层级契约测试和 Tauri 窗口实测：无浮层、公告、更新、强度工坊、外观抽屉、右侧复盘抽屉、开屏层分别拖动标题栏，确认按钮仍可点击。

## 执行顺序

1. 先在真实 0.5.12 安装器中确认反馈路径对应的实际版本，并收集 `tool version / tool SHA / input VPK SHA / output path / free space / retry`。
2. 先完成工坊异步化和请求序列保护，再做提取自愈；两者分别增加测试，避免把性能问题和少数机器失败混成一个补丁。
3. 再统一浮层顶部保留和标题栏拖动 surface，补充无障碍、按钮点击和窗口拖动回归测试。
4. 运行前端 typecheck、定向 Vitest、Rust bot-difficulty 测试和 Web/Tauri 构建。
5. 由用户在真实 CS2 BOT 环境验证打开、切换、创建、保存、应用，以及至少一台受影响机器的提取回读；本地测试不能替代这些证据。

## 发布门禁

- 工坊操作过程中窗口可拖动、可响应，旧请求结果不会覆盖新选择。
- 提取失败能区分路径占用、权限/安全软件、磁盘空间、工具异常和 VPK 输入异常。
- 旧临时目录清理不触碰自定义档案、备份和活动 VPK。
- 所有浮层出现时顶部标题栏仍可拖动，标题栏按钮不会误触发拖动。
- 在真实受影响电脑上至少完成一次打开内置档案、创建自定义档案、保存回读和应用验证后，才能把少数机器问题标记为修复完成。
