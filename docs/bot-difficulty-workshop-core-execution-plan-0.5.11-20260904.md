# 0.5.11 人机强度工坊核心实现交接方案

日期：2026-09-04  
工作区：`E:\CS2AS05`  
交接对象：下一位实际执行 AI  
方案角色：方案制定 AI 调查后的可执行实施方案  
版本边界：继续使用现有 `0.5.11`，不升版本号、不新开分支、不回退无关改动

## 1. 目标与完成定义

把玩家过去需要外部 VPKEdit 完成的流程内置到助手：从 Low、Medium、High 或已有自定义档案读取 `botprofile.vpk` 内的 `botprofile.db`，编辑受控参数，另存为自定义档案，经过 VPK round-trip 与 entry manifest 校验后，安全激活到 `game/csgo/overrides/botprofile.vpk`。

本任务只影响助手启动的本地 BOT/`-insecure` 场景。不得修改 Online 模式的 `gameinfo.gi`、在线启动参数或官方插件包行为。内置档案是只读基线，自定义档案必须存放在 app-local 数据目录，安装升级、卸载和切换模式不得静默删除它们。

只有以下证据全部成立，才能将报告写成“核心功能实现完成，待真实 BOT 行为验收”：

1. 四份 VPK（Low、Medium、High、active）均能提取并重新打包。
2. candidate 二次提取后的 DB bytes 等于预期，非 DB entry 的路径、大小和 SHA 全部不变。
3. 自定义档案保存后重启助手仍能读取同一 DB/VPK SHA。
4. active VPK 原子替换后回读 SHA 与 profile SHA 一致，失败可回滚。
5. 安装升级保留自定义 profile、备份和 active 选择的自动化测试通过。

真实 CS2 BOT 场景未由用户完成前，绝不能写“真实 BOT 验收通过”。

## 2. 当前已确认基线

执行 AI 开始前必须重读以下文件，不得直接套用旧报告中的状态：

- `docs/bot-difficulty-workshop-execution-report-0.5.11-20260902.md`
- `docs/bot-difficulty-workshop-core-completion-handoff-plan-0.5.11-20260902.md`
- `src-tauri/src/services/bot_difficulty.rs`
- `src-tauri/src/models/bot_difficulty.rs`
- `src-tauri/src/commands/bot_difficulty.rs`
- `src/components/BotDifficultyWorkbench.vue`
- `src/services/tauri/bot-difficulty.ts`
- `src-tauri/src/services/panel.rs`
- `src-tauri/src/services/cs2.rs`

已确认事实：

- VPKEdit 来源为 `craftablescience/VPKEdit v5.0.0.4`。
- release archive SHA-256：`d9ceaf3f16aea17c06e3be79da93c55a597af1487eed8a1b42dada1ea8d54503`，大小 `32396298` bytes。
- 纳入的 CLI：`src-tauri/binaries/vpkeditcli-x86_64-pc-windows-msvc.exe`，大小 `2107392` bytes，SHA-256：`df354e590d157abd633b4a047591363f17e066486e0f59184ff71c51a81b582a`。
- `src-tauri/tauri.conf.json` 已包含 `externalBin: ["binaries/vpkeditcli"]`，许可证和 provenance 已纳入项目。
- 干净目录运行 `--help` 退出码为 0。
- Low、Medium、High、active 四份 VPK 的 `--file-tree` 均成功；唯一实际目标条目为根级 `botprofile.db`。旧方案中的 `db` 是简写，执行时必须以实际 CLI 输出和 entry path 为准。
- `--output` 要求目标输出目录事先存在。此前 round-trip 探针在候选输出目录阶段失败，因此当前仍没有任何写入成功证据。
- 已通过：`npm run typecheck`、`cargo check --manifest-path .\src-tauri\Cargo.toml`、`npm run build:web`；先前全量 Vitest 为 48 文件/169 测试通过。本轮工坊单测 fork worker 超时，不计为通过。

当前代码仍是只读骨架，且有必须先修的错误：`TOOL_VERSION` 仍为 `VPKEdit CLI v5.0.0-beta.4`；工具状态只检查文件存在而非 hash/self-test；内置 profile 的 `open` 路径仍错误返回 tool missing；组件只在 `onMounted` 加载，关闭后再次打开不会刷新。

