# 0.5.6 Demo 播放、文件定位、赛后最新战报与发布交接方案

> 交接对象：下一位【实际执行 AI】。两个 AI 不共享上下文，本文必须独立执行。
>
> 本文由【方案制定 AI】在 2026-08-05 基于 `E:\CS2AS05` 当前脏工作树、参考项目固定提交和本机发布材料调查后编写。本轮只新增本文，没有实现功能、没有构建新安装器、没有提交/推送、没有上传 GitHub、没有写 R2/D1、没有部署官网。

## 1. 目标与完成定义

实际执行必须同时完成：

1. 对录像库中每个已扫描入库、原文件仍存在的 Demo 提供：
   - **播放**：关闭状态下启动 CS2 并播放该 Demo；不依赖浏览器能力，不修改原 Demo。
   - **在文件夹中显示**：打开 Windows 资源管理器并选中该 `.dem` 文件。
2. 每次真实游戏会话结束后，识别该会话产生的**最新一份 Demo**，等待稳定性检查和后台解析；只有报告完整提交且满足计分板可展示契约时才显示独立计分板。解析中不显示，解析失败不显示空窗口，也不能误弹旧局。
3. 生成全新的 `0.5.6` 发布文件集，完成自动化、真实 Windows/Tauri、真实 CS2、安装升级和 updater 签名验证；全部通过后才能进入 GitHub/R2/D1 发布。

这里的“播放”是让 CS2 播放 Demo，不是应用内二维地图回放；现有 `MatchViewer2D` 保持不变。

结果必须分级：

- **已实现**：代码和自动化通过。
- **可发布候选**：再通过真实 Demo、Tauri、CS2 播放、安装升级和签名产物验证。
- **正式发布**：再完成发布 commit/tag、唯一一次 push/GitHub Release、R2 GET 回读、D1/feed 和旧版客户端 updater 验证。

中间状态不能写成“0.5.6 已正式发布”。

## 2. 当前基线，不得回退

### 2.1 Git 与版本

- 工作目录：`E:\CS2AS05`。
- 当前分支：`main`，通常不新开分支。
- 当前 HEAD：`8552b554993fec66866d2133d315342be3e0cbca`，提交信息 `fix: unblock 0.5.5 scoreboard and recovery flows`。
- 当前有大量已跟踪修改和未跟踪的 Demo 平台、BotVision、Rating、viewer、preflight、文档及资源。这些属于现有工作树，必须逐项审阅并在其上继续。禁止 `git reset --hard`、`git checkout --`、`git clean` 或为了缩小任务回退用户改动。
- `package.json`、`package-lock.json` 顶层、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 自身 package、`src-tauri/tauri.conf.json` 当前均为 `0.5.6`。
- 技术栈是 Tauri 2 + Vue 3 + TypeScript + Rust + SQLite，Windows x64 NSIS，主二进制 `CS2BotImproverAssistant`。

执行开始重新保存：

    Set-Location E:\CS2AS05
    git status --short --branch
    git rev-parse HEAD
    git diff --stat
    git diff --check

不要把本文调查时的状态当实时状态。

### 2.2 现有 Demo 能力

现有代码已经提供：

- 本地 Demo roots、扫描、导入、SQLite 录像库和异步 analysis jobs。
- Rust 全局 parse gate：CS2 运行时不允许解析；parser open、解析后和 commit 前复查文件。
- `launch_demo_at_tick(demo_id, tick, player_key)`，目前通过 Steam 参数 `-applaunch 730 +playdemo <path> +demo_gototick <tick>` 启动。
- 预加载但默认隐藏的 `scoreboard` Tauri 窗口。
- `open_scoreboard` 先读取报告再下发 load event；前端成功读取后调用 `scoreboard_present`，此时才显示和聚焦。
- `demo://report-ready` 事件和主窗口监听。
- Demo 列表按 `mtime_ms DESC` 排序；当前只有“弹出本局战报”“打开报告”“重新解析”，没有播放和文件定位。

必须复用这些能力，不另起 Electron、Python/FastAPI 或浏览器文件系统链路。

### 2.3 当前赛后竞态

