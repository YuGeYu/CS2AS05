# 0.5.11 BOT 强度工坊核心功能完成交接方案

日期：2026-09-02  
工作区：`E:\CS2AS05`  
交接对象：实际执行 AI  
文档性质：在首版骨架基础上新增的第二阶段实施方案；不替代 `docs/bot-difficulty-workshop-plan-0.5.11-20260902.md`。  
本轮状态：只调查并新增方案，没有修改现有骨架、VPK、`gameinfo.gi`、资源包或版本号。

## 1. 任务定义与正确状态

目标是把现在只能“看见入口和档案摘要”的骨架完成为玩家真正可用的内置功能：玩家不再单独下载 VPKEdit，可以从 Low、Medium、High 创建自己的强度，修改 `botprofile.db`，生成经过 round-trip 校验的 VPK，安全设为当前 BOT 强度，并在助手升级后保留。

当前执行报告写“实现完成，待用户真实 BOT 验收”，但运行事实是核心流程尚不存在。执行 AI 开工前应把状态理解为：

> **首版发现/UI 骨架完成；编辑、保存、重包、激活和升级保留仍被 sidecar 缺失阻断。**

只有“保存后的 DB 二次提取一致、非 DB 条目不变、active VPK SHA 回读一致、升级保留自动化通过”后，才可写“核心功能实现完成，待真实 BOT 行为验收”。真实 BOT 固定场景完成前，不得写“已通过真实 BOT 验收”。

## 2. 当前代码基线（执行 AI 必须先重读）

### 2.1 已存在的骨架

- `src/views/OverviewView.vue` 已在 BOT 难度旁加入“打开强度工坊”按钮，并挂载 `BotDifficultyWorkbench`。
- `src/components/BotDifficultyWorkbench.vue` 已有模态、档案列表、active 图标、工具文案和空/错误状态。
- `src/services/tauri/bot-difficulty.ts` 只有 `list_bot_profiles` 和 `open_bot_profile`。
- `src-tauri/src/models/bot_difficulty.rs` 只有列表摘要和纯文本 document 模型。
- `src-tauri/src/services/bot_difficulty.rs` 能计算内置/active VPK SHA、读取 app-local 自定义 `profile.json`，但不能操作 VPK。
- `src-tauri/src/commands/bot_difficulty.rs`、`commands/mod.rs`、`models/mod.rs`、`services/mod.rs`、`lib.rs` 已完成两个只读命令注册。
- `tests/bot-difficulty-workshop-contract.spec.ts` 当前 2 个字符串契约测试通过。
- 当前复核 `cargo check --manifest-path .\src-tauri\Cargo.toml` 通过；仅有既有 demoparser warnings 和 `demo.rs` 的 `unused_mut`。

### 2.2 已确认的 P0 缺陷

| 当前行为 | 必须改为 | 原因 |
|---|---|---|
| `onMounted(() => { if (props.open) load() })` | `watch(() => [props.open, props.rootPath], ...)`，每次由关闭变为打开且 root 有效时加载 | 组件平时以 `open=false` 常驻，首次点击不会触发 mounted，实际可能一直空白 |
| 无论工具是否存在都返回 `toolVersion=VPKEdit 5.0.0.4` | 返回结构化 `tool.status=missing/invalid/ready`；只有运行时文件存在、EXE SHA 和自检通过才返回 ready | 当前 UI 会把“计划采用版本”误显示成“已纳入工具” |
| `TOOL_SHA256` 保存 release ZIP digest | 分开保存 `archiveSha256` 和实际 `vpkeditcli.exe`/依赖文件 SHA | ZIP 摘要不能验证运行中的 EXE |
| 内置档案点击不调用 `open_bot_profile` | ready 时内置和自定义都可打开；missing/invalid 时显示真实工具门禁 | 当前错误分支对内置档案只靠硬编码判断，不是工具探测 |
| 自定义 `profile.json` 反序列化失败被静默跳过 | 列表返回损坏档案占位及 warning，提供恢复/定位证据 | 静默消失会让玩家以为自定义档案被删除 |
| `read_to_string` 强制 UTF-8 | 保存原始 bytes、BOM、编码判定与换行元数据 | DB 可能包含非 UTF-8 名称/注释，不能静默损坏 |
| 模态没有 Escape、焦点入口、焦点回还和 focus trap | 完整 dialog 键盘行为，关闭时回到“打开强度工坊”按钮 | 当前只满足 `role=dialog`，并未完成可访问交互 |
| active 仅靠 SHA 相等推断 | active state 同时记录 profile ID/SHA；磁盘 SHA 是最终权威，元数据用于解释来源 | 两个档案相同 SHA 时仅靠布尔值会同时显示 active |

