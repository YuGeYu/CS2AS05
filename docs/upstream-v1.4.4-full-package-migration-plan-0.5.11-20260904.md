# 0.5.11 上游 v1.4.4 整包迁移与定制恢复交接方案

日期：2026-09-04  
工作区：`E:\CS2AS05`  
交接对象：下一位实际执行 AI  
方案角色：方案制定 AI 调查结果；本文不代表代码、资源或真实 CS2 已完成

## 1. 目标与边界

0.5.11 必须以 `ed0ard/CS2-Bot-Improver v1.4.4` 的 **Windows 官方整包 `CS2BotImprover.zip`** 作为唯一新基线，然后把 0.5.10 相对于上游 v1.4.3 的全部下游定制按证据逐项恢复。禁止从 v1.4.3、0.5.10 ZIP 或“rules unchanged”包选择性拼接上游文件，也禁止只挑用户看得到的更新；v1.4.4 整包中的所有新 DLL、gamedata、Panel、配置、模型和运行时文件必须先进入候选，再按冲突/定制审计决定如何保留。

本轮同时要求：

1. 保留 VPKEdit、自动换图默认状态相关源码、IPC、Rust 服务和测试，不删除功能代码。
2. 在功能未完成前，从所有面向玩家的导航、按钮、卡片、设置入口和快捷路径隐藏这两项功能；隐藏后不得在后台调用对应 IPC。
3. 不修改 0.5.10 正式 R2 对象、线上发布记录、夸克链接、安装器或历史 `.sig`；不新开分支、不回退无关脏改动、不擅自发布。
4. Online 模式保持官方 `gameinfo.gi` 语义；BOT 变体仍只在退出 CS2 后通过现有事务安装，不能把 v1.4.4 迁移扩大为在线修改。

完成状态只能是“v1.4.4 整包候选已通过自动化、待真实 BOT 验收”或“阻断：最早失败阶段与证据”。

## 2. 已核实输入证据

### 2.1 v1.4.4 实时发布

来源：<https://github.com/ed0ard/CS2-Bot-Improver/releases/tag/v1.4.4>，GitHub API 在 2026-09-04 读取到：

| 项目 | 值 |
|---|---|
| tag | `v1.4.4` |
| 发布时间 | `2026-09-04T09:58:48Z` |
| Windows 资产 | `CS2BotImprover.zip` |
| 资产大小 | `68,536,430` bytes |
| 官方 digest | `sha256:cba05fdc239bf0e3e95dff7d8670863fe69de9fb03ade8512b6d4f79f88664db` |
| 下载地址 | `https://github.com/ed0ard/CS2-Bot-Improver/releases/download/v1.4.4/CS2BotImprover.zip` |

发布说明确认的整包变更包括 Volumetric Smoke 适配、全新 fake-defuse、性能提升、队伍名单更新、BOT 投掷道具时的 radio/sound/chat、开屏探员模型、减少站桩、仅冻结时间购买道具、每个 BOT 有有效 profile、Linux、Steam China、Panel 更新以及 sig/offset 修复。Linux 资产 `CS2BotImprover_for_Linux.zip` 和 `CS2BotImprover_rules_unchanged.zip` 均不是本轮 Windows 基线。

### 2.2 当前 0.5.10 定制基线

执行前重读当前文件和历史证据，不直接相信本文旧摘要：

- 资源：`src-tauri/resources/CS2BotImprover.zip`；当前报告曾记录最终摘要 `8C9D9315A79EDA0428598659DFBB1A8F02ACA7A5D937E2B545EE97241F69FA89`，实际执行前必须重新计算。
- 资源 marker：`addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json`，版本必须与当前 `package.json`（0.5.11 开发版本）重新生成，不能沿用旧版本文字。
- Rust 资源门禁：`src-tauri/src/services/cs2.rs` 的 `CUSTOM_ZIP_SHA256`、`PLUGIN_MARKER`、Panel 文件名和安装/回滚逻辑。
- 定制生成器：`scripts/generate-plugin-manifest.ps1`；它包含 NadeSystem、BotVision、MapRotation 固定 payload 和可变配置声明。
- 0.5.10 资源/功能报告：`docs/release-publish-0.5.10-final-20260831.md`、`docs/map-rotation-payload-digest-second-fix-and-ui-verified-execution-report-20260831.md`、`docs/gameinfo-bot-mode-medium-difficulty-execution-report-20260831.md`、`docs/CS2BotImprover-marker-diff-0.5.5.json` 及 `workspace/runtime/upstream-v1.4.3-0.5.4/`。