`observe_assistant_launch` 当前流程：

1. 等待助手启动的 CS2 出现并退出。
2. 调用 `scan`。
3. 立即查询 `status=done` 列表。
4. 找第一条 `mtime_ms >= started_at - 5000` 并发 `demo://report-ready`。

但 `scan` 只把新 Demo 放入后台队列，worker 之后才解析，因此：

- 新 Demo 仍为 `queued/parsing` 时不会弹。
- 同一时间窗口存在旧 `done` 记录时可能弹上一局。
- 没有绑定本次 CS2 session 与候选 Demo ID。
- 没有等待 `report_json`、schema、metrics 和 scoreboard 数据完整。
- 只覆盖助手主动启动的 CS2。
- 播放 Demo 也会启动 CS2；若不区分 session kind，退出回放后可能误弹战报。

必须修 Rust 状态模型，不能在 Vue 中增加固定延时。

## 3. 参考项目与许可证边界

参考仓库：`https://github.com/DrEAmSs59/CS2-insight-agent`。

固定调查版本：

    commit 17d2a213ee8c32608feee3c63f0b6d05eef8f945
    date   2026-08-02
    title  fix(ci): restore green main validation

本机临时调查副本：

    C:\Users\GOPtZ\AppData\Local\Temp\codex-cs2-insight-agent

参考项目的录像库给每行直接提供 Lucide `Play` 和 `FolderSearch`。Windows 文件定位使用三个独立参数：

    explorer.exe
    /select,
    <absolute path>

其播放服务值得学习的原则：

- 播放前确认文件存在。
- CS2 已运行时拒绝第二个 managed playback。
- 使用参数数组，不拼 shell 命令。
- 对外部 Demo 建受控临时播放副本，不修改原件。
- 监控真实 CS2 生命周期，退出后清理临时副本。
- 所有失败路径 best-effort cleanup。

**许可证边界：**参考项目顶层是 **PolyForm Noncommercial 1.0.0**，不是 MIT；我们的项目是 AGPL-3.0-or-later。因此只能学习交互、状态机和 Windows 参数语义：

- 禁止复制其 Python、React、测试、文案、样式或资产。
- 禁止把参考仓库加入 `third_party` 或发布包。
- 必须用本项目现有 Rust/Tauri/Vue 独立实现。
- `NOTICE.md` 可注明“行为参考”及仓库/commit，但不能错误写 MIT，不能声称合入源码。

## 4. 目标架构

    DemoLibrary row
      -> play_demo(demoId, selectedCs2Root)
      -> reveal_demo_file(demoId)

    Rust GameSessionCoordinator
      -> observe CS2 false -> true -> false
      -> kind: live_match | demo_playback
      -> scan after exit
      -> bind newest candidate demo id
      -> wait exact candidate done/error
      -> validate presentable report
      -> emit report-ready or report-failed

    Hidden scoreboard window
      -> accept exact report id
      -> load report
      -> render ready
      -> scoreboard_present
      -> only now show/focus

禁止：

- 浏览器 `<input type=file>`、`window.open` 或浏览器文件系统能力。
- 前端传任意路径后拼 shell 命令。
- `cmd /c start`、PowerShell 字符串命令或 URL scheme 冒充文件定位。
- 游戏退出后固定 sleep 再猜列表第一条。
- 先显示空计分板再等解析。
- 仅从 `status=done` 子集挑旧记录。

## 5. 阶段 A：Demo 播放

### 5.1 IPC 与错误契约

新增：

    play_demo(demo_id: i64, root_path: String)
      -> Result<DemoPlaybackResult, String>

建议返回字段：

    demoFileId, sessionId, sourcePath, preparedPath, started

稳定错误码：

    DEMO_NOT_FOUND
    DEMO_FILE_NOT_FOUND
    DEMO_PLAYBACK_CS2_RUNNING
    DEMO_PLAYBACK_BUSY
    DEMO_PLAYBACK_ROOT_INVALID
    DEMO_PLAYBACK_COPY_CHANGED
    STEAM_NOT_FOUND
    DEMO_LAUNCH

