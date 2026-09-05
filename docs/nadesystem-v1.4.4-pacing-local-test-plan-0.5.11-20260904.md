# 0.5.11 NadeSystem v1.4.4 节奏恢复与本机测试交接方案

日期：2026-09-04  
工作区：`E:\CS2AS05`  
交接对象：下一位实际执行 AI  
状态：方案制定完成；本文不等同于实现、打包或真实 CS2 验收

## 1. 任务目标与明确排除项

本轮交付一个用于本机真实测试的 0.5.11 安装程序，核心任务只有两项：

1. 在 `ed0ard/CS2-Bot-Improver v1.4.4` 的整包、源码、依赖和 API 之上恢复本项目的 NadeSystem 道具投掷节奏定制。
2. 极速构建 NSIS 安装器供本机测试。是否使用 Tauri updater 私钥、是否生成 `.sig`、是否 Authenticode 签名均不影响本轮目标；安装器不得上传或替换 0.5.10 正式发布资产。

**本轮删除 BotVision 合入任务。** 不下载、不升级、不替换、不重新审计 `XBribo/CS2-Bot-Vision`，不把 BotVision 版本改为 0.2.5，也不因本轮修改其 provenance、source hash、许可证或资源 fixture。当前已有 BotVision 资源必须在 v1.4.4 整包迁移中保持可用、不得被误删或被旧文件覆盖，但这只是回归约束，不是新版合入目标。

本轮不删除 VPKEdit 或自动换图默认状态代码；两项未完成功能继续保持生产入口隐藏。若当前工作树已有隐藏实现，执行 AI 只需回归确认，不要为本任务重新开放入口。

## 2. 已调查的上游基线与历史问题

### 2.1 v1.4.4 Windows 整包

2026-09-04 已核实官方 release：<https://github.com/ed0ard/CS2-Bot-Improver/releases/tag/v1.4.4>

- Windows 资产：`CS2BotImprover.zip`
- 大小：`68,536,430` bytes
- SHA-256：`CBA05FDC239BF0E3E95DFF7D8670863FE69DE9FB03ADE8512B6D4F79F88664DB`
- 下载地址：`https://github.com/ed0ard/CS2-Bot-Improver/releases/download/v1.4.4/CS2BotImprover.zip`

v1.4.4 已包含 Volumetric Smoke、fake-defuse、性能和队伍更新、BOT 投掷手雷时 radio/sound/chat、减少站桩、冻结时间购买限制、完整 profile、Panel、sig/offset 等变化。官方 NadeSystem 为 `ModuleVersion=1.2.1`，包含 `NadeSystemPlugin.Audio.cs` 相关逻辑。

### 2.2 必须修复的旧覆盖错误

此前安装包将旧定制源码编译的 `NadeSystem.dll`（`ModuleVersion=1.1.7`）直接覆盖到 v1.4.4 官方整包中，实际替代了官方 NadeSystem `1.2.1`，导致 `NadeSystemPlugin.Audio.cs` 的 BOT 投掷 radio、sound、chat 逻辑没有进入最终包。

本轮必须禁止该做法：不能把 `third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/bin/Release/.../NadeSystem.dll` 复制进 v1.4.4 包；不能以文件名相同为理由替换官方 DLL；不能通过删除 Audio partial 或绕过官方入口来“恢复节奏”。

### 2.3 当前项目文件与证据

执行前重读：

- `docs/upstream-v1.4.4-custom-package-file-audit-20260904.md`
- `docs/upstream-v1.4.4-migration-0.5.11-execution-report-20260904.md`
- `docs/upstream-v1.4.4-full-package-migration-plan-0.5.11-20260904.md`
- `src-tauri/resources/CS2BotImprover.zip`
- `src-tauri/src/services/cs2.rs`
- `scripts/generate-plugin-manifest.ps1`
- `third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/`

已有报告曾记录 v1.4.4 官方 `NadeSystem.dll` SHA-256 为 `9E4FC0CFD6B78D67C5EAEC86F76C76A7ACCC59820FCA3B02CBC517451D4074E4`；执行时必须从 v1.4.4 staging 重新计算，不能只引用旧报告。当前 ZIP 中的 BotVision 条目、版本和摘要保持原状，不得因为本轮而变更。

## 3. 执行顺序

### 阶段 A：冻结现场与取得唯一基线