## 3. 执行顺序与硬门禁

### 阶段 A：工具状态与只读原语

先不开放 UI 写按钮。实现并单测：

- `inspect_vpk_tool()`：定位 Tauri resource/external binary 的真实运行时路径，校验 EXE SHA、依赖闭包和 `--help` 自检；返回 `missing | invalid | ready`，版本固定为 `5.0.0.4`。
- `list_vpk_entries(input)`：调用 CLI 并严格解析 file-tree；只接受普通文件、拒绝绝对路径、`..`、reparse point 和重复大小写变体。
- `extract_db(input, workspace)`：先创建 workspace、extract output 目录，再执行 CLI；只接受恰好一个 `botprofile.db`，不得把任意包含 `db` 的文本当目标。
- `entry_manifest(input)`：为所有 entry 记录规范路径、大小、SHA；manifest digest 使用稳定排序。

CLI 调用必须使用 `Command` 参数数组，禁止 shell 拼接；工作目录为随机 app-local workspace；Windows 隐藏窗口；stdout/stderr 各限制 64 KiB；30 秒超时后 kill 并 wait；日志只记录动作、路径摘要、SHA、退出码和耗时，不记录完整 DB。

### 阶段 B：先完成无修改 round-trip

四份输入逐一执行，不得只测 Medium：

1. 创建唯一临时目录，并显式 `New-Item -ItemType Directory`/Rust `create_dir_all` 创建 CLI 的 `--output` 目录。
2. 提取唯一 `botprofile.db`，保持原始 bytes。
3. 对 candidate 执行“移除旧 entry + 添加原 bytes”流程；参数顺序必须以实际 `vpkeditcli --help` 和一次最小实验确定。
4. 重新运行 file-tree、checksum（若格式支持）和 DB 提取。
5. 比较 DB bytes；比较全部非 DB entry 的路径、大小和 SHA；允许容器 VPK SHA 改变，不允许内容改变。

任一文件失败立即停止，不写游戏目录、不开放保存按钮。证据放入 `artifacts/bot-difficulty-workshop-0.5.11-20260904/roundtrip/<profile>/`，包括命令摘要、输入/输出 SHA、manifest 和退出码。只能在四份全部通过后进入阶段 C。

### 阶段 C：行保真 DB 编辑模型

不要使用 `HashMap<String, f64>` 或完整反序列化后重新序列化。后端保留原始 bytes，并把每行建模为：空行、注释、section、key/value、未知原文。key/value 必须保存 section path、重复 occurrence、缩进、键和值的 byte span、分隔空白、行尾注释和 newline bytes。

编辑请求只允许 `{nodeId, expectedOldToken, newToken}`。后端再次读取当前 source SHA；与打开时不一致返回 `[BOT_WORKSHOP_SOURCE_CHANGED]`。未修改内容逐字节复用，保留 CRLF/LF、BOM、尾部无换行、Tab/空格、中文和异常未知行。

首版只对已有证据支持的字段提供结构化控件：`Skill`、`Aggression`、`ReactionTime`、`AttackDelay`、`Teamwork`、`AimFocus*`、`AimfocusInterval`、`LookAngle*`。没有可靠范围的字段只能使用 token 文本编辑和严格语法校验，不得擅自断言数值越大越强。High 中疑似异常超长小数必须加入 fixture：未编辑时逐字节保留，编辑时禁止转换为 `f32/f64`。

字段字典放在 `src/features/bot-difficulty/field-catalog.ts`，说明标记 `DB 注释`、`上游文档` 或 `待实机确认`。未知 profile、Template、WeaponPreference 和重复 key 必须可见或至少可诊断、可原样导出。

### 阶段 D：自定义 profile 与保存事务

建议 app-local 目录：

```text
<app-local-data>/bot-difficulty-workshop/
  profiles/<uuid>/profile.json
  profiles/<uuid>/botprofile.db
  profiles/<uuid>/botprofile.vpk
  profiles/<uuid>/source.vpk
  profiles/<uuid>/backups/<timestamp>-botprofile.vpk
  workspace/<operation-id>/...
```