错误详情必须带恢复动作，如“请先退出 CS2 后重试”，不能只显示 IO 文本。

### 5.2 Rust 实现

建议新增 `src-tauri/src/demo/playback.rs`，避免继续膨胀 `services/demo.rs`：

1. 根据 `demo_id` 从 SQLite 取路径，前端不能传任意 Demo 路径。
2. `dunce::canonicalize` 后确认是现存普通 `.dem`；拒绝目录、缺失和错误扩展名。
3. 调用 `cs2::check_cs2_process()`；CS2 已运行立即拒绝。
4. `root_path` 必须通过现有 `cs2::normalize_root` / `inspect_cs2_root`，并解析真实 `game\csgo`。
5. 增加单会话 `DemoPlaybackState` 并由 Tauri `manage`，防止连续双击启动多个 CS2。
6. 复制前记录源 `size + mtime`，复制后复查；变化则删除临时副本并返回 `DEMO_PLAYBACK_COPY_CHANGED`。
7. 源已在当前 `game\csgo` 下时生成合法相对路径且不复制；外部 Demo 复制到：

       <root>\game\csgo\replays\_cs2as_play_<session-id>.dem

   临时名由程序生成，不用用户文件名，不覆盖已有文件。
8. 使用 Steam discovery 和 `std::process::Command` 参数数组：

       -applaunch
       730
       -insecure
       -novid
       +playdemo
       <relative managed demo path>

   不经过 shell。是否保留 `-console` 以当前体验为准，不能强制控制台遮挡播放。
9. 监控 Steam child 与真实 `cs2.exe`；短命 Steam child 不算结束，给真实 CS2 最多约 12 秒出现，再每秒检查直到退出。
10. 退出后只删除本 session 创建的临时 Demo；不删除、移动、改写原件，不递归删除目录。
11. 所有失败路径释放 busy；一次失败不能永久锁死。
12. 启动前通知赛后协调器 session kind 为 `demo_playback`。

参考项目的二进制兼容改写和 POV HUD 不在本次范围。旧 Demo 不兼容当前 CS2 时显示已知限制，不临时发明重写器。

### 5.3 与 tick 启动统一

优先让现有 `launch_demo_at_tick` 调用同一个 playback service，传入 `tick > 0`。若暂时保留现有实现，也必须加入 file/CS2-running 防线并标记 `demo_playback`，否则退出回放会误弹战报。`player_key` 仍不支持时保留现有明确错误。

## 6. 阶段 B：在文件夹中显示

新增：

    reveal_demo_file(demo_id: i64)
      -> Result<RevealDemoResult, String>

实现要求：

1. 只按 `demo_id` 从 SQLite 取 path。
2. canonicalize 并确认文件存在。
3. Windows 使用：

       Command::new("explorer.exe")
         .arg("/select,")
         .arg(&canonical_path)
         .spawn()

4. 不能拼接整条命令；空格、中文、逗号、`&` 路径仍是一个 path arg。
5. 未来若保留跨平台编译，macOS `open -R`，Linux 打开 parent；0.5.6 验收只针对 Windows x64。
6. 文件已移动/删除时返回 `DEMO_FILE_NOT_FOUND` 并提示重新扫描，不打开默认“文档”目录。

不需要引入 `tauri-plugin-shell`。

## 7. 阶段 C：录像库 UI

文件级修改：

- `src-tauri/src/commands/demo.rs`：新增两个 command wrapper。
- `src-tauri/src/lib.rs`：注册 commands，manage playback/session state。
- `src-tauri/src/models/demo.rs`：新增返回 DTO（如采用）。
- `src/services/tauri/demo.ts`：新增 `playDemo`、`revealDemoFile`。
- `src/types/demo.ts`：同步 DTO。
- `src/stores/demo.ts`：增加 per-row busy，不能用全局 busy 锁整表。
- `src/views/DemoReviewView.vue`：新增按钮和反馈。
- `src/styles/main.css`：只补稳定操作列布局和状态，不重做主题。

每行固定顺序建议：

    [Play] [FolderSearch] [MonitorUp] [打开报告]

UI/UX Pro Max 约束：