已知必须审计、不能遗漏的定制类别：BotVision 三项资源及 provenance；NadeSystem 1.1.7/道具投掷节奏定制；MapRotation DLL、外部默认配置及 `mutableConfigEntries` 语义；BOT/Online 两套 `gameinfo.gi` SearchPath 变体；`CS2AS05.plugin.json` marker、payload digest 和 0.5.10 版本门禁；Panel 命令/默认值/中文 UI 对齐；现有安装事务、备份、回读和 `-insecure` BOT 启动契约。实际白名单必须由逐文件比较生成，以上列表不是允许漏项的替代品。

## 3. 执行阶段

### 阶段 A：冻结现场和取得整包

1. 记录 `git status --short`、当前版本文件、资源 ZIP SHA、所有未跟踪/已修改路径；不得清理或覆盖既有改动。
2. 下载 v1.4.4 到 `workspace/vendor-cache/CS2-Bot-Improver-v1.4.4/`，保存 API JSON、URL、时间、大小、digest；对下载文件重新计算 SHA，必须等于 `CBA05FDC239BF0E3E95DFF7D8670863FE69DE9FB03ADE8512B6D4F79F88664DB`。
3. 复制当前 0.5.10 ZIP 为带时间戳的只读证据备份；官方压缩包和解压目录均置于隔离 staging。解压拒绝绝对路径、`..`、重复大小写路径、符号链接和异常条目。
4. 获取并固定 v1.4.4 源码 tag/commit，记录 commit、许可证、release JSON 和所有输入 SHA；Windows 整包内容优先于源码注释。

任一摘要不匹配、下载内容不是 Windows 整包、解压失败或工作树现场无法保留，立即停止。

### 阶段 B：整包逐文件审计

为官方 v1.4.4、当前 0.5.10 ZIP、已知 v1.4.3 官方快照生成规范化清单 `{path,size,sha256,isDirectory}`，路径按 ordinal 排序。输出：

`artifacts/upstream-v1.4.4-migration-0.5.11-20260904/input-manifests/{v1.4.4,custom-0.5.10,upstream-v1.4.3}.json`

将差异分成 `ADDED/REMOVED/CHANGED/UNCHANGED`，再为每个变化标注 `upstream-required`、`downstream-custom`、`conflict` 或 `unknown`。Panel、所有 DLL、gamedata、cfg、VPK、模型、资源和许可证都必须出现在报告中；不能用“主要文件”摘要代替全量清单。

### 阶段 C：在 v1.4.4 整包上恢复 0.5.10 定制

1. 复制官方 v1.4.4 整包到临时工作 ZIP，默认保留全部 v1.4.4 文件。
2. 对每个 0.5.10 定制文件从源码/脚本/历史报告重建，而不是把旧二进制盲目覆盖到新包：NadeSystem 必须以 v1.4.4 API/依赖重新编译；MapRotation 必须重新编译并核对 API；BotVision 要确认 v1.4.4 MetaMod/CS2 兼容性；gameinfo 变体从当前 Steam 官方 bytes 重新生成；marker 由脚本生成。
3. 对 Panel、CounterStrikeSharp API、gamedata、sig/offset 等 v1.4.4 已更新文件，先做兼容审计。只有在证明 0.5.10 定制仍需改变且不会回退 v1.4.4 修复时才重放定制；不得以旧 v1.4.3 DLL 覆盖新上游 DLL。
4. 每个定制重放保存 patch 输入、目标路径、before/after SHA、构建命令、依赖版本和理由。未知冲突不猜测，标为阻断。
5. 用临时 ZIP 原子替换 `src-tauri/resources/CS2BotImprover.zip`；失败保留原 ZIP 和备份，不直接破坏工作树。
6. 运行 `scripts/generate-plugin-manifest.ps1`，同步 marker、payload entries、`mutableConfigEntries`、`CUSTOM_ZIP_SHA256`、fixture 和 ZIP 契约测试。`generatedFrom` 改为 `CS2-Bot-Improver-v1.4.4`，不可保留 v1.4.3。

### 阶段 D：隐藏未完成入口

**VPKEdit：**保留 `src-tauri/src/services/bot_difficulty.rs`、命令、模型、`src/services/tauri/bot-difficulty.ts`、`BotDifficultyWorkbench.vue`、sidecar、测试和文档；从 `src/views/OverviewView.vue` 移除强度工坊按钮/入口及任何快捷入口。组件可以继续被内部测试挂载，但生产导航不得可达，不得在概览加载时调用 VPKEdit IPC。不要删除 `src-tauri/binaries/` 或 `resources/vpkedit/`。