profile ID 由 Rust 使用安全 UUID 生成，前端只传 ID，不传任意文件路径或 sidecar 参数。schema 至少包含 id/name/source/baseProfileId、创建/更新时间、source/current DB/VPK SHA、tool version/EXE SHA/dependency digest、entry manifest digest、validation status 和 warnings。损坏 JSON 不得静默消失，要返回占位档案和 `[BOT_WORKSHOP_PROFILE_CORRUPT]`。

`save_bot_profile` 流程：确认 CS2 未运行，取得 profile/安装服务共用锁，校验 expected source SHA 和 edits，生成候选 DB，创建已存在的 candidate output 目录，VPKEdit 重包，执行阶段 B 的完整回读与 manifest 比较；随后写 `.tmp` 文件、flush/sync、同目录原子替换。失败保留旧 profile，返回 `[BOT_WORKSHOP_SAVE_ROLLBACK]`，不得把失败 candidate 当当前版本。成功后保留最近 5 个通过验证的备份，并让前端重新 `open` 回读，不能只改本地状态。

### 阶段 E：激活、回滚与现有难度切换并发

`activate_bot_profile` 必须：

1. CS2 运行中立即返回 `[BOT_WORKSHOP_CS2_RUNNING]`。
2. 校验 profile VPK、lastValidation、tool provenance 和 current SHA。
3. 与 `panel::set_difficulty()` 使用同一进程级锁；同时修复现有难度切换，使其运行中也不能写 active VPK。
4. 将当前 active 备份到 app-local activation backup。
5. 在 active 同目录创建随机 `.tmp`，flush/sync，回读确认临时 SHA 等于 profile SHA 后原子 replace。
6. 再读 active bytes/SHA，成功后才写 `active-profile.json`。
7. 任一步失败，恢复旧文件并回读旧 SHA；恢复失败返回 `[BOT_WORKSHOP_ACTIVE_ROLLBACK]`，保留新旧路径和摘要。

active 判定以磁盘 SHA 为权威、以 `active-profile.json` 的 profile ID 做来源解释。多个 profile 同 SHA 时只允许一个匹配 ID 的 profile 显示 active；否则显示 unknown/multiple，不得同时点亮多个档案。

### 阶段 F：安装升级保留

检查并修改 `src-tauri/src/services/cs2.rs` 的安装事务：app-local `profiles` 永远不属于 `remove_upstream_package()` 清理范围；安装前记录 custom active、磁盘 drift、profile 和备份；ZIP 覆盖内置档案后，在同一外层事务中恢复已验证 custom active。恢复失败必须让整个安装回滚，不能静默切回 Low。切换 Online/卸载时保留 app-local profiles，`clear_assistant_data` 不能把它们当普通缓存删除。

如果修改 `CS2BotImprover.zip`，必须运行 `scripts/generate-plugin-manifest.ps1`，同步 manifest、hash、fixture 和契约测试；本任务默认不改 ZIP。

## 4. 前端完成要求

更新 `BotDifficultyWorkbench.vue` 和 Tauri service/types：

- 使用 `watch(() => [props.open, props.rootPath])`，每次打开刷新，root 变化时清空旧档案；请求序号或 Abort 语义防止迟到响应覆盖新选择。
- 工具状态显示 missing/invalid/ready，禁止用固定版本字符串冒充运行时就绪。
- 内置 profile 可读取 DB；内置只读，操作为“另存为自定义”；自定义支持编辑、保存、激活、恢复、导出。
- 模态具备 Escape、焦点陷阱、打开聚焦、关闭回焦入口；错误 `role=alert`，长操作 `aria-live`。
- 1440px 为档案栏/编辑区/变更摘要三栏；1100px 以下摘要折叠；768px 以下单列；375px 不横溢出，底部操作不遮挡正文。控件沿用现有 token 和 Lucide，不新增依赖。
- 只影响 BOT、不会上传 DB 的文案必须保留；不显示未经验证的“职业 BOT 等级保证”。

## 5. 命令与错误契约

最终可按实现需要落地以下命令：

```text
get_bot_workshop_state(root_path)
list_bot_profiles(root_path)
open_bot_profile(root_path, profile_id)
create_bot_profile(root_path, base_profile_id, name)
validate_bot_profile_edit(document_id, edits)
save_bot_profile(document_id, edits)
activate_bot_profile(root_path, profile_id)
restore_bot_profile(root_path, profile_id, backup_id)
delete_bot_profile(root_path, profile_id)
export_bot_profile_db(document_id)
```