- 使用现有 `lucide-vue-next` 的 `Play`、`FolderSearch`，禁止 Emoji/手绘 SVG。
- 图标 16-18px，按钮尺寸稳定；紧凑桌面表格可见框 36-40px，但通过 padding/命中层接近 44px，不让动态文字改列宽。
- `title` 和 `aria-label` 分别为“用 CS2 播放 Demo”“在文件夹中显示 Demo”。
- 不依赖 hover 才出现；Tab 可达，`focus-visible` 清楚。
- 准备播放时只禁用该行，用 `LoaderCircle`；整表不得跳动。
- CS2 running 或 playback busy 时 Play disabled 并有 tooltip；后端仍二次校验。
- 文件定位不依赖解析状态。播放也不依赖报告 `done`，但写入中/CS2 running/metadata 不稳定由后端拒绝。
- `done` 才显示战报；`error` 仍允许播放和定位，并保留重试。
- 最小窗口下只允许表格容器横向滚动，按钮不覆盖状态。
- 动效 150-220ms，遵守 `prefers-reduced-motion`。

## 8. 阶段 D：赛后最新战报协调器

### 8.1 单一 Rust 状态机

建议新增 `src-tauri/src/demo/post_match.rs`，在 Tauri setup 启动后台观察器。不要继续让每次 `launch_panel_cs2` 创建独立 360 秒线程。

状态：

    Idle
    Running {
      session_id,
      kind: live_match | demo_playback,
      detected_at,
      assistant_started_at?,
      baseline_fingerprints
    }
    WaitingForStableDemo { session_id, ended_at, candidate_demo_id }
    WaitingForReport { session_id, demo_id, deadline }
    Presented | Failed | Suppressed

### 8.2 会话识别

- 每秒检查 CS2，识别 `false -> true` 和 `true -> false`。
- 助手启动时向 coordinator 提供精确 `started_at` 与 `live_match`。
- 应用运行期间从 Steam/桌面启动也建立 `live_match`。
- `play_demo` 与 `launch_demo_at_tick` 启动前标记下一 session 为 `demo_playback`。
- 应用启动时 CS2 已运行：建立 session 并记录当时 baseline，退出后只考虑 baseline 之后新增/变化的 Demo。
- 同一 CS2 生命周期只有一个 session ID；多个入口不能产生多个 observer。

### 8.3 候选选择

CS2 退出后：

1. 复用 parse gate/watcher 的稳定文件规则，至少三次 size/mtime 稳定采样，不能只看一次 `is_file`。
2. 扫描，把本 session 期间新增或 fingerprint 变化的 Demo 记为 candidate set。
3. 先按 `mtime_ms DESC, id DESC` 确定唯一最新 candidate ID，再等待它；不能从 `done` 子集挑旧局。
4. candidate 为空不弹，只记录 `POST_MATCH_NO_DEMO`。
5. `demo_playback` session 可扫描/缓存，但直接 `Suppressed`，不自动弹战报。
6. 对精确 demo ID 最多等待合理上限（建议 10 分钟）。
7. `queued/parsing` 等待；`error/canceled` 停止并通知主窗口，不打开 scoreboard。
8. 一次 session 多份 Demo 只展示最新一份，不能先弹旧的再替换。

### 8.4 可展示报告的唯一契约

新增纯函数 `report_is_presentable(report)`。自动弹窗至少要求：

- `demo_files.status == done`。
- core job `stage == done` 且事务已 commit。
- `report_json` 可反序列化。
- schema、parser adapter、metrics **等于**当前常量。
- `summary.demo_file_id` 等于候选 ID。
- `summary.total_rounds > 0`。
- 至少一名 `participantRole=player`。
- `scoreboard_status` 为 `complete` 或项目明确允许的 `partial`；`unavailable` 不自动弹。

Rating 某字段缺失可以按现有契约显示 `--`，但不能弹完全没有玩家的空表。手动“打开报告”可继续显示 partial/诊断；自动计分板用严格 gate。`open_scoreboard` 本身也必须调用 gate，不能绕过事件传坏 ID。

### 8.5 事件与恰好一次

把裸 number 升级为对象：

    demo://report-ready
      sessionId, reportId, fileName, completedAt, origin=post_match