不得为了快速完成而删除现有硬阻断；应把它升级为真实的工具状态机和事务门禁。

## 3. VPKEdit sidecar 纳入方案

### 3.1 已核实的上游证据

- 项目：`craftablescience/VPKEdit`，MIT。
- 固定 release：`v5.0.0.4`。
- 源码 tag 的 `CMakeLists.txt` 声明版本 `5.0.0.4`。
- Windows standalone 资产：`VPKEdit-Windows-Standalone-msvc-Release.zip`。
- 资产大小：32,396,298 bytes。
- GitHub API digest：`sha256:d9ceaf3f16aea17c06e3be79da93c55a597af1487eed8a1b42dada1ea8d54503`。
- 官方 workflow 明确将 `vpkeditcli.exe`、`vpkedit.exe`、`CREDITS.md`、`LICENSE`、根目录 DLL 和若干 GUI 插件 DLL打入 standalone ZIP。
- CLI 源码确认支持 `--extract`、`--output`、`--add-file`、`--remove-file`、`--file-tree`、`--verify-checksums`，修改模式会从输入 pack 打开后 bake 到 output。

本次调查因 GitHub 下载速度过慢，只取得部分文件后主动停止；没有得到完整 ZIP，也没有得到 EXE 独立 SHA。执行 AI 不得把上述 ZIP digest 填入 EXE 校验常量。

### 3.2 下载、验证和依赖闭包

执行 AI 按以下顺序操作：

1. 下载固定 release 到 `workspace/vendor-cache/VPKEdit-v5.0.0.4/`；支持断点续传，最多一次完整重新下载，不循环重试。
2. 验证 ZIP 大小和 SHA-256 精确等于上述值；不一致立即停止。
3. 解压到独立 staging，记录所有文件相对路径、大小和 SHA 到 `third_party/VPKEdit-v5.0.0.4/SHA256SUMS.txt`。
4. 保存上游 `LICENSE`、`CREDITS.md`、release URL、tag、源码 commit、下载时间和 archive SHA 到 `third_party/VPKEdit-v5.0.0.4/UPSTREAM.md`。
5. 在只有 `vpkeditcli.exe` 的全新临时目录运行 `--help`/无副作用 file-tree 操作。若缺 DLL，逐一加入 CLI 实际依赖，直到在不依赖开发机 PATH/Qt 安装的干净目录运行成功。
6. 不打包 `vpkedit.exe` GUI、platform/style/preview 插件或 Qt DLL，除非进程启动证据证明 CLI 必须依赖其中某个文件。最终只纳入最小依赖闭包和许可证文件。
7. 使用 Windows `dumpbin /dependents`（若可用）、PE 导入检查和干净目录启动三项交叉确认依赖；静态检查不能替代真实启动。

### 3.3 Tauri 打包与运行时定位

优先采用 Tauri `externalBin`：

```text
src-tauri/binaries/vpkeditcli-x86_64-pc-windows-msvc.exe
src-tauri/tauri.conf.json -> bundle.externalBin: ["binaries/vpkeditcli"]
```

Tauri schema已确认会按 `binary-name-{target-triple}.exe` 查找构建输入。若 CLI 还需同目录 DLL，`externalBin` 只负责 EXE，依赖 DLL 作为 `bundle.resources` 放入专用 `resources/vpkedit/`，运行前复制/校验到 app-local `tools/vpkedit-5.0.0.4/`；不要污染应用根目录或依赖系统 PATH。