错误码至少包括：`BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID`、`BOT_WORKSHOP_TOOL_TIMEOUT`、`BOT_WORKSHOP_SOURCE_CHANGED`、`BOT_WORKSHOP_PROFILE_CORRUPT`、`BOT_WORKSHOP_PROFILE_ACTIVE`、`BOT_WORKSHOP_SAVE_ROLLBACK`、`BOT_WORKSHOP_ACTIVATION_LOCKED`、`BOT_WORKSHOP_ENTRY_MANIFEST_MISMATCH`、`BOT_WORKSHOP_VPK_ROUNDTRIP_MISMATCH`、`BOT_WORKSHOP_CS2_RUNNING`。

## 6. 测试与证据门槛

新增/扩展：

- `tests/vpk-roundtrip.spec.ts`：四份 VPK 无修改 round-trip、唯一 `botprofile.db`、DB bytes 和非 DB manifest 一致、output 目录预创建回归。
- Rust `bot_difficulty` 单测：工具三态、实际 EXE hash、路径穿越、损坏 profile、编码换行、重复 key、异常 token、source changed、超时 kill、save/activate rollback。
- UI 测试：首次打开加载、再次打开刷新、内置读取、另存自定义、dirty 关闭确认、迟到请求、ARIA、375/768/1440 布局。
- 安装测试：custom active、profile 元数据、备份在候选升级前后保持；恢复失败整体回滚。

执行命令：

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/bot-difficulty-workshop-contract.spec.ts tests/vpk-roundtrip.spec.ts tests/installer-contract.spec.ts
npm run typecheck
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml bot_difficulty panel cs2
cargo check --manifest-path .\src-tauri\Cargo.toml
```

`cargo fmt -- --check` 若仍被既有无关模块阻断，只记录基线；新增 Rust 文件必须单独格式化。Vitest worker 超时不能改写为通过，必要时单独串行执行目标测试并记录命令和结果。

证据根目录固定为 `E:\CS2AS05\artifacts\bot-difficulty-workshop-0.5.11-20260904\`，至少包含 `tool.json`、四份 round-trip manifest、输入/输出/DB SHA、profile save/reopen、activation/rollback、升级保留和测试输出。不得放入完整玩家 DB、凭据或密钥。

## 7. 真实验收边界

自动化和文件证据通过后，由用户在真实 CS2 环境执行固定场景：退出游戏后打开 Medium，另存“我的中等”，只改一个已确认字段，保存、重启助手、激活、启动 BOT；记录固定地图、BOT 数量、武器、距离、站位、启动参数、active SHA 和日志，再与原 Medium 对比。助手侧还要确认 gameinfo SHA 未变、Online 切换无 BOT SearchPath、卸载/候选升级后 profile 仍在。

没有用户真实场景记录时，执行报告只能写“核心文件链路通过，待真实 BOT 行为验收”。

## 8. 立即停止条件

遇到以下任一情况，停止向后扩展并在执行报告记录最早失败点：

- VPKEdit round-trip 改变非 DB entry、无法唯一识别 `botprofile.db` 或必须改名/删除原游戏文件才能成功。
- CLI 依赖在干净运行时不闭包、hash 与 provenance 不一致、超时进程未退出。
- DB 未修改内容被重排、编码/BOM/换行/异常 token 被静默归一化或截断。
- CS2 运行中仍可写 active，激活失败无法回滚，或并发难度切换覆盖 profile。
- 安装升级删除 custom profile 或 Online 残留 BOT 配置。
- 只有 UI、文件存在或中间 CLI 成功证据，没有二次提取、manifest、active 回读证据。

## 9. 交接输出

实际执行 AI 应修改产品代码和聚焦测试，更新 `docs/bot-difficulty-workshop-execution-report-0.5.11-20260902.md` 或新增带实际日期的执行报告，逐项记录“通过/阻断/未执行”；不得覆盖本方案中的调查基线。最终状态只能是：

- `核心功能实现完成，待真实 BOT 行为验收`；或
- `候选已通过真实 BOT 验收`；或
- `阻断：<最早失败阶段与证据>`。

不得将 sidecar 已纳入、`--help` 通过或 `--file-tree` 通过单独表述为编辑、保存、重包、激活或真实 BOT 功能已完成。