失败事件：

    demo://report-failed
      sessionId, demoId, errorCode, message

要求：

- 每个 session 最多 emit 一次 ready 或 failed。
- AppShell 记录最近已处理 session ID，重复事件不重复打开。
- 手动点击战报不使用 session 去重。
- 新 ready 使用现有 `ScoreboardState.sequence`；旧异步 load 不能覆盖新报告。
- scoreboard 尚未 ready 时 pending 保留；load 成功后才 show。
- 5 秒 boot timeout 只报错，不显示空窗口。

### 8.6 删除旧竞态

新 coordinator 完成后删除或改造 `observe_assistant_launch` 的旧轮询。`panel::launch_cs2` 只通知 coordinator，不启动第二套 observer。测试应断言只有一个赛后观察器。

## 9. 阶段 E：测试矩阵

### 9.1 Rust

播放必须覆盖：Demo ID/文件/扩展不存在；CS2 running；root 无效；空格、中文、`&`、逗号路径；外部 Demo managed copy 且原 hash 不变；复制时源 metadata 变化；Steam child 短命但真实 CS2 后出现；正常退出和所有失败路径释放 busy；tick 播放与列表播放都标记 `demo_playback`。

文件定位必须覆盖：Windows 参数严格为 `explorer.exe`、`/select,`、canonical path 三段；特殊路径仍为单个 path arg；ID 不存在/文件删除时不启动 Explorer；前端不能传任意 path。

赛后协调器至少覆盖：

1. `false -> true -> false` 只建立一个 live session。
2. 较新 candidate 仍 parsing、较旧已 done 时必须等待较新。
3. candidate `queued -> parsing -> done`，事务 commit 后恰好 emit 一次。
4. candidate `error/canceled` 只 emit failed，不 show scoreboard。
5. schema/adapter/metrics 旧、JSON 损坏、0 rounds、players 空、scoreboard unavailable 均不可自动展示。
6. partial 是否允许展示用固定测试锁定。
7. playback session 完全 suppress 自动战报。
8. App 启动时 CS2 已运行，也能关联 baseline 之后的新文件。
9. 同一 session 重复 worker notification 不重复弹。
10. 下一局新 sequence 可替换旧 pending，旧 load 不能 show 错报告。

### 9.2 Vue/Vitest

- 每行有 Play、FolderSearch 和完整 accessible name。
- `done/error/queued/parsing` 操作组合正确。
- per-row busy 不影响其他行和分页。
- CS2 running 时 Play disabled，定位仍可用。
- report-ready 对象打开 exact report ID；重复 session ID 不重复调用。
- report-failed 只通知，不打开 scoreboard。
- scoreboard 仍 `visible=false`，经 `scoreboard_present` 才显示。
- 860x600、980x640、1280x800 无按钮覆盖，仅表格容器横向滚动。
- 键盘、focus-visible、reduced-motion 保留。

### 9.3 一次性自动化 Gate

按用户要求，同一种验收最多跑一次；完成代码和静态审阅后统一执行：

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
    git diff --check

首次 Rust 编译或网络慢时耐心等待，不因短暂无输出并发重跑。保存 command、started/ended、exit code、stdout/stderr 到：

    workspace/release-evidence/0.5.6-demo-actions-postmatch-<timestamp>/

证据目录不得提交。

## 10. 阶段 F：真实 Windows/Tauri/CS2 验收

### 10.1 文件定位

- 选择路径含空格和中文的 Demo，点击 FolderSearch。
- Explorer 必须打开正确父目录并选中文件，不是默认“文档”。
- 文件被移动/删除后应提示重新扫描，不打开错误目录。

### 10.2 播放

- 记录原 Demo size/SHA-256；确认 CS2 关闭后点击 Play。
- CS2 以 `-insecure` 启动并真正进入目标 Demo，不停在主菜单。
- 退出后无 `_cs2as_play_*.dem`、cfg、第二个 CS2、线程或 busy 残留。
- 原 Demo size/SHA-256 不变；CS2 已运行时再次 Play 被阻止。
- 外部目录和 `game\csgo\replays` 内 Demo 各测一个。
- Demo 版本不兼容时只报告限制，不修改原件。

