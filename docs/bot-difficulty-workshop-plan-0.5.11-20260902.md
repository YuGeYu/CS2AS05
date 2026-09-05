# 0.5.11 BOT 强度工坊内置功能交接方案

日期：2026-09-02  
工作区：`E:\CS2AS05`  
交接对象：实际执行 AI  
方案状态：调查完成，待实现；本轮未修改产品代码、资源 ZIP、版本号或玩家 CS2 文件。

## 1. 目标与边界

玩家当前需要自行下载 VPKEdit，再手工完成“打开 `overrides/<难度>/botprofile.vpk`、提取 `db`、编辑、放回 VPK、删除旧 `db`”。0.5.11 要把这条流程内置到助手，使玩家可以选择已有难度、编辑 `botprofile.db`、另存为自定义强度，并在启动 BOT 前安全激活。

必须遵守：

- 只服务助手启动的本地 BOT/`-insecure` 场景；绝不修改 Online 模式的官方 `gameinfo.gi` 或在线启动参数。
- 只修改 VPK 中的 `db` 条目；不得删除、替换或重排其他资源条目。
- 不自行实现 VPK 二进制格式；固定使用经过哈希校验的 VPKEdit CLI sidecar 解包与重包。
- 内置 Low/Medium/High 是只读基线，玩家编辑生成独立档案；安装、升级、恢复默认不能覆盖玩家档案。
- 不臆测“职业 BOT 参数”或字段含义。只有当前上游注释/文档和实机证据支持时才提供中文说明；未知字段显示原名、原值和来源注释。
- 当前工作树有大量既有改动。执行 AI 不得 reset、clean、checkout 或回退无关修改。

## 2. 已调查事实

### 2.1 当前项目结构

项目是 Tauri 2 + Vue 3 + Rust + SQLite 桌面应用。相关入口：

- `src/views/OverviewView.vue`：当前“BOT 难度”控件调用 `usePanelStore().setDifficulty()`。
- `src/stores/panel.ts`、`src/features/panel/types.ts`、`src/services/tauri/panel.ts`：前端状态与 Tauri API。
- `src-tauri/src/commands/panel.rs`：已有 `set_panel_difficulty`。
- `src-tauri/src/services/panel.rs`：已有 `disk_difficulty()`、`write_difficulty_at()`、`set_difficulty()` 和事务/原子写入辅助函数。
- `src-tauri/src/services/cs2.rs`：安装服务。`remove_upstream_package()` 会移除 `overrides/Low|Medium|High` 和 active `overrides/botprofile.vpk`，所以安装升级必须显式保留玩家自定义层。
- `src-tauri/resources/CS2BotImprover.zip` 当前包含 Low/Medium/High 三档 VPK 和 active `overrides/botprofile.vpk`。

当前 `PanelSnapshot.difficulty` 只有 `current` 与 `available`，不能表达自定义档案、解析错误、dirty 状态或 active SHA；必须扩展后端协议，不能让前端猜测磁盘状态。

### 2.2 VPK/DB 事实

当前内置 VPK 均为 VPK v2 头（开头 `34 12 AA 55 02 00 ...`），可见唯一目标条目扩展名为 `db`。当前资源：

| 档案 | 大小 | SHA-256 |
|---|---:|---|
| Low | 115,353 bytes | `5EC7F50BCD97E3678C7545306407110E7E87C6E09CBC604DA70CFA56A612206A` |
| Medium | 406,110 bytes | `495FCAE5DFAABB1B47CEDF636FD48E906085823C09BEC4C6D5589A33BEBEF695` |
| High | 115,601 bytes | `EA7D7FC6DEF8E0342AA817F64A7E9BC3844E3A6372F1567C3C60C74F6AECE233` |

DB 是纯文本 KeyValues 风格文件，包含 `Default`、`Template ...`、Bot profile、注释和重复字段。可见字段包括 `Skill`、`Aggression`、`ReactionTime`、`AttackDelay`、`Teamwork`、`AimFocusInitial`、`AimFocusDecay`、`AimFocusOffsetScale`、`AimfocusInterval` 和六个 `LookAngle*` 字段。

