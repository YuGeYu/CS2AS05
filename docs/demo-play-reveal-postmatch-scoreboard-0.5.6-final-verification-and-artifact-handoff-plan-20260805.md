# 0.5.6 最终验证与发布文件准备交接方案

> 交接对象：下一位【实际执行 AI】。两个 AI 不共享上下文；请将本文视为独立执行手册。
>
> 本文是收尾方案，不是发布授权。本轮只准备验证路径、证据格式和发布文件边界；不构建、不安装、不上传、不部署、不提交、不打 tag、不 push、不创建 GitHub Release，也不写 R2/D1/feed。

## 1. 目标和状态定义

当前目标是把已经实现的 Demo 播放、文件定位和赛后最新战报功能验证到“可发布候选”所需的证据门槛，并在本地准备一套彼此一致的 0.5.6 文件。最终执行 AI 只能在所有 Gate 通过后报告“可发布候选”；本轮结束状态保持“已实现，待验证”。

状态不得混用：

- **已实现**：源码和静态/自动化检查显示功能存在。
- **可发布候选**：自动化、Rust、真实 Windows/Tauri/CS2、安装升级、签名和产物一致性全部通过，但尚未公开发布。
- **正式发布**：另需用户授权后的 commit/tag/push、GitHub Release、夸克、R2/D1/feed 和旧版 updater 端到端证据。本任务明确不做这些动作。

## 2. 当前基线和不可回退规则

工作目录为 `E:\CS2AS05`，当前分支 `main`，调查时 HEAD 为 `8552b554993fec66866d2133d315342be3e0cbca`（`fix: unblock 0.5.5 scoreboard and recovery flows`）。恢复执行时必须先重新读取：

```powershell
Set-Location E:\CS2AS05
git status --short --branch
git rev-parse HEAD
git diff --stat
git diff --check
```

工作树包含用户已有的 Demo 平台、BotVision、Rating、viewer、preflight、资源和文档改动。逐项审阅 `git status --short` 后再决定 0.5.6 提交范围；不得使用 `git reset --hard`、`git checkout --`、`git clean`，不得删除历史 evidence。`workspace\release-evidence\` 只存本地证据，不提交。

当前自有版本字段已为 `0.5.6` 的文件包括 `package.json`、`package-lock.json`、`src-tauri\Cargo.toml`、`src-tauri\Cargo.lock` 和 `src-tauri\tauri.conf.json`。不要全局替换 `0.5.5`，也不要改第三方依赖版本。

## 3. 已实现范围（作为验收基线）

### 3.1 Demo 播放和文件定位

`src-tauri\src\demo\playback.rs` 提供 `play_demo(demo_id, root_path)`：Demo 路径只由 SQLite 的 ID 解析，校验 canonical `.dem`、CS2 根目录和进程；CS2 已运行返回 `DEMO_PLAYBACK_CS2_RUNNING`，不启动第二实例。外部 Demo 复制到 `game\csgo\replays\_cs2as_play_<session>.dem`，使用独立参数数组 `-applaunch 730 -insecure -novid +playdemo <relative path>`，监视真实 `cs2.exe` 生命周期并只清理本次临时副本。

`reveal_demo_file(demo_id)` 固定调用 `explorer.exe`、`/select,`、canonical path 三个独立参数。前端不得传任意路径，不得使用 `cmd /c start`、字符串命令或浏览器文件系统 API；缺失文件返回 `DEMO_FILE_NOT_FOUND`。

### 3.2 赛后最新战报

`src-tauri\src\demo\post_match.rs` 的 `GameSessionCoordinator` 统一观察 `false -> true -> false`，区分 `live_match` 和 `demo_playback`。完整 fingerprint 用于选择本 session 新增或变化的唯一最新 Demo；等待精确候选的 core job 完成并通过 `presentable_report` 后才发送对象化 `demo://report-ready`。回放 session 必须 suppression，不自动弹战报；失败发送 `demo://report-failed`，不显示空计分板。

自动展示契约同时要求：`demo_files.status=done`、core job `stage=done` 且已提交、schema/adapter/metrics 与当前常量一致、`summary.demo_file_id` 精确等于候选、正式回合数大于 0、至少一个 `participantRole=player`，且 `scoreboard_status` 为 `complete` 或允许的 `partial`。`open_scoreboard` 也必须复用此门禁。AppShell 按 `sessionId` 去重。