### 10.3 赛后最新战报

由用户完成一次最短真实 BOT 对局：

1. 从应用启动 CS2，至少完成 3 个正式回合并正常退出。
2. CS2 运行期间确认 demoparser 没有解析。
3. 退出后可显示稳定/解析进度，但 scoreboard 保持隐藏。
4. 最新 Demo core job commit 后，独立 scoreboard 只弹一次。
5. 文件名、地图、比分、回合、玩家 K-D-A 与最新 Demo 一致，不是上一局。
6. 解析失败模拟只通知主窗口，不弹空 scoreboard。
7. 随后 Play 回放旧 Demo；退出回放不得触发“最新比赛”战报。

保存 session ID、candidate ID、fingerprint、job stages、ready emit 和 scoreboard present 时间。日志不得含密钥。

## 11. 阶段 G：0.5.6 发布材料调查结论

### 11.1 本机 updater 密钥

已确认存在，只允许本机读取，不得提交、复制到证据、打印或上传：

    C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key
    C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key.pub
    C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater-password.dpapi

调查时大小为 348、152、622 bytes。磁盘 `.pub` 是 Base64 包装；解码后与 `tauri.conf.json` 内嵌公钥解码结果完全一致。执行时必须同层比较，不能直接比较 152-byte 包装文本与 113-character 解码文本，也不能因此擅自换钥。

安全核对只输出布尔结果：

    $keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
    $conf = Get-Content .\src-tauri\tauri.conf.json -Raw | ConvertFrom-Json
    $embedded = [Text.Encoding]::UTF8.GetString(
      [Convert]::FromBase64String($conf.plugins.updater.pubkey)
    ).Trim()
    $diskWrapped = (Get-Content (Join-Path $keyDir 'updater.key.pub') -Raw).Trim()
    $diskDecoded = [Text.Encoding]::UTF8.GetString(
      [Convert]::FromBase64String($diskWrapped)
    ).Trim()
    if ($embedded -cne $diskDecoded) { throw 'Updater public key mismatch' }

### 11.2 当前已有产物不是最终产物

调查时存在：

    src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.6_x64-setup.exe
    src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.6_x64-setup.exe.sig
    dist-release\cs2-bot-improver\updater-prod.json

当前候选 EXE：

    size         112391145 bytes
    sha256       1AC9E290E6ABDB7272A42772A6F7BBE06A83D83790315D6D298F2B85D7CD18EA
    Authenticode NotSigned

当前 `.sig` 为 436 bytes；manifest 的 version/channel/projectId、size 和 SHA-256 与该 EXE 一致。但本方案功能尚未实现，任何修改都会使 hash/签名失效。必须先把当前同版本候选移动到 evidence backup，全部 Gate 后重新构建；禁止发布当前文件，也不能让 `release-manifest.mjs` 误选旧候选。

Tauri updater `.sig` 与 Windows Authenticode 是两件事。没有 Windows 代码签名证书时如实写“未做 Authenticode”，不能冒充 Windows 发布者签名。

## 12. 阶段 H：生成全新发布文件

### 12.1 安全构建

先把旧同版本 EXE、`.sig`、`.sha256` 和 manifest **移动**到 evidence backup，不递归删除其他版本。只在当前 PowerShell 进程解密 DPAPI 和注入 key：

    Set-Location E:\CS2AS05
    $keyDir = 'C:\Users\GOPtZ\Documents\CS2AS05-release-keys'
    $secure = Get-Content (Join-Path $keyDir 'updater-password.dpapi') -Raw |
      ConvertTo-SecureString
    $credential = [PSCredential]::new('tauri-updater', $secure)
    $env:TAURI_SIGNING_PRIVATE_KEY = (
      Get-Content (Join-Path $keyDir 'updater.key') -Raw
    ).Trim()
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
      $credential = $null
      $secure = $null
    }

不得回显环境变量。构建最多 5 次；同一失败无新诊断时不重复。

### 12.2 发布所需文件