当前 High 样本出现超长且疑似非法的小数文本（重复 `.9`）。因此不能把整个 DB 解析成 `f32/f64` 表单后再序列化；必须保留原文，并给异常值提供“原文编辑 + 语法门禁”路径。

### 2.3 上游与工具事实

- `ed0ard/CS2-Bot-Improver` 调查时 `main` 为 `b102704bd5d97d45bb66fb6bd94ff0f606b87302`，公开 `overrides/{Low,Medium,High}/botprofile.db`。调查时大小约为 Low 116,319、Medium 407,897、High 116,079 bytes；这是来源对账证据，不可直接覆盖本地 VPK。
- Medium 是上游主要混合档案并含职业 profile；产品文案应写“中 · 上游混合基线”，不能保证等同职业玩家水平。
- `craftablescience/VPKEdit` 为 MIT 项目；调查时最新 release 是 `v5.0.0.4`。Windows standalone 资产 `VPKEdit-Windows-Standalone-msvc-Release.zip` 的 GitHub API SHA-256 为 `d9ceaf3f16aea17c06e3be79da93c55a597af1487eed8a1b42dada1ea8d54503`。
- VPKEdit 提供无外部依赖的 `vpkeditcli`，支持 `--extract`、`--output`、`--add-file`、`--remove-file`、`--file-tree`、`--verify-checksums` 和 VPK 创建/修改。
- VPKEdit README 提示主要目标为 Source 1。执行 AI 纳入资源前必须重新核验 release、SHA、许可证、CLI 文件名，并用当前四个 VPK做解包/重包回读。若失败，停止，不得自制写入器或把 `.db` 改名成 `.vpk`。

## 3. 产品与 UI 方案

### 3.1 入口和信息架构

在 `OverviewView.vue` 的 BOT 难度控制组旁增加“打开强度工坊”图标按钮，使用 Lucide `SlidersHorizontal` 或 `Wrench`，带 tooltip 和 `aria-label`。打开独立的 `BotDifficultyWorkbench.vue` 大尺寸模态/抽屉；首版不增加侧栏导航，避免让维护工具挤占主流程。

工坊包含：

1. **档案栏**：内置 Low/Medium/High、自定义档案、当前 active 标记、来源和 VPK SHA 前 12 位。
2. **DB 编辑区**：字段搜索、区块筛选、结构化字段表；每行显示字段名、原始值、当前值、可信中文说明和来源注释。未知字段不能隐藏。
3. **变更摘要**：修改行数、原始/当前 DB SHA、解析警告、最近保存时间。
4. **操作区**：`保存为自定义档案`、`覆盖当前自定义档案`、`恢复原始版本`、`设为当前 BOT 强度`、`导出 DB`。内置档案只读。
5. **状态区**：解包、解析、待保存、重包、校验、失败/回滚。操作超过 300ms 必须显示 spinner/状态，事务期间禁用冲突按钮。

推荐标题“人机强度工坊”；自定义示例“我的稳健中等”“练枪极限”。固定提示：“只影响 BOT 模式，在线模式不会使用这些文件。”

### 3.2 响应式、交互与无障碍

- 1440px：左侧档案栏约 260px，右侧是可滚动参数表，不使用嵌套卡片。
- 768px 以下：档案栏变为顶部 tabs/下拉，参数行变为字段/值两列；禁止横向溢出。
- 375px：按钮换行，长 SHA/路径截断并提供 tooltip；图标按钮至少 44x44px。
- 所有输入有显式 label；搜索披露使用动态 `aria-expanded`；错误 `role=alert`，保存结果 `aria-live=polite`；Tab 顺序与视觉顺序一致。
- 延续现有主题 token 和蓝/青语义色，不新增紫色渐变或装饰光球。危险操作使用红色并二次确认。
- 动效只用 150-250ms opacity/transform transition，尊重 `prefers-reduced-motion`；不对参数表使用重排动画。
- 已证实且有合理范围的数值使用滑杆 + 步进器；未知/无范围证据字段使用文本 token 编辑。禁止用一大片裸 `number` 输入框替代上游能力。