推荐后端使用 Rust `std::process::Command` 调用经 `resolve_vpk_tool()` 返回的固定绝对路径；无需把任意 shell 能力暴露给前端。必须设置：

- Windows `CREATE_NO_WINDOW`；
- `current_dir` 为隔离 workspace；
- 最小化环境，显式设置 PATH 仅包含 CLI 依赖目录和必要系统目录；
- stdout/stderr 各上限 64 KiB；
- 30 秒超时，超时后 kill 并 wait，不能遗留后台进程；
- 参数数组逐项传入，禁止拼接命令字符串；
- 日志记录 action、输入/输出 SHA、退出码、耗时和截断后的错误，不记录完整 DB 文本。

新增 `VpkToolState`：

```text
status: missing | invalid | ready
version: 5.0.0.4 | null
archiveSha256: <ZIP sha> | null
executableSha256: <EXE sha> | null
dependencyDigest: <sorted path/size/hash digest> | null
selfTest: notRun | passed | failed
message: 用户向中文状态
```

`list_bot_profiles` 必须先调用 `inspect_vpk_tool()`。EXE/依赖任一 hash 不符或 `--help`/fixture file-tree 自检失败时返回 `invalid`，所有写操作继续阻断。

## 4. 先证明 VPK 原语，再写编辑器

### 4.1 只读原语

先在 `bot_difficulty.rs` 完成：

```text
inspect_vpk_tool()
list_vpk_entries(input_vpk)
extract_single_db(input_vpk, workspace)
hash_vpk_entries(input_vpk)
```

对当前安装目录中的 Low、Medium、High 和 active 四份 VPK逐一证明：

- VPK 可打开；
- 恰好存在一个根级 `db` entry；
- 不接受 `../db`、绝对路径、多个大小写变体 DB 或 reparse point；
- 提取后 DB SHA、大小、编码、BOM、换行类型可记录；
- file tree 输出可稳定解析。若 CLI 只提供人类文本，优先在后端对行进行严格状态机解析并保存原始输出证据，不用模糊 `contains("db")`。

### 4.2 无修改 round-trip

在开放任何 UI 编辑前，先对四份 VPK做无修改 round-trip：

1. 复制输入 VPK 为 candidate source。
2. 提取 DB，不改一字节。
3. 用 `--remove-file db --add-file <db> db --output <candidate>` 创建候选；具体顺序以 `vpkeditcli --help` 和最小手工试验为准。
4. 二次列出和提取 candidate。
5. 比较 DB bytes、entry 集合、每个非 DB entry bytes/SHA；允许容器级 VPK SHA 因重排变化，但不允许内容语义或非 DB 条目变化。
6. 对 checksum：存在则必须验证通过；不存在则状态为 `not-present`，不能写 passed。
7. candidate 再次由 VPKEdit 打开成功。

四份任一失败均命中停止条件。不要只验证 Medium。

## 5. DB 行保真编辑器

### 5.1 后端文档模型

解析输入始终保留 `original_bytes`。新增 line-preserving AST：

```text
Document -> lines[]
Line -> blank | comment | section | keyValue | rawUnknown
KeyValue -> nodeId, sectionPath, occurrence, keySpan, valueSpan,
            indentBytes, separatorBytes, valueBytes, trailingCommentBytes,
            newlineBytes, parsedHint
```

`nodeId` 由 section path + key + occurrence + 原始行 SHA 派生，保存请求只传 `{nodeId, expectedOldToken, newToken}`。后端应用 optimistic concurrency：当前 source DB/VPK SHA 与打开时不一致则返回 `[BOT_WORKSHOP_SOURCE_CHANGED]`，禁止覆盖。

必须覆盖：