| 文件 | 预期路径/生成方式 | 用途 |
|---|---|---|
| NSIS 安装器 | `src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.6_x64-setup.exe` | GitHub、夸克、R2 |
| Tauri updater 签名 | 同路径 `.exe.sig` | updater feed/D1/GitHub |
| SHA-256 文件 | 同路径 `.exe.sha256` | 用户/GitHub 校验 |
| updater manifest | `dist-release/cs2-bot-improver/updater-prod.json` | 本地发布证据 |
| 发布说明 | `docs/release-notes-0.5.6.md` | GitHub、官网 D1、公告同源 |
| 源码 tag | `v0.5.6` 指向最终 commit | AGPL 对应源码 |
| LICENSE/NOTICE | 仓库 `LICENSE`、`NOTICE.md`，bundle resources 已包含 | 合规 |

`updater-prod.json` 当前含本机绝对 installer path，可作本地证据，但不得原样上传或提交公开仓库。若要公开，先改生成器输出 basename/公开 URL、补测试再生成，不能手改 JSON。

发布说明至少包含：录像库 Play/FolderSearch；最新 Demo 解析成功后才弹计分板；回放 suppression；最终实际验证的比分/BOT/observer/Rating/BotVision 状态；旧 Demo/CS2 running/Authenticode 已知限制；参考项目是 PolyForm Noncommercial 行为参考且未复制代码。

### 12.3 一致性检查

- EXE size/mtime/SHA-256。
- `.sig` size/mtime/SHA-256、非空。
- manifest version=`0.5.6`、channel=`prod`、projectId=`cs2-bot-improver`。
- manifest size/SHA-256/signature 与最终文件精确一致，`pub_date` 晚于 EXE mtime。
- `Get-AuthenticodeSignature` 单独记录。
- 安装后文件/产品版本为 0.5.6。
- bundle 有 LICENSE、NOTICE、`CS2BotImprover.zip`、地图资源。
- 插件 marker、BotVision、Rating、Demo schema/metrics 与代码一致。

## 13. 阶段 I：安装与升级 Gate

1. 干净安装：启动、主窗口、隐藏 scoreboard、Demo roots、定位和播放可用。
2. 从正式 0.5.5 覆盖安装：用户设置、CS2 root、Demo SQLite 历史和插件选择保留。
3. 卸载/重装：不误删原始 Demo；应用数据按现有 NSIS 契约。
4. 快捷方式目标、图标、工作目录仍指向 `CS2BotImproverAssistant.exe`。
5. 安装后的 0.5.6 真正完成一次 post-match 自动战报。
6. 用正式旧版客户端走 updater 被动安装，验证 `.sig` 与内嵌公钥；只有本地签名文件不算端到端成功。

任一步失败，停止在候选。

## 14. 阶段 J：GitHub、夸克、R2/D1

### 14.1 Git/GitHub

发布前审阅全部 dirty/untracked，禁止提交：

    target/
    dist-release/ 中含本机路径的 manifest
    workspace/
    *.dem
    *.sqlite3
    日志、截图、临时播放副本
    updater.key
    updater-password.dpapi
    token/secret
    参考项目临时 clone

全部 Gate 通过后才创建发布 commit 和 `v0.5.6` tag。不新开分支；push 最多一次、不 force。GitHub Release 上传最多一次完整尝试，附件为最终 EXE、`.sig`、`.sha256`，正文使用同源 release notes。

本轮因网络超时未确认远端 `v0.5.6` tag/Release。发布前实时执行：

    git fetch --tags origin
    git ls-remote --tags origin refs/tags/v0.5.6
    gh release view v0.5.6 --repo YuGeYu/CS2AS05 --json url,isDraft,isPrerelease,publishedAt,assets

若已存在已下载/已被 updater 提供的不同 0.5.6，禁止覆盖，改发更高 SemVer 并先请用户确认。

### 14.2 夸克由用户执行

`E:\cs2as\scripts\publish-self-update.mjs` 要求 `--quark-url` 且只接受 `https://pan.quark.cn/`。链接由用户创建并提供；不得伪造或绕过。

### 14.3 官网/R2/D1