## 4. 数据与文件契约

### 4.1 玩家工作区

不使用 `localStorage`。通过 Tauri `app_local_data_dir()` 保存：

```text
<app-local-data>/bot-difficulty-workshop/
  profiles/<uuid>/profile.json
  profiles/<uuid>/botprofile.db
  profiles/<uuid>/botprofile.vpk
  profiles/<uuid>/source.vpk
  profiles/<uuid>/backups/<timestamp>-botprofile.vpk
  workspace/<operation-id>/extract/db
  manifest.json
```

`profile.json` 至少包含：schema、id、name、source (`builtin`/`custom`)、baseDifficulty、createdAt、updatedAt、dbSha256、vpkSha256、vpkToolVersion、vpkToolSha256、entryPath、active、lastValidation、warnings。目录由 Rust 生成；前端只能传 profile ID 和受限字段，不能传任意路径。

### 4.2 DB 行保真模型

不要用 `HashMap<String, f64>` 重建全文。实现 line-preserving AST：

- 节点类型：comment、blank、section header、key/value、unknown raw line。
- 每个 key/value 保存 section、occurrence、line span、缩进、键 token、分隔空白、值 token、行尾注释和换行符。
- 重复 `WeaponPreference`/重复键按 occurrence 单独寻址；只替换玩家明确选择的 value token。
- 未修改节点逐字节复用原始 slice，保持编码、注释、顺序、空白和尾部换行。
- 结构化解析失败时允许只读查看和导出诊断，不允许覆盖 VPK。

### 4.3 VPK 事务流水线

1. 确认 CS2 未运行，记录输入 VPK 的大小、mtime、SHA-256。
2. 在应用数据临时目录创建随机 workspace；禁止在游戏目录中间编辑。
3. 调用固定 sidecar 提取唯一 `db`。具体参数以纳入版本的 `--help` 为准；目标形态为 `vpkeditcli <input.vpk> --extract db --output <workspace>/db`。
4. 检查只产生一个普通文件 `db`，拒绝路径穿越、额外可执行文件、reparse point/symlink。
5. 读取 UTF-8、UTF-8 BOM 或可证明的原编码；保存时保持编码和换行。
6. 修改目标 token，校验区块/键值语法、已编辑数值、文件大小上限；异常不静默截断或归一化。
7. 先备份输入 VPK，再把原 VPK复制到 workspace candidate，使用 `--remove-file db` + `--add-file <edited-db> db` 输出新 VPK；不得先删除游戏目录中的原文件。
8. 对 candidate 运行 `--file-tree` 与 `--verify-checksums`。若格式没有 checksum，记录“无 checksum”，不能伪造通过。
9. 再次提取 candidate，确认只有 `db` 内容发生预期变化，其他 entry 路径/大小/SHA 与输入一致，DB SHA 等于编辑结果，candidate 可再次打开。
10. 保存到 profile 目录；激活时写同目录临时文件、flush/sync、原子 rename 到 `game/csgo/overrides/botprofile.vpk`，回读 SHA 后才更新 active 元数据。
11. 任一步失败都保留原 VPK；active 替换后回读失败必须从写前备份回滚并返回明确错误。

“删除修改前的 db”只发生在 candidate 内部，最终 VPK 必须恰好一个 `db` entry。

### 4.4 内置档案与升级

- 内置 Low/Medium/High 继续来自 `CS2BotImprover.zip`，只读；编辑任一内置难度都创建自定义派生档案。
- 自定义档案激活时复制到 active `overrides/botprofile.vpk`，Panel 记录 `activeProfileId` 和 `activeVpkSha256`；`disk_difficulty()` 扩展为识别自定义 SHA，不能只返回三档。
- 安装升级先调用 `capture_difficulty_workshop_state()`，校验并保留 profile 元数据、玩家 VPK 和 active 自定义选择；安装后恢复 custom active。当前 `capture_panel_preferences()` 只识别 active 等于三档的情况，必须同步修复。
- 卸载插件/切回 Online 不删除 app-local profiles。active 被移除后显示“自定义档案未激活”，不能误报 Low/Medium/High。
- 自定义档案不加入不可变插件 payload manifest；manifest 只校验内置资源。玩家 profile 用独立 schema/SHA 校验。
- 难度编辑绝不改 `gameinfo.gi`。BOT/Online 仍服从现有 `backup/WithBots` 与 `backup/Online` 契约。