- CRLF/LF、BOM/无 BOM、尾部无换行；
- 重复 `WeaponPreference` 与相同 section 的重复 key；
- 行尾注释、Tab/空格缩进；
- 超长数值和多个小数点等异常 token；
- 无法识别行原样保留；
- 文件大小、单行长度、节点数上限，避免超大输入拖死 UI。

现有 High VPK 中疑似异常超长 `.9` token 是必须加入 fixture 的回归项：未编辑它时逐字节保留；用户若试图编辑该行，只有新 token 通过严格语法后才允许保存，绝不转成 `f64`。

### 5.2 参数能力与说明

不要一次性把 DB 所有值变成输入框。首版按层次提供：

- 快速调节：`ReactionTime`、`AttackDelay`、`Aggression`、`Teamwork` 等有明确语义且已确认范围的字段。
- 瞄准细节：`AimFocus*`、`LookAngle*`，只有证据充分时提供 slider/stepper；否则保持 token 文本编辑。
- 高级原文：完整只读行视图 + 受限 token 编辑，不提供无约束全文替换。
- 模板/Profile 筛选：Default、Template、Bot profile；重复项逐条显示上下文。

字段字典独立放入 `src/features/bot-difficulty/field-catalog.ts`，每项含 label、description、evidence、unit、min/max/step 或 `textTokenOnly`。不得写“数值越大一定越强”等未经实机证明的结论。

## 6. 自定义档案、保存与激活事务

### 6.1 Profile schema

升级 `profile.json` 为版本化 schema，至少包含：

```text
schema, id, name, source=custom, baseProfileId,
createdAt, updatedAt, sourceVpkSha256, sourceDbSha256,
currentVpkSha256, currentDbSha256,
toolVersion, toolExecutableSha256, dependencyDigest,
entryManifestDigest, validationStatus, warnings
```

内置档案不在 app-local 写伪 profile.json；每次从当前游戏目录读取并标记 `readOnly=true`。从内置档案编辑时，第一步是“另存为自定义档案”，使用 `crypto` 安全 UUID/后端 UUID，不允许玩家名称作为目录名。

### 6.2 保存流程

`save_bot_profile` 只保存 app-local 自定义档案，不直接写游戏目录：

1. 获取 per-profile 互斥锁，确认 CS2 未运行。
2. 校验 document token、打开时 source SHA 和 profile schema。
3. 写入 candidate DB，生成 candidate VPK。
4. 执行完整 round-trip 和 entry manifest 比较。
5. 写 `profile.json.tmp`、`botprofile.db.tmp`、`botprofile.vpk.tmp`，flush/sync 后同目录原子替换。
6. 每个 profile 保留最近 5 个通过校验的备份，按时间淘汰；失败 candidate 放 evidence 临时目录，不加入 profile 当前状态。
7. 返回实际新 DB/VPK SHA 和逐项 diff；前端收到结果后重新调用 open，不能只乐观修改本地 state。

### 6.3 激活流程

`activate_bot_profile`：

1. CS2 运行时直接返回 `[BOT_WORKSHOP_CS2_RUNNING]`；同时修复现有 `panel::set_difficulty()`，难度切换也必须阻断运行中写 active VPK。
2. 校验 profile VPK、tool provenance 和 round-trip lastValidation；过期/invalid 要重新验证。
3. 获取与 Panel 安装/难度切换共用的进程级文件锁，避免并发覆盖。
4. 备份当前 `game/csgo/overrides/botprofile.vpk` 到 app-local activation backup。
5. 同目录写 `.cs2as05-botprofile-<nonce>.tmp`，flush/sync，确认临时 SHA 等于 profile SHA，再原子 replace active。
6. 回读 active bytes/SHA；只有完全一致才更新 `active-profile.json`。
7. 失败则恢复旧 active 并回读旧 SHA；回滚失败返回 `[BOT_WORKSHOP_ACTIVE_ROLLBACK]` 并保留两个文件路径和摘要。
8. 返回更新后的 `PanelSnapshot`，其 difficulty 增加 `activeProfileId`、`activeProfileName`、`activeVpkSha256`、`source`。