### 3.3 UI

`DemoReviewView.vue`、`src\services\tauri\demo.ts`、`src\stores\demo.ts`、`src\types\demo.ts` 和 `src\styles\main.css` 已加入 Lucide Play/FolderSearch、逐行 busy、禁用态、tooltip、ARIA 名称和窄窗口表格横向滚动。保持现有高密度桌面工具布局，不新增营销式卡片或浏览器文件操作。

## 4. 多次加时赛的比分口径（必须写入验收结论）

理论上可以正确解析多次加时赛，前提是 Demo 包含完整且已结束的所有加时回合，并且执行 AI 用真实样本验证。当前代码的可信口径是：按逻辑回合中已完成的 canonical winner 逐回合累计 CT/T；`CCSTeam.m_scoreOvertime` 只作为辅助输入，不能覆盖规范化回合累计。这样可避免边序反转导致的 `TEAM_PROPS_SIDE_MISMATCH`，也不会把未完成的最后一段加时计入比分。

验收必须至少包含一份多次加时或构造 fixture，记录：总完成回合数、每段加时的回合范围、CT/T 累计、`summary` 分数、`data_quality.canonical_*`、`unfinished_rounds` 和警告。若官方 Demo 的队伍标签与 canonical winners 不一致，应保留 warning，并以 canonical winner 结果为准；不得为了让属性相等而交换最终分数。未能取得多次加时样本时，只能报告“算法具备路径、真实多次加时未验证”，不能声称已证明。

## 5. 最终执行顺序与 Gate

### Gate A：源码和自动化

先人工确认 `tests\demo-actions-postmatch.spec.ts` 与 `tests\release-blocker-recovery.spec.ts` 已从旧的 `report.metrics_version`/`report_is_presentable` 断言同步到 `presentable_report`，然后在获得新的验收窗口后**只完整运行一次**：

```powershell
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
```

每条命令记录开始/结束时间、exit code、stdout/stderr 和工具超时。首次旧证据为 Vitest `127 tests / 125 passed / 2 failed`，Cargo check 在 124 秒超时；不得把它们写成通过。只有 Vitest `0 failed`、Rust 三项有明确成功诊断，Gate A 才通过。

### Gate B：真实 Windows/Tauri/CS2

1. 在包含空格、中文、逗号和 `&` 的目录中点 FolderSearch，确认 Explorer 正确选中文件；删除/移动后只提示 `DEMO_FILE_NOT_FOUND`。
2. 记录原 Demo size、mtime、SHA-256；CS2 关闭时点 Play，确认 `-insecure` 真正进入目标 Demo。退出后无 `_cs2as_play_*.dem`、残留进程或 busy，原文件 fingerprint 完全不变；CS2 已运行时再次 Play 必须拒绝。
3. 用真实 BOT 新局至少完成 3 个正式回合，记录 session、candidate、fingerprint、job stage、ready/present 时间。解析期间计分板隐藏；精确最新 Demo done 且报告可展示后只弹一次，地图、比分、回合和玩家 K/D/A 对应本局。
4. 播放旧 Demo 并退出，确认 `demo_playback` 不触发赛后战报。用失败、损坏 JSON、旧版本、0 回合、空玩家和 unavailable scoreboard 样本确认只通知失败、不显示空窗口。
5. 使用多次加时样本按第 4 节记录比分；若只能验证普通局，明确标记加时 Gate 未通过。

### Gate C：安装和升级

用新构建候选做干净安装、0.5.5 覆盖升级、卸载/重装；确认设置、CS2 root、Demo SQLite 历史、插件选择、快捷方式、隐藏 scoreboard 和 0.5.6 产品版本。使用正式旧版客户端验证 updater 被动安装、`.sig` 验签和重启；本地生成签名文件不等于端到端通过。

## 6. 旧产物隔离和新产物生成（仅在 Gate A/B/C 通过后）

以下调查时旧文件不能发布，不能因同名复用：

```text
E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\CS2人机增强助手_0.5.6_x64-setup.exe
E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\CS2人机增强助手_0.5.6_x64-setup.exe.sig
E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\updater-prod.json
E:\CS2AS05\dist-release\cs2-bot-improver\0.5.6\SHA256SUMS.txt
```

