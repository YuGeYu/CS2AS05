# 0.5.6 Demo 播放、定位与赛后战报执行报告

日期：2026-08-05  
结果：已实现，未达到可发布候选

## 已实现

- 新增 Rust `play_demo(demo_id, root_path)`：只从 SQLite 解析 Demo ID，校验 canonical `.dem`、CS2 目录、CS2 运行状态和单一播放 busy；外部 Demo 复制到会话命名的 `game/csgo/replays/_cs2as_play_<session>.dem`，使用 `-applaunch 730 -insecure -novid +playdemo` 参数数组启动，并在真实 CS2 生命周期结束后清理程序创建的副本。
- 新增 Rust `reveal_demo_file(demo_id)`：Windows 固定使用 `explorer.exe`、`/select,`、canonical path 三个独立参数；前端不传任意路径。
- 新增 `GameSessionCoordinator`：统一识别 CS2 生命周期，区分 `live_match` 与 `demo_playback`，以全量 Demo fingerprint 绑定会话候选，等待最新候选的精确 core job 和 `presentable_report`，通过后仅发出一次对象化 `demo://report-ready`；失败发出 `demo://report-failed`，回放 session 不自动弹战报。
- `open_scoreboard` 现在验证数据库 `done`、当前 schema/adapter/metrics、core job `done`、有效回合、参赛玩家和 complete/partial scoreboard 后才接受请求。
- 录像库加入 Lucide `Play`、`FolderSearch`、逐行 busy、禁用状态、tooltip、ARIA 名称和稳定操作列；AppShell 对 post-match session 去重。
- 新增 `docs/release-notes-0.5.6.md`，并在 `NOTICE.md` 记录 `CS2-insight-agent@17d2a213ee8c32608feee3c63f0b6d05eef8f945` 仅为 PolyForm Noncommercial 行为参考，未复制源码、测试、样式或资产。

## 自动化证据

证据目录：`workspace/release-evidence/0.5.6-demo-actions-postmatch-20260805-180811/`

- `npm run workspace:check`：0
- `npm run typecheck`：0（此前已单独完成）
- `npm run lint`：0，oxlint 0 warning / 0 error
- `npm test -- --pool=threads --maxWorkers=1`：首次整套运行 127 tests，125 passed，2 failed；失败是两条旧源码契约断言仍期待旧的 `report.metrics_version` 字符串和 `report_is_presentable` 名称，已在源码中同步更新为 `presentable_report`，但遵守本轮“同一验收最多一次”约束，没有重跑。
- `npm run build:web`：0
- `cargo fmt --manifest-path .\\src-tauri\\Cargo.toml -- --check`：0
- `cargo check --manifest-path .\\src-tauri\\Cargo.toml`：工具 124 秒上限超时，没有编译诊断；没有将其记为通过。
- `git diff --check`：0

## 发布阻塞

- 尚未重新构建本次功能之后的 NSIS 安装器，因此 `dist-release` 中已有的旧 0.5.6 EXE、`.sig` 和 manifest 不能使用。
- Rust 编译 Gate 未完成；Vitest 断言虽已修正但没有二次证据。
- 尚未做真实 Windows Explorer 特殊路径、真实 CS2 Demo 播放、原文件 hash 前后比较、BOT 三回合新局、回放 suppression、干净安装、0.5.5 覆盖升级和旧版 updater 验收。
- BotVision 上游再分发许可证人工复核、Windows Authenticode（当前为 `NotSigned`）和公开发布仍未完成。
- 没有 commit、tag、push、GitHub Release、夸克、R2、D1 或 updater feed 变更。

## 交付边界

当前应标记为“已实现，待验证”，不是“可发布候选”或“0.5.6 已正式发布”。在 Rust 编译和自动化复核、真实机器验收及签名/安装 Gate 全部成立前，保留旧发布产物和脏工作树，不执行任何公开发布。