当多个 profile VPK SHA 相同时，只有 `active-profile.json` 中 ID 与磁盘 SHA 同时匹配的一个档案显示 active；否则显示 `active source=unknown/matchingMultiple`，不同时点亮多个项目。

## 7. 安装、升级和恢复保留

当前 `install_game_files_transactionally()` 会用 ZIP 覆盖内置三档和 active VPK，`capture_panel_preferences()` 只保存 Low/Medium/High 文本。因此必须在修改安装器前建立以下契约：

1. app-local `profiles/` 永远不进入 `remove_upstream_package()` 范围。
2. 安装事务开始前读取 `active-profile.json`，确认 active 磁盘 SHA 与 custom profile SHA 一致；不一致则记录 drift，不自动覆盖未知用户文件。
3. ZIP 安装完成并恢复 mode/aim/nades/items 后，若原 active 是已验证 custom，则在同一外层事务中重新激活 custom。
4. custom 恢复失败时，整个插件安装回滚到写前 active VPK、内置目录和 Panel state；不能安装成功但静默切回 Low。
5. 新内置 Low/Medium/High 更新不会重写 custom profile 的 `source.vpk`；只在 UI 显示“基线已有新版本”，由玩家主动派生/迁移。
6. 手动卸载只移除游戏目录插件与 active 文件，不删除 app-local profiles；再次安装后允许玩家手动激活。
7. `clear_assistant_data` 的范围要显式决定：默认保留自定义强度档案；若产品要求删除，必须单独列出数量并二次确认，不能被普通缓存清理带走。

## 8. API 完成清单

建议最终 Tauri 命令：

```text
get_bot_workshop_state(root_path)
list_bot_profiles(root_path)
open_bot_profile(root_path, profile_id)
create_bot_profile(root_path, base_profile_id, name)
validate_bot_profile_edit(document_id, edits)
save_bot_profile(document_id, edits)
activate_bot_profile(root_path, profile_id)
restore_bot_profile(profile_id, backup_id)
delete_bot_profile(profile_id)
export_bot_profile_db(profile_id)
```

删除自定义档案时若其处于 active 状态必须阻断，先要求切换到内置档案。导出使用 Tauri dialog，由 Rust 只写用户选定文件；页面不接收任意路径字符串。

在上一版错误码基础上新增：

```text
[BOT_WORKSHOP_TOOL_DEPENDENCY_INVALID]
[BOT_WORKSHOP_TOOL_TIMEOUT]
[BOT_WORKSHOP_SOURCE_CHANGED]
[BOT_WORKSHOP_PROFILE_CORRUPT]
[BOT_WORKSHOP_PROFILE_ACTIVE]
[BOT_WORKSHOP_SAVE_ROLLBACK]
[BOT_WORKSHOP_ACTIVATION_LOCKED]
[BOT_WORKSHOP_ENTRY_MANIFEST_MISMATCH]
```

## 9. UI 完成方案

### 9.1 交互修正

| 当前骨架 | 完成后 | Why |
|---|---|---|
| 首次点击可能不加载 | 监听 open/root，每次打开刷新并取消旧请求 | 入口必须确定可用，避免空模态 |
| 工具版本始终显示 | 三态状态条：未安装、校验失败、已就绪 | 状态必须来自运行时证据 |
| 列表选中与 active 共用相近视觉 | selected 用主色边框，active 用独立“当前使用”标签 | 编辑对象和游戏使用对象是两个概念 |
| 仅空状态 | 参数搜索、区块 tabs、字段表、diff 侧栏、固定操作栏 | 支持实际编辑闭环 |
| 只有鼠标关闭 | Escape、焦点陷阱、打开聚焦标题/首控件、关闭回焦入口 | 完整 dialog 可访问性 |
| 点击切换没有竞态控制 | 请求序号/Abort 语义，迟到响应不得覆盖新选择 | 快速切换档案不会串数据 |
| 保存只有最终反馈 | 验证、重包、回读三阶段进度；失败聚焦错误摘要 | 让长操作可理解且可恢复 |