旧 EXE 调查 SHA-256 为 `1AC9E290E6ABDB7272A42772A6F7BBE06A83D83790315D6D298F2B85D7CD18EA`，仅作识别。不得删除旧文件；执行时将其移动/复制到带时间戳的 evidence backup，并从新构建输入中隔离。

发布密钥仅允许本机临时使用：

```text
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key.pub
C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater-password.dpapi
```

只输出内嵌公钥与磁盘 `.pub` 解码后一致的布尔结果；绝不打印、提交或复制私钥、密码、token。构建时只在当前 PowerShell 进程注入 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`，并在 `finally` 清理环境变量。Tauri `.sig` 与 Windows Authenticode 分开记录；当前 Authenticode 为 `NotSigned`，没有证书就如实保留该状态。

构建命令由执行 AI 按项目脚本实际定义运行（通常为 `npm run bundle:desktop`，随后生成 manifest），但本轮不得运行。新 EXE、`.sig`、SHA-256 清单和 manifest 必须来自同一次最终构建；manifest 不得包含本机绝对路径，公开 URL 需由生成器产生，不能手改后冒充一致。

## 7. 必须准备的文件清单

| 文件 | 本地用途 | 约束 |
|---|---|---|
| `CS2人机增强助手_0.5.6_x64-setup.exe` | 安装器候选 | 最终源码重建，记录 size/SHA-256/mtime |
| `CS2人机增强助手_0.5.6_x64-setup.exe.sig` | Tauri updater 签名 | 必须对应上述 EXE；不能复用旧 `.sig` |
| `CS2人机增强助手_0.5.6_x64-setup.exe.sha256` 或 `SHA256SUMS.txt` | 用户校验 | hash 必须逐字匹配最终 EXE |
| `updater-prod.json` | 本地 feed 候选 | version `0.5.6`、prod、project `cs2-bot-improver`；不得含本机绝对路径 |
| `docs/release-notes-0.5.6.md` | 同源发布说明 | 只写已实际验证内容和已知限制 |
| `LICENSE`、`NOTICE.md` | 合规交付 | 保留 AGPL、第三方归属和 PolyForm 行为参考边界 |
| `v0.5.6` 对应源码归档 | 源码供给 | 仅准备归档/待授权 tag；本轮不创建 tag |

安装器 bundle 还必须包含现有 `CS2BotImprover.zip`、BotVision、地图资源和对应第三方许可证；BotVision 上游再分发条款需人工复核后才可进入可发布候选。

## 8. 产物一致性检查表

执行 AI 必须保存：EXE、`.sig`、清单和 manifest 的 size/mtime/SHA-256；manifest 的 version/channel/projectId、size、hash、signature、`pub_date`；安装后产品版本；`Get-AuthenticodeSignature` 独立结果。任何一项不一致立即停止，不覆盖旧 evidence，不上传。

## 9. 本轮明确禁止的动作

不要运行 npm/Vitest/Cargo 验收、`npm run bundle:desktop`、真实 CS2/Explorer、安装升级、`git commit/tag/push`、`gh release`、夸克、R2/D1/feed、官网 deploy 或任何密钥轮换。不要删除旧候选、历史 evidence、用户脏改动或参考项目记录。

## 10. 停止条件和回报格式

任一原 Demo 被修改、临时副本残留、CS2 重复启动、回放误弹战报、未完成解析即显示、弹出旧局/空表、多次加时比分无法解释、Rust/前端失败、签名/hash/manifest 不一致、安装升级失败或出现未知远端 0.5.6，立即停止在“未完成/已实现”，不要发布。

实际执行 AI 回报必须包含：结果等级；branch/HEAD；自动化逐命令 exit code 和证据路径；Play/FolderSearch 实测及原文件 fingerprint；live_match 与 demo_playback session、candidate、job、ready/present；多次加时比分及完成回合；安装/升级；EXE/.sig/.sha256/manifest 的路径、size、hash、签名状态；未验证项和阻塞原因。只有全部 Gate 成立才可写“可发布候选”。

## 11. 当前结论

功能已实现，但现有 Vitest 有历史两项失败证据且未重跑，Cargo check 超时，真实 Explorer/CS2/BOT/加时/回放 suppression/安装升级/签名尚无完成证据；旧 0.5.6 产物明确不可发布。因此本轮只新增本交接方案，保持项目为“已实现，待验证”，不生成或发布新的安装器。