1. 记录 `git status --short`、`package.json`/Cargo 版本、当前资源 ZIP SHA；保留所有用户脏改动，不执行 reset、clean、checkout 或删除历史证据。
2. 下载 v1.4.4 Windows 官方整包到 `workspace/vendor-cache/CS2-Bot-Improver-v1.4.4/`，保存 release API JSON、下载时间、大小、SHA 和 URL；SHA 必须等于上表值。
3. 解包到隔离 staging，拒绝绝对路径、`..`、重复路径、符号链接；生成完整条目清单。Linux 资产和 `CS2BotImprover_rules_unchanged.zip` 不得作为基线。
4. 复制当前 `src-tauri/resources/CS2BotImprover.zip` 为时间戳备份；所有候选 ZIP 通过临时文件和原子替换产生。

摘要、release tag、Windows 资产或解压内容不匹配时立即停止。

### 阶段 B：以 v1.4.4 整包为底恢复既有定制

1. 复制官方 v1.4.4 整包到临时工作 ZIP，默认保留全部 v1.4.4 文件；不要从旧 0.5.10 ZIP 选择性拼装上游文件。
2. 只恢复已存在且有证据的项目定制：MapRotation DLL/可变配置、`gameinfo.manifest.json`/BOT 变体、`CS2AS05.plugin.json` marker 及本项目必要安装门禁。已有 BotVision 文件保持 v1.4.4 基线中当前版本和内容，不做版本升级。
3. 对上游 v1.4.4 已存在的 NadeSystem 文件，先保存官方原始 `{path,size,sha256}`；后续只允许由阶段 C 的 v1.4.4 源码构建组替换，不得使用旧 1.1.7 二进制。
4. 生成 v1.4.4 与候选 ZIP 的全量 manifest，分类 `UNCHANGED/ADDED/CHANGED/REMOVED`；除明确的本项目新增或 NadeSystem 节奏重编译条目外，不得出现意外变化。BotVision 条目必须证明未被删除、降级或改写。

### 阶段 C：在 v1.4.4 源码上恢复节奏

1. 获取并固定 v1.4.4 对应源码和 NadeSystem 项目，确认官方 `ModuleVersion=1.2.1`、`NadeSystemPlugin.Audio.cs`、其余 partial 类、grenades 数据、csproj 依赖和 CounterStrikeSharp API 版本。
2. 保留官方 v1.4.4 `NadeSystem.cs`、`NadeSystemPlugin.Audio.cs`、其它 partial、`.deps.json`、PDB 和新 API 适配；不要修改或覆盖 `third_party/CS2-Bot-Improver-v1.4.3` 历史目录。
3. 从历史 1.1.7 源码/patch 中只提取“道具投掷节奏”行为差异，逐段移植到 v1.4.4 源码。补丁范围需明确记录：每 BOT 每回合上限、投掷间隔/调度、选择与计数、实体创建成功后的扣除/回滚、失败重试和配置常量。
4. v1.4.4 Audio partial 必须原样保留并可执行：BOT 投掷手雷时的 radio、sound、chat 不得被节奏补丁删除、绕过或重写。节奏代码应调用官方入口，不复制旧版 Audio 类。
5. 在独立 `third_party/CS2-Bot-Improver-v1.4.4/nades-pacing/` 保存上游源码、`UPSTREAM.md`、最小 patch/diff、构建产物摘要和测试；保留源码 hash 与最终程序集 hash。
6. 执行 `dotnet restore`、`dotnet build -c Release --nologo` 和节奏测试。使用程序集元数据/反射/IL 或等价证据确认输出来自 v1.4.4 API，语义为 `ModuleVersion=1.2.1`，而不是 1.1.7。
7. 将新编译 DLL 与匹配的 `.deps.json`、PDB 作为同一构建组写入临时 ZIP；保存官方 Audio 源码 hash、节奏 patch diff、DLL/deps/PDB hash。

若节奏 patch 无法在 v1.4.4 源码上干净移植，或必须覆盖/删除 Audio 逻辑才能编译，停止并报告冲突，不退回使用 1.1.7 DLL。

### 阶段 D：marker、payload 与安装门禁

1. 运行 `scripts/generate-plugin-manifest.ps1`，使 `version=0.5.11`、`generatedFrom=CS2-Bot-Improver-v1.4.4`；BotVision component/version/hash保持当前既有值，不改成新 release。MapRotation JSON 继续只列 `mutableConfigEntries`。
2. 更新 `CUSTOM_ZIP_SHA256`、Panel v1.4.4 常量、NadeSystem payload entries 和测试 fixture；不要改写或删除现有 BotVision provenance。
3. 校验 marker、payload、Rust 常量和实际 ZIP 三者一致；确认 NadeSystem DLL/deps/PDB 来自同一 v1.4.4 构建组，官方 Audio 相关文件仍在候选包中。
4. 安装事务继续校验 CS2 未运行、备份、临时文件、flush/sync、原子替换和安装后回读；Online 不得写入 BOT SearchPath。失败保留旧目录并回滚。