### 9.2 布局与动效

- 桌面保持左 240-260px 档案栏、中间参数区、右侧 260-300px 变更摘要；窗口不足 1100px 时摘要折叠为底部可展开区域。
- 768px 以下改为顶部档案选择 + 单列字段，不显示横向宽表；375px 下按钮文字可换行，底部主操作始终可见但不得遮挡内容。
- 参数多时使用虚拟列表或分组懒渲染，搜索输入 debounce 100-150ms；不要一次渲染数千行。
- 模态进入 180-220ms ease-out、退出 140-180ms；只动画 opacity/transform。键盘触发可立即响应，`prefers-reduced-motion` 移除位移。
- 所有保存/激活按钮至少 44px；破坏性删除为次级危险按钮并确认档案名。
- 继续使用现有 Lucide；不增加新 UI 库，除非现有表单复杂度实际需要且先证明收益。

## 10. 测试矩阵

### 10.1 Rust 与 sidecar

- tool missing、ZIP SHA 误当 EXE SHA 的回归、EXE hash 错、依赖缺失、自检失败、超时 kill、stdout/stderr 截断。
- Low/Medium/High/active 无修改 round-trip；DB 唯一性；非 DB entry manifest；checksum `passed/not-present/failed` 三态。
- 行保真：CRLF/LF、BOM、中文、重复键、注释、unknown raw、超长 High token、并发 source changed。
- profile：非法 ID、损坏 JSON、schema 迁移、备份淘汰、save rollback、activate rollback、多 profile 同 SHA。
- CS2 运行中：open/list 可读，create/save/activate/set_difficulty 均拒绝写入。
- 安装升级：custom active 恢复、drift 阻断、恢复失败使安装整体回滚、卸载保留 app-local profiles。

### 10.2 Vue/UI

- 组件初始 `open=false` 后点击可加载；关闭再打开会刷新；root 改变清空旧档案。
- 工具三态文案和按钮禁用；迟到请求不会覆盖当前选中项。
- 内置只读 -> 另存自定义 -> 编辑 -> validation -> save -> reopen -> activate 完整 mock 流程。
- dirty 关闭确认、Escape/focus trap/focus restore、错误 `role=alert`、状态 `aria-live`。
- 1440x900、1100x700、768px、375px 无溢出；浅色/深色/高对比和 reduced-motion。