官网仓库 `E:\cs2as` 当前分支 `codex/all-command-library`、HEAD `5eb0263`，有用户改动和 evidence，禁止清理或顺手提交。

先 dry-run（PowerShell 中以下命令写为单行，避免续行符在聊天/文档复制时丢失）：

    Set-Location E:\cs2as
    npm run self-update:publish -- --installer "E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.6_x64-setup.exe" --sig "E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.6_x64-setup.exe.sig" --version 0.5.6 --quark-url "<用户提供的夸克链接>" --title "CS2 人机增强助手 v0.5.6" --summary "Demo 播放、文件定位与解析成功后自动战报" --items "Demo 录像库新增播放和文件定位|游戏结束后只展示最新且解析成功的计分板|修复后台解析与自动弹窗竞态"

核对 key、size、SHA-256 和签名后，生产最多一次加 `--remote`。既有脚本顺序应为：R2 put -> R2 GET -> exact size/hash -> D1 disabled upsert -> feed 未提前变化 -> enable。

发布后回读：

- R2 GET size/SHA-256 等于最终 EXE。
- D1 version/key/signature/hash/size 一致。
- custom release API 返回 0.5.6 和正确夸克链接。
- Tauri feed 对 0.5.5 返回 200/0.5.6，对 0.5.6 返回 204。
- download URL GET 回读完整 EXE，不能只 HEAD。
- CORS/cache 符合 Worker 契约。
- 真实旧版客户端可下载、验签、被动安装并重启到 0.5.6。

官网 Worker 正常不需 deploy；确需部署最多 3 次。

## 15. 停止条件与回退

任一情况立即停止发布：

- Play 修改/删除原 Demo，或残留 managed copy/进程/busy。
- Explorer 未选中正确文件，或特殊路径触发注入/错误目录。
- CS2 running 时仍启动第二播放会话。
- 回放退出后误弹 post-match。
- 最新 Demo 未 done 就显示，或弹旧局、空玩家、0 回合、同局两次。
- parser、比分、K/D/A、Rating、BOT/observer/viewer 回归。
- 自动化失败/超时无明确结论。
- 真实 Tauri/CS2、安装升级或 updater 失败。
- 公钥解码后不一致、DPAPI 失败、签名不对应最终 EXE。
- R2 GET、D1、feed、GitHub asset 的 hash/size/signature 不一致。
- 远端已有用户可见/已下载的不同 0.5.6。
- 需要回退用户工作树才能继续。

线上失败：

1. 先将 D1 `updater_enabled=0`，必要时 `is_active=0`。
2. 保留 R2 对象/evidence，不覆盖同 key。
3. GitHub Release 已公开则标注暂停，不删除审计信息。
4. 恢复上一已验证 latest/feed；修复后发更高版本。

## 16. 实际执行 AI 最终回报格式

    结果：未完成 / 已实现 / 可发布候选 / 已正式发布

    Git：branch、HEAD、commit、tag、push、Release URL
    参考边界：commit、仅学习内容、未复制源码/资产

    Demo 操作：
    - Play IPC、CS2 args、managed copy、cleanup、原文件 hash
    - FolderSearch IPC、Explorer 参数、特殊路径实测

    赛后战报：
    - session id/kind/start/end
    - candidate ids 与选择理由
    - job stage、presentability
    - ready/present 时间、恰好一次
    - playback suppression

    自动化：逐命令 exit code、耗时、日志路径
    真实机器：Tauri、Explorer、CS2、BOT 新局、安装升级

    发布文件：
    - EXE path/size/SHA-256/Authenticode
    - .sig path/size/SHA-256
    - .sha256 path
    - manifest 字段一致性
    - release notes path

    生产：
    - GitHub assets
    - 夸克链接（用户提供）
    - R2 GET hash/size
    - D1 release/updater state
    - feed 0.5.5 -> 0.5.6、0.5.6 -> 204
    - 真实旧版 updater

    未验证/阻塞：明确列出
    回退：是否触发、当前线上状态

只有实现、自动化、真实机器、签名、安装升级、GitHub、R2/D1/feed 和旧版 updater 全部成立，才能写“0.5.6 已正式发布”。