## 5. Rust/API 实现清单

新增 `src-tauri/src/services/bot_difficulty.rs`、`models/bot_difficulty.rs` 和 `commands/bot_difficulty.rs`。复用现有 SHA、原子写入、CS2 进程检测和错误风格，但不要把 VPK 全部塞进 `panel.rs`。

建议命令：

```text
list_bot_profiles(root_path) -> BotProfileList
open_bot_profile(root_path, profile_id) -> BotProfileDocument
validate_bot_profile(document_id, text_or_edits) -> BotProfileValidation
create_bot_profile(root_path, name, base_profile_id) -> BotProfileDocument
save_bot_profile(document_id, edits) -> BotProfileSaveResult
activate_bot_profile(root_path, profile_id) -> PanelSnapshot
restore_bot_profile(root_path, profile_id) -> BotProfileSaveResult
export_bot_profile_db(document_id) -> ExportResult
```

返回模型包括：`id/name/source/baseDifficulty/active/dirty/dbSha256/vpkSha256/warnings/readOnly/validation/toolVersion`。所有 root 先走 `cs2::normalize_root()`；profile ID 是 UUID；拒绝 `..`、绝对路径和任意 sidecar 参数。

错误码至少包括：

```text
[BOT_WORKSHOP_CS2_RUNNING]
[BOT_WORKSHOP_PROFILE_NOT_FOUND]
[BOT_WORKSHOP_VPK_TOOL_MISSING]
[BOT_WORKSHOP_VPK_TOOL_HASH_INVALID]
[BOT_WORKSHOP_VPK_OPEN_FAILED]
[BOT_WORKSHOP_DB_ENTRY_MISSING]
[BOT_WORKSHOP_DB_AMBIGUOUS]
[BOT_WORKSHOP_DB_INVALID]
[BOT_WORKSHOP_VPK_REPACK_FAILED]
[BOT_WORKSHOP_VPK_ROUNDTRIP_MISMATCH]
[BOT_WORKSHOP_ACTIVE_ROLLBACK]
[BOT_WORKSHOP_BUILTIN_READONLY]
```

Sidecar 固定绝对路径，使用最小环境、隐藏窗口、30 秒超时、有限 stdout/stderr；不从 PATH 查找用户可替换的 CLI。纳入 `vpkeditcli.exe` 时同时保留 MIT LICENSE/CREDITS、版本和 SHA provenance，并在 `NOTICE.md` 说明。

## 6. 参数说明与编辑门禁

首版结构化字段限于：`Skill`、`Aggression`、`ReactionTime`、`AttackDelay`、`Teamwork`、四个 `AimFocus*`/`AimfocusInterval` 以及六个 `LookAngle*`。字段范围必须由当前上游 DB、注释与实机共同确定；没有范围证据就使用字符串 token 校验，不强制数值范围。

说明标注证据等级：

- `DB 注释`：直接来自目标 DB 注释。
- `上游文档`：来自固定 commit 的 README/源码。
- `待实机确认`：只写“影响 BOT 行为，具体效果需固定场景验证”。

未知 profile、Template、WeaponPreference 和重复键原样保留。保存前显示行号/区块/旧 token/新 token 差异；不上传用户 DB 全文或写入遥测。

## 7. 自动化测试