### 10.3 建议命令

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/bot-difficulty-workshop-contract.spec.ts tests/vpk-roundtrip.spec.ts tests/installer-contract.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml bot_difficulty
cargo test --manifest-path .\src-tauri\Cargo.toml panel
cargo test --manifest-path .\src-tauri\Cargo.toml cs2
cargo check --manifest-path .\src-tauri\Cargo.toml
```

测试必须使用临时复制的 VPK，不直接对真实 `game/csgo/overrides` 写入。真实目录测试只在用户明确执行验收时进行，并在每次操作前后记录 SHA。

## 11. 分阶段执行顺序

### 阶段 0：修正骨架真实性

- 修复 open watcher、工具状态、ZIP/EXE SHA 混淆、损坏 profile warning、模态键盘行为。
- 增加对应聚焦测试。

### 阶段 1：纳入 sidecar 并证明只读

- 固定下载、许可证/provenance、最小依赖闭包、Tauri 打包。
- 完成 tool self-test 和四份 VPK 只读提取。

### 阶段 2：证明无修改 round-trip

- 完成 entry manifest、checksum 三态和 candidate 二次提取。
- 四份 VPK全通过后才能进入下一阶段。

### 阶段 3：完成 DB 行保真编辑与自定义保存

- AST、字段字典、diff、create/save/backup/restore。
- 保存只落 app-local，不碰 active。

### 阶段 4：完成 active 激活与 Panel 集成

- 文件锁、原子 replace、回读、回滚、PanelSnapshot 自定义状态。
- 同时修复运行中 `set_difficulty` 写入问题。

### 阶段 5：安装升级保留

- custom capture/restore/drift/rollback/卸载保留。
- 覆盖安装 fixture 与一次真实候选覆盖安装验证。

### 阶段 6：UI/真实验收与报告

- 多尺寸、主题、键盘、竞态、错误态截图。
- 用户执行固定场景 BOT 对比；执行 AI保存日志、SHA 和结果。

每完成一个阶段就更新执行报告和证据，不要等全部结束后凭记忆补写。

## 12. 真实 BOT 验收与证据

继续使用：

```text
E:\CS2AS05\artifacts\bot-difficulty-workshop-0.5.11-20260902\
```

至少保存：

- `tool-provenance.json`：release、archive/EXE/dependency SHA、许可证、CLI help 摘要。
- `roundtrip/<profile>/manifest-before.json`、`manifest-after.json`、DB SHA、CLI stdout/stderr。
- `profile-save.json`：node diff、旧/新 SHA、备份路径、回读结果，不含完整 DB。
- `activation.json`：旧 active SHA、新 profile SHA、临时 SHA、最终回读、回滚状态。
- `upgrade-preservation.json`：安装前/后 profile、active 和 Panel snapshot。
- UI 截图：1440x900、1100x700、768px、375px，浅/深主题至少各一组。
- BOT 场景：同地图、武器、距离、BOT 数量、站位，比较内置 Medium 与只改一个字段的自定义档案；记录首次发现、首次开火、瞄准稳定性和日志。
- Online 恢复：官方 gameinfo SHA、无 overrides/metamod SearchPath、在线启动参数不含 `-insecure`。

UI 显示“保存成功”不是成功证据；candidate 能打开也不是激活成功证据；聊天框显示难度也不是 BOT 行为成功证据。最终以磁盘 SHA、round-trip、进程实际加载和固定场景为准。

## 13. 停止条件

命中任一项立即停止，不得继续发布/打包：

- 完整 release ZIP SHA 不符，或 EXE/依赖 SHA 未单独记录；
- CLI 不能在最小依赖目录或最终安装目录运行；
- 任一内置/active VPK 无修改 round-trip 改变 DB bytes 或非 DB entry；
- checksum failed、DB 不唯一、出现路径穿越/reparse point；
- 行保真测试改变未编辑行、注释、编码、换行或异常 token；
- CS2 运行中可以保存/激活/切换 active VPK；
- active 回读不一致或回滚失败；
- 安装升级覆盖/丢失 custom profile，或 drift 时静默回退 Low；
- Online gameinfo/启动参数被本功能改变；
- 只有自动化/UI 证据，没有真实 BOT 加载与固定场景证据，却准备写“已验收”。

## 14. 交付物和最终口径

执行 AI 完成后应更新（不是再新增同名报告）：

- `docs/bot-difficulty-workshop-execution-report-0.5.11-20260902.md`：把“实现完成”纠正为与证据一致的状态，并逐阶段记录。
- `NOTICE.md`：VPKEdit MIT、版本、来源、CLI/依赖再分发说明。
- `third_party/VPKEdit-v5.0.0.4/UPSTREAM.md`、`LICENSE`、`CREDITS.md`、`SHA256SUMS.txt`。
- 上述 artifacts 证据目录。
- 若改 `CS2BotImprover.zip`，仍须运行 `scripts/generate-plugin-manifest.ps1` 并同步 `CUSTOM_ZIP_SHA256`、fixture 和 ZIP 契约；但 VPKEdit sidecar 本身优先独立于游戏插件 ZIP 打包。

最终只允许使用以下状态：

- `首版骨架完成，核心功能阻断`：sidecar/编辑/重包/激活任一未完成；
- `核心功能实现完成，待真实 BOT 行为验收`：round-trip、保存、active、升级保留和 UI 自动化均通过；
- `候选已通过真实 BOT 验收`：再加真实加载、固定场景和 Online 恢复证据；
- `阻断`：命中停止条件，写清最早失败阶段、输入 SHA 和保留的恢复路径。