**自动换图默认状态：**保留 `MapRotationDefaultControl.vue`、`MapRotationDefaultNotice.vue`、`src/services/tauri/mapRotation.ts`、Rust 命令和插件代码；从概览和“安装与诊断 → 主题设置”移除入口、说明模态和所有可点击控件。不得只改成 disabled 后继续展示，也不得后台读取 `get/set/reset_map_rotation_default`。保留内部测试/源码契约，生产 UI 隐藏可由明确常量控制但默认必须为 false。

隐藏测试必须证明：玩家导航、概览、设置抽屉和键盘/快捷路径均找不到入口；挂载主界面不会触发相关 IPC；源码/服务仍存在，未来恢复无需重写。不要把隐藏误写成删除或功能修复。

### 阶段 E：版本、安装和回归

更新 `src-tauri/src/services/cs2.rs`、Panel/资源版本显示、README/NOTICE、上游来源文档和测试 fixture；历史 0.5.10 文档保持原样，不做全局替换。安装、升级、卸载、Online/BOT 切换继续使用现有备份、原子写、回读和 CS2 运行锁。自定义 VPK profile（若工坊代码已产生）必须保留，不能因替换整包删除 app-local 数据。

## 4. 验证和证据门槛

自动化至少执行并保存 stdout/stderr：

```powershell
Set-Location E:\CS2AS05
npm test -- --run tests/panel-data.spec.ts tests/gameinfo-upstream-contract.spec.ts tests/map-rotation-contract.spec.ts tests/map-rotation-card-visibility.spec.ts tests/bot-difficulty-workshop-contract.spec.ts
npm run typecheck
npm run build:web
cargo fmt -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 panel bot_difficulty map_rotation
cargo check --manifest-path .\src-tauri\Cargo.toml
```

新增/更新测试必须覆盖：全量 ZIP 白名单和每个变化分类；v1.4.4 marker/source/payload；BotVision/NadeSystem/MapRotation/gameinfo 条目及摘要；Panel 新版本资源与命令契约；Online 不含 BOT SearchPath；隐藏入口无 IPC；安装升级回滚和 app-local profile 保留。构建安装器前先通过资源和代码门禁；不要复用 0.5.10 `.sig`。

证据根目录：`E:\CS2AS05\artifacts\upstream-v1.4.4-migration-0.5.11-20260904\`，至少包含 `release-api.json`、输入/候选 ZIP SHA、逐文件 manifest、定制恢复报告、marker/payload 报告、测试输出、隐藏入口截图或 DOM 证据、安装器摘要。不得写入完整玩家 DB、密钥、token 或私钥。

真实 CS2 由用户执行：退出 CS2 安装候选，分别验证 Online、BOT、Low/Medium/High、NadeSystem、BotVision、MapRotation、Panel、切回 Online 和升级后自定义 profile；记录启动参数、插件日志、gameinfo/VPK/marker SHA 和异常退出码。自动化通过不等于真实 BOT 通过。

## 5. 立即停止条件

- 没有拿到或校验通过 v1.4.4 官方 Windows 整包；误用 Linux 或 rules-unchanged 包。
- 任一 v1.4.4 文件被旧 v1.4.3/0.5.10 二进制覆盖，或无法证明定制重放不会丢失上游新功能。
- 全量 manifest 缺失、marker/payload/CUSTOM_ZIP_SHA256 不一致、非预期条目被删除。
- v1.4.4 API/gamedata/offset 与定制插件不兼容，或只能通过放宽版本门禁/跳过 hash 才能启动。
- 隐藏后仍有玩家可达入口、后台 IPC、可操作 Toggle，或删除了要求保留的代码。
- Online 写入 BOT SearchPath、CS2 运行中写盘、安装失败无法回滚、升级删除自定义 profile。
- 只有 ZIP/编译/UI 证据，没有安装后实际目录回读；真实 BOT 未由用户执行时不得宣称行为完成。

## 6. 执行 AI 交付报告

实际执行 AI 应新建或更新带实际日期的执行报告，逐阶段记录输入、命令、SHA、文件变化、通过/阻断/未执行和最早失败点；不得覆盖本方案。报告结论必须使用以下之一：

1. `v1.4.4 整包迁移和 0.5.10 定制恢复已通过自动化，待真实 BOT 验收`；
2. `候选已通过真实 BOT 验收`；
3. `阻断：<阶段、路径、错误码和证据文件>`。

禁止把“下载成功”“Panel 更新”“VPKEdit sidecar 存在”或“入口隐藏”单独描述为整包迁移、VPK 编辑、自动换图或真实游戏功能完成。