- `tests/bot-difficulty-workshop-contract.spec.ts`：入口、只影响 BOT 文案、ARIA、375/768/1440 布局契约。
- Rust 单测：profile ID 路径穿越、内置只读、CS2 运行中阻断、重复键选择、编码/换行保留、异常值拒绝、超时和事务回滚。
- `tests/vpk-roundtrip.spec.ts`：有 sidecar 时对 Low/Medium/High 和 active VPK 提取、无修改重包、二次提取；断言 DB 字节及非 DB 条目均一致。
- 自定义持久化：改一行、保存、重启读取同一 SHA；激活后 active SHA/Profile ID 一致；切回内置后自定义仍存在。
- 安装升级：`install_game_files_transactionally()` 前后 profile、metadata、备份和 active 自定义选择仍在；内置 manifest/marker/hash 仍通过 0.5.11 契约。
- fixture 使用临时目录；不得提交真实 Steam 路径、完整玩家 DB 或凭据。

建议命令：

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/bot-difficulty-workshop-contract.spec.ts tests/vpk-roundtrip.spec.ts tests/installer-contract.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml bot_difficulty panel cs2
```

全量 lint 若被既有生成物阻断，只记录基线阻断，不修改无关文件伪造通过。

## 8. 真实验收

证据目录：`E:\CS2AS05\artifacts\bot-difficulty-workshop-0.5.11-20260902\`。

1. 退出 CS2，打开内置 Medium，确认 DB 可读、档案只读、VPK/DB SHA 和工具版本有记录。
2. 创建“我的中等”副本，只改一个已证实字段；保存、关闭并重启助手，确认 diff、DB SHA、VPK SHA 持久化。
3. 激活自定义档案并启动 BOT；保存启动前 Panel snapshot、gameinfo SHA、active VPK SHA、启动参数与 Metamod/CounterStrikeSharp 日志。
4. 用固定地图、武器、距离、BOT 数量和站位对比内置 Medium/自定义，记录首次发现、开火延迟、瞄准稳定性等。聊天框文案只能作辅助证据。
5. 激活 Low/Medium/High/自定义，确认 gameinfo 不变、active SHA 正确，道具/NadeSystem 不回归。
6. 切回 Online，确认 active gameinfo 等于官方基线且不含 BOT SearchPath；app-local profiles 仍在。
7. 重启助手并执行一次候选包覆盖安装，确认 profile、元数据、备份与选择仍在；发现覆盖立即回滚并停止。

证据至少包含 `tool.json`（版本/SHA/许可证）、输入/输出/DB SHA、round-trip manifest、Panel snapshot、启动参数、日志、固定场景记录、Online 恢复摘要和测试输出。没有真实行为证据时只能写“内置编辑候选，待真实 BOT 验收”。

## 9. 停止条件

命中任一项立即停止：

- VPKEdit 无法稳定读写当前 VPK，或 round-trip 改变非 DB 条目；
- 找不到唯一 `db`、出现路径穿越/额外文件或目录树/checksum 回读不一致；
- 只能靠改名、全局替换、删除原 VPK或放宽校验让流程成功；
- 内置档案被覆盖、升级删除 profile、Online 残留 BOT SearchPath/参数；
- CS2 运行中仍能写 active VPK；
- 异常数值被截断、归一化或静默改写；
- 只有 UI/文件存在证据，没有 active SHA、round-trip 和真实 BOT 场景证据。

## 10. 执行交付物

1. 实现代码和聚焦测试，不改无关模块。
2. 新增 `docs/bot-difficulty-workshop-execution-report-0.5.11-20260902.md`，记录实际 sidecar commit/URL/SHA、CLI 命令、round-trip、数据目录、升级保留、测试和实机状态。
3. 脱敏证据放入上述 artifacts 目录；不得提交玩家完整 DB、凭据或密钥。
4. 若修改 `CS2BotImprover.zip`，运行 `scripts/generate-plugin-manifest.ps1`，同步 manifest、`CUSTOM_ZIP_SHA256`、fixture 和 ZIP 契约测试；不得修改 0.5.10 发布对象或历史 `.sig`。

最终状态只允许：

- `实现完成，待用户真实 BOT 验收`：文件链路和自动化通过，真实行为未完成；
- `候选已通过真实 BOT 验收`：固定场景、日志、SHA、Online 恢复和升级保留都有证据；
- `阻断`：命中停止条件，报告最早失败阶段与原因。