### 阶段 E：极速本机安装器

资源和代码门禁通过后立即构建：

```powershell
Set-Location E:\CS2AS05
npm run typecheck
npm run build:web
cargo check --manifest-path .\src-tauri\Cargo.toml
npm run bundle:desktop
```

以 `package.json` 当前 scripts 为准。构建前可不注入 Tauri 私钥；签名不是本机测试门槛。最终安装器放入：

`E:\CS2AS05\artifacts\nadesystem-v1.4.4-pacing-0.5.11-local-test-20260904\CS2人机增强助手_0.5.11_x64-setup.exe`

记录安装器大小、SHA-256、文件版本、生成时间、Authenticode 和 Tauri `.sig` 实际状态。只用于本机测试，不上传 R2/GitHub，不覆盖 0.5.10 产物。

## 4. 自动化测试门槛

至少执行并保存输出：

```powershell
npm test -- --run tests/gameinfo-upstream-contract.spec.ts tests/panel-data.spec.ts tests/map-rotation-contract.spec.ts tests/bot-difficulty-workshop-contract.spec.ts
npm run typecheck
npm run build:web
dotnet test .\third_party\CS2-Bot-Improver-v1.4.4\nades-pacing\tests\NadePacingPolicy.Tests.csproj -c Release --nologo
cargo check --manifest-path .\src-tauri\Cargo.toml
```

新增/更新契约必须证明：

- v1.4.4 上游文件无意外删除或变更；当前 BotVision 条目和 provenance 未被升级、降级或覆盖；
- NadeSystem DLL/deps/PDB 来自同一 v1.4.4 构建组，ModuleVersion 不是 1.1.7；
- `NadeSystemPlugin.Audio.cs` 或等价编译证据存在，radio/sound/chat 逻辑未被节奏 patch 删除；
- 节奏上限、间隔、实体失败回滚和计数测试通过；
- marker、payload、Rust `CUSTOM_ZIP_SHA256`、安装后回读一致；
- VPKEdit 和自动换图默认状态源码仍存在，生产入口隐藏且不调用 IPC；
- 安装升级/回滚、Online/BOT gameinfo 和 Panel 契约通过。

## 5. 本机真实测试交接

执行 AI 将安装器交给用户本机安装，用户至少记录：

1. 安装后 `NadeSystem.dll/.deps.json/.pdb`、Audio 相关文件、marker、payload 和 gameinfo SHA；同时确认当前 BotVision 文件未被改动。
2. 启动 BOT `-insecure`，确认 CounterStrikeSharp、NadeSystem 和现有 BotVision 均加载；让 BOT 在至少 3 个回合投掷手雷，观察 radio、sound、chat 以及节奏/间隔。
3. 切回 Online，确认官方 `gameinfo.gi` 字节恢复且不含 BOT SearchPath；再验证 Panel、Low/Medium/High。
4. VPKEdit、自动换图默认状态在生产界面不可达；其保留代码不影响启动。

真实游戏行为由用户记录前，报告只能写“本机安装器已生成，文件链路通过，待真实 BOT 行为验收”。

## 6. 立即停止条件

- 误用非 Windows v1.4.4 整包，或无法证明候选以 v1.4.4 为唯一新基线；
- NadeSystem 只能通过复制旧 1.1.7 DLL 实现节奏，或 v1.4.4 Audio 逻辑被删除、覆盖、绕过；
- 候选 ZIP 出现旧 Panel、旧 NadeSystem、旧 gameinfo，或现有 BotVision 被删除/降级/覆盖；
- marker、payload、Rust `CUSTOM_ZIP_SHA256`、安装后目录任一不一致；
- Online 污染、CS2 运行中写盘、安装失败无法回滚，或只生成裸 EXE 而无可安装 NSIS；
- 只有编译/ZIP 证据，没有安装后目录回读；未签名可以接受，但缺少路径、大小和 SHA 记录不可接受。

## 7. 执行 AI 输出

请新增实际日期执行报告，不覆盖本文，至少记录：v1.4.4 release/source commit、官方与候选 NadeSystem 文件摘要、节奏 patch diff、ModuleVersion/Audio 证据、全量 ZIP manifest、BotVision 未变更证明、测试结果、安装器路径/size/SHA/签名状态以及用户真实验收是否完成。最终结论只能使用：

- `v1.4.4 NadeSystem 节奏恢复已通过自动化，本机安装器已生成，BotVision 保持原状，待真实 BOT 验收`；
- `候选已通过真实 BOT 验收`；或
- `阻断：<最早阶段、错误、路径和证据>`。
