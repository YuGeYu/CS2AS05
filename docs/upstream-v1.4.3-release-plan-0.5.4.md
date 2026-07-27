# 0.5.4 上游 v1.4.3 整包迁移、Panel 对齐与道具节奏实施方案

## 1. 交接角色与任务边界

本文交给下一位【实际执行 AI】。两位 AI 不共享聊天上下文，执行 AI 必须把本文当作完整任务说明，先复核时间敏感状态，再直接实施、验证和构建候选安装程序，不要再次只输出方案。

本轮【方案制定 AI】只完成调查、静态提取和方案文档：没有修改业务代码，没有替换 `src-tauri/resources/CS2BotImprover.zip`，没有升版本，没有编译 DLL/安装程序，没有写真实 CS2 目录，没有部署官网或发布 GitHub Release。

仓库：`E:\CS2AS05`

上游项目：`https://github.com/ed0ard/CS2-Bot-Improver`

固定上游版本：`v1.4.3` / commit `d1d83982db88fbdb686b2bf13aa8c6f9d65a4604`

用户提供的 Windows 整包：`E:\dow\CS2BotImprover (5).zip`

目标应用版本：`0.5.4`

不要新建分支。不要 reset、checkout、restore 或清理当前工作区。当前已有 UI 动效和 Tauri 联网自更新改动通常允许随 `0.5.4` 一起发布，必须增量开发并完整回归。

## 2. 完成定义

只有下列条件全部成立，才能称为 `0.5.4` 候选版完成：

1. 内置包以用户给定的官方 Windows `v1.4.3` 整包为新基线，不从旧 `v1.4.2` ZIP 逐文件拼装。
2. 新包中除明确的下游定制项外，其余文件与官方 `v1.4.3` 保持逐项一致；官方 `Panel v1.4.3.exe` 大小和 SHA256 不变。
3. 原生 Bot Items 页面和后端与发布包内实际 `Panel v1.4.3.exe` 的八项模型、顺序、联动及最终磁盘效果一致。
4. 不做旧 `skins` 偏好迁移。旧用户覆盖升级后允许回到上游新模型默认值，但不得崩溃、卡死、误删用户文件或阻止启动。
5. `Nades` 增加 `less`，前端、TypeScript、Tauri command、Rust 白名单、首次初始化、回读、命令库和测试形成完整闭环。
6. NadeSystem 以官方 `v1.4.3` / module `1.1.7` 源码为基线，手工重放当前项目最近的“道具投掷节奏”定制；不得把旧 `1.1.6` DLL 直接塞进新包。
7. `less` 保留上游“每 Bot 每回合总数最多 4”的语义，同时所有实际创建路径继续遵守本项目的硬上限、节奏、事务提交和失败回滚约束。
8. 版本、来源、资源哈希、marker、fixture、README/NOTICE、安装页和测试契约全部更新到 `0.5.4` / `v1.4.3`，但历史 `0.5.3` 方案文档不做全局替换。
9. 当前联网自更新、退出确认、NSIS hooks 和 UI 动效改动不回退；签名 NSIS、`.sig`、release manifest、全套测试和隔离升级验证通过。
10. 真实 CS2 游戏内效果由用户执行本文给出的最短验收。用户确认游戏内结果前，只交付候选安装包，不切生产 D1/R2 更新元数据，不宣布正式发布。

## 3. 2026-07-26 调查基线

### 3.1 当前仓库与脏工作区

调查时：

- 分支：`main`
- HEAD：`ba51d0b`（`Release 0.5.3 with integrated Panel and overview enhancements`）
- `main` 与 `origin/main` 指向同一提交。
- 项目版本仍为 `0.5.3`。
- 工作区不干净，既有已暂存、未暂存和未跟踪内容。

主要既有改动包括：

- UI 动效：`AppShell.vue`、`SegmentedControl.vue`、`ToggleSwitch.vue`、多个 view、`main.css`、截图和 `tests/technology-motion.spec.ts`。
- 联网自更新：updater/process 依赖、Tauri capability/config、`SoftwareUpdateModal.vue`、`SupportActions.vue`、`PendingUpdateExitModal.vue`、updater state/service 和测试。
- `README.md`、`docs/technology-motion-ui-plan-0.5.3.md`、`docs/network-self-update-r2-plan-0.5.3.md` 等文档。

执行 AI 第一步必须重新运行：

```powershell
Set-Location E:\CS2AS05
git status --short --branch
git diff --stat
git diff --cached --stat
git log -5 --oneline --decorate
```

这些改动不得回退；后续版本升级和测试必须覆盖它们。

### 3.2 官方 v1.4.3 发布资产

GitHub Release API 与本地文件一致：

| 项目 | 值 |
| --- | --- |
| Release | `v1.4.3` |
| 发布时间 | `2026-07-26T08:14:39Z` |
| 本地文件 | `E:\dow\CS2BotImprover (5).zip` |
| 大小 | `67,801,513` bytes |
| SHA256 | `F4330B13F6154A36EEB2820FB79AF015014397C7E06C7791AE0AF51F762A3652` |
| ZIP entries | `718`（含目录） |
| 非目录文件 | `650` |

GitHub API 对 `CS2BotImprover.zip` 返回的官方 digest 也是：

```text
sha256:f4330b13f6154a36eeb2820fb79af015014397c7e06c7791ae0af51f762a3652
```

不要误用另一个 `CS2BotImprover_rules_unchanged.zip`。后者大小 `67,801,219`，SHA256 `0F105F27128EBB9226239C6CA9E52CDBAC83407A463D8BE282B55AD2705F6DA3`，不是用户指定的整包。

官方 Panel：

| 项目 | 值 |
| --- | --- |
| ZIP entry | `Panel v1.4.3.exe` |
| 大小 | `5,844,480` bytes |
| SHA256 | `3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD` |

### 3.3 当前 0.5.3 内置包

当前 `src-tauri/resources/CS2BotImprover.zip`：

| 项目 | 值 |
| --- | --- |
| 大小 | `67,680,423` bytes |
| SHA256 | `55DC504BFF8340ABE6AB317661518B512DB6FFA463F10EE3AF84149B1795C3E9` |
| 非目录文件 | `651` |
| Panel | `Panel v1.4.2.exe` |

逐文件哈希比较当前定制包与官方 `v1.4.3` 共发现 `65` 个路径变化。关键差异包括：

- `BotRandomizer.dll` 从 `45,568` 增至 `98,816` bytes。
- 删除 `BotRandomizer/bot_randomizer_options.json` 和 `skins_en.json`。
- 新增 `charm_placements.json` 和约 1.19 MB 的 `cosmetic_catalog.json`。
- `NadeSystem.dll` 官方新版本为 `54,784` bytes、SHA256 `A4ADBF6BECC70736F6DE67DFBECB2BC7354E5BB262DCF05F7EEC7FC98CA96E7C`。
- `overrides/botprofile.vpk`、BotAI、BotController、BotHider、BotState 和 Metamod 文件有更新。
- 官方新包的两个 `my_bot_*.cfg` 不再包含根级 `bot_aim` / `bot_nades` 默认行，只保留 alias；下游首次默认值脚本必须在新基线上重新处理。
- 当前下游 marker `CS2AS05.plugin.json` 不在官方包内，必须由 `0.5.4` 生成脚本重新创建。

### 3.4 公开源码与实际 Panel 的冲突

行为证据优先级必须是：实际发布 EXE/落盘差分 > 发布 ZIP > tag 源码。

`v1.4.3` tag 的公开 `Panel/src` 只把版本文字改为 `1.4.3`，仍错误地声明：

- Bot Items 只有 `skins / profiles / agents / music` 四项。
- Nades 只有 `max / more / normal / off`。

但发布包内实际 `Panel v1.4.3.exe` 的 Tauri Brotli 资源已静态提取，内嵌 JS 为：

```text
workspace/runtime/upstream-v1.4.3-analysis/index-Bzt5rBFp.js
SHA256 FE16A2E1D50B99C928D063BF959A05E78C4124FA01E1695E98774D4356BF9F4E
```

实际前端契约是：

```text
BotItemKey:
profiles, agents, music, weapons, knives, gloves, stickers, charms

中文标签与顺序：
档案、探员、音乐盒、武器皮肤、刀、手套、印花、挂件

Nades:
max, more, normal, less, off
```

实际 EXE 的 Rust DTO 字符串同样包含上述八个布尔字段和 `src\commands\bot_items.rs`。因此实现时禁止继续以公开 `Panel/src/lib/api.ts` 的四项类型为准。

## 4. 已批准范围与需确认范围

### 4.1 已批准并直接实施

1. 使用用户提供的官方 `v1.4.3` Windows 整包。
2. Bot Items 直接跟随实际 Panel 的八项模型，不兼容旧 `skins` 用户偏好。
3. Nades 新增 `less`，并把 `bot_nades less` 加入本项目命令数据。
4. 把上游新命令 `br_reroll` 加入本项目“命令”UI，支持解析、搜索和复制。
5. 在官方 `v1.4.3` NadeSystem 上恢复本项目最近的“道具投掷节奏”定制。
6. 版本升为 `0.5.4` 并构建新的安装程序。
7. 保留当前安装器、联网更新、签名和关闭逻辑的大部分实现，只做版本/资源必要调整。
8. 进行任何逆向破解，获取或间接获取源码。

### 4.2 使用整包后必然一并带入的上游运行时变化

下列内容不是本项目新增 UI，但由于用户明确要求采用新整包，会随官方二进制自动进入 `0.5.4`：

- 印花和挂件、扩展皮肤库及稳定性改进。
- Bot 看到敌人时中断换弹并切回武器。
- 合适时机检视武器。
- Bot POV 清理。
- Deathmatch/Arms Race 错误切刀修复。
- 职业队伍添加修复。
- BotController、BotHider、BotAI/签名/offset 等整包依赖更新。

不要为这些运行时变化擅自增加独立设置页、开关、宣传文案或下游逻辑。若用户不同意其中任何一项，就无法同时满足“采用官方新整包”，应停止并说明冲突。

### 4.3 其他未批准项

`br_reroll` 命令 UI 已由用户明确批准，不再是实施阻塞项。仍未经确认不得：替换定制 PDB、增加新设置页、改变默认难度/模式、重做安装器视觉、删除旧 app-data 工具目录或公开发布元数据。

## 5. 实施顺序

严格按本节顺序执行。每阶段通过后再进入下一阶段；不要一上来覆盖资源 ZIP 或构建 NSIS。

### 阶段 A：冻结输入与备份

1. 记录当前 Git 状态、所有版本字段和资源摘要。
2. 在 `workspace/runtime/upstream-v1.4.3-0.5.4/` 建立本次专用工作目录。
3. 把 `E:\dow\CS2BotImprover (5).zip` 复制为只读基线副本，不修改用户下载的原文件。
4. 对当前 `src-tauri/resources/CS2BotImprover.zip` 做时间戳备份，备份放入 `workspace/runtime/`，不要提交大体积备份。
5. 获取并固定 tag `v1.4.3` 源码；记录 commit 和所有输入 SHA256。
6. 在任何真实 CS2 文件操作前让用户备份实际 `game/csgo`，候选开发阶段优先使用复制夹具。

输入摘要不匹配时立即停止，不得“按文件名看起来一样”继续。

### 阶段 B：先证明官方 Panel 的 Bot Items 后端契约

新官方包删除了旧 `bot_randomizer_options.json`，当前 Rust 后端却仍把该文件列为必需条目并直接读写四个字段。不能通过扩展旧 JSON 猜测新行为。

实施编码前先做一次官方 Panel 黑盒差分：

1. 从固定 ZIP 提取经过哈希验证的 `Panel v1.4.3.exe` 到隔离工具目录。
2. 准备一份可丢弃的 `game/csgo` 夹具，包含完整新包，不指向用户真实游戏目录。
3. 快照所有文件的相对路径、大小、SHA256 和关键 JSON 结构。
4. 用官方 Panel 逐次只切换一个项目，每次从同一干净夹具开始：
   - `profiles`
   - `agents`
   - `music`
   - `weapons`
   - `knives`
   - `gloves`
   - `stickers`
   - `charms`
5. 每次切换后立即重新快照，记录新增、删除、移动、内容变化和八项回读状态。
6. 特别确认多个选项是否共享同一个 BotRandomizer 插件开关。不得因为 UI 有八个 toggle 就假设八项一定独立落盘。
7. 再验证 CS2 运行中切换时的 pending/restart 状态；不得在真实游戏运行时改文件。
8. 保存报告为 `docs/panel-v1.4.3-bot-items-contract.json`，至少包含 EXE/ZIP 摘要、每个输入、变化路径、before/after SHA256、联动项和默认状态。

如果无法获得这份落盘差分，停止 Bot Items 后端实现并向用户说明；禁止退回旧 `bot_randomizer_options.json` 方案。

### 阶段 C：实现八项 Bot Items

以阶段 B 的实际差分为唯一后端真值，修改：

- `src/features/panel/types.ts`
- `src-tauri/src/models/panel.rs`
- `src-tauri/src/services/panel.rs`
- 必要时 `src-tauri/src/commands/panel.rs`
- `src/services/tauri/panel.ts`
- `src/stores/panel.ts`
- `src/views/BotItemsView.vue`
- 对应 Vitest 和 Rust tests

前端类型必须精确为：

```ts
export type BotItem =
  | 'profiles'
  | 'agents'
  | 'music'
  | 'weapons'
  | 'knives'
  | 'gloves'
  | 'stickers'
  | 'charms'
```

删除活动代码中的 `skins` 字段，不做迁移层。旧配置处理规则：

- 旧 `bot_randomizer_options.json` 存在时不解析、不迁移，也不主动删除；它在新插件中是无效遗留文件，保留比广泛删除更安全。
- 新配置缺失时按官方 Panel `v1.4.3` 的默认状态初始化。
- 旧 `cfg/cs2as05-panel-state.json` 中四项偏好不能让启动失败；忽略不认识的旧 Bot Items 部分，再按官方新状态回读。
- 其他模式、难度、Aim、Nades 和刀具偏好仍按现有逻辑保留，不因“不兼容 skins”而整体清空。

UI 与实际 Panel 对齐：

| key | 中文标签 | 建议说明 |
| --- | --- | --- |
| `profiles` | 选手档案 | 控制 Bot 档案相关效果。 |
| `agents` | 探员模型 | 控制 Bot 探员模型。 |
| `music` | 音乐盒 | 控制 Bot 音乐盒。 |
| `weapons` | 武器皮肤 | 控制武器皮肤。 |
| `knives` | 刀具外观 | 控制 Bot 刀具外观。 |
| `gloves` | 手套外观 | 控制 Bot 手套外观。 |
| `stickers` | 武器印花 | 控制武器印花。 |
| `charms` | 武器挂件 | 控制武器挂件。 |

继续使用现有 `ToggleSwitch.vue`，不要用浏览器原生弹窗或 prompt/confirm。成功动效只能由 Tauri 后端成功返回的新 snapshot 触发，不能点击即乐观成功。

八行设置允许页面纵向滚动。不要为了塞进 `720x620` 缩小字体或压缩触控区域；保持键盘顺序、可见 focus、错误 `role="alert"`、双主题和 `prefers-reduced-motion`。不得引入 3D、渐变装饰或新色系，继续沿用当前紧凑桌面工具样式。

### 阶段 D：Nades 增加 less，命令 UI 增加 br_reroll

修改所有枚举/白名单：

- `src/features/panel/types.ts`：`NadesValue` 加 `'less'`。
- `src/views/PresetsView.vue`：在 `normal` 与 `off` 之间增加 `{ value: 'less', label: '较少' }`。
- `src-tauri/src/services/panel.rs`：所有 `managed_value`、`set_preset` 和校验数组加 `less`。
- `src/data/panel/commands.txt`：在 `BOT NADE THROWING` 下增加 `bot_nades less`。
- fixtures、panel data tests、API tests、technology motion fixtures 全部补齐。

在 `src/data/panel/commands.txt` 的 `BOT MANAGEMENT` 分类中加入精确命令 `br_reroll`。复用现有 `parseCommands()` 和 `CommandsView.vue` 通用列表，不新增专用设置页、专用后端 Tauri command 或浏览器原生剪贴板调用。至少增加以下测试：

- `tests/panel-data.spec.ts` 更新命令文件 SHA256、解析总数和 fixture manifest，并断言 `br_reroll` 仅解析为一条可复制命令。
- 搜索 `br_reroll` 时只定位到该命令，上/下匹配导航仍限于命令列表滚动容器。
- 点击该行后通过 `@tauri-apps/plugin-clipboard-manager` 写入精确字符串 `br_reroll`，成功/失败反馈与现有命令一致。
- `tests/technology-motion.spec.ts` 或独立 CommandsView 测试覆盖搜索、复制成功和剪贴板失败；不只依赖文件包含断言。

不要只改 UI。验收必须证明：

1. 点击 `less` 后两个 cfg 都只有一行 `bot_nades less`。
2. 重新启动应用后 snapshot 仍为 `less`。
3. 覆盖安装后 `less` 仍能回读。
4. 非法值被 Rust 拒绝，不能写任意 console command。
5. 五段控件在 `720x620`、`960x700`、`1280x800` 下不截字、不重叠。

### 阶段 E：在官方 1.1.7 上重放道具投掷节奏

当前定制源码位于：

```text
third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/
```

旧 patch 直接应用到 `v1.4.3` 已确认失败：

```text
patch failed: .../NadeSystem.cs:101
patch does not apply
```

因此必须新建：

```text
third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/
```

从官方 tag 复制 `NadeSystem.cs` 和 `NadeSystem.csproj`，再手工移植，不要以旧定制文件覆盖新文件。更新 `UPSTREAM.txt`，记录：

- tag/commit
- 官方 ZIP/源码 SHA256
- 官方 NadeSystem module version `1.1.7`
- RayTraceApi 依赖来源与 SHA256
- 构建命令
- 下游策略说明

必须保留的上游 `less` 行为：

- 命令接受 `off|less|normal|more|max`。
- `less` 使用 normal 的方向/信息/早期烟雾判断。
- 每 Bot 每回合 smoke/HE/molotov 各最多 1。
- flash 上限取 `ammo_grenade_limit_flashbang`，默认 2。
- 每 Bot 每回合所有投掷合计最多 4。
- retaliation 同样计入 `less` 上限。
- round start 清空 `less` 计数。

必须恢复的下游节奏：

- 每 Bot 每回合 flash 最多 2，smoke/HE/molotov/incgrenade 各最多 1。
- 开局 15 秒每 Bot 最多一次非紧急投掷。
- 开局队伍总预算为 `min(aliveBots, 5)`；最多 1 烟、3 个非烟。
- 同一 Bot 两次非紧急投掷至少间隔 5 秒；同队至少间隔 0.5 秒。
- defuse cover 和 extinguish fire 绕过节奏但不绕过硬上限；plant cover 只绕过开局队伍预算。
- 候选按 molotov > HE > flash > smoke 和稳定 ID tie-break 选择。
- HE/molotov 有效敌人半径 450；team-tagged scheduled flash fallback 为 1600。
- normal planned smoke 每队每回合成功创建最多 1。
- 不生成 `[NadeAudit]` 或 `[NadeLimit]` 新日志；保留上游原有诊断。

事务要求：官方 `1.1.7` 仍有部分路径先扣钱/计数再创建实体。下游必须继续做到只有实体创建成功后才提交：

- 硬上限计数
- pacing reservation
- 金钱/round spend
- team/less 计数
- cooldown

创建失败、下一帧 pawn 失效或异常时必须 Cancel，不能消耗额度或金钱。

推荐把 `less` 的总数 4 约束合并进可测试策略，或保留上游 `_roundCountByBot` 并明确证明它与 `NadePacingPolicy` 同步。不得出现两个计数器一个提交、另一个失败的分裂状态。

策略覆盖至少三类入口：

1. planned replay (`TryReplay`)
2. instant special（defuse/plant/extinguish）
3. retaliation HE/molotov

测试至少增加：

- `less` 第 4 次允许、第 5 次拒绝。
- `less` 各类型上限和 `incgrenade`/`molotov` 共桶。
- `less` round reset。
- `less` 在 replay、instant、retaliation 都不会绕过硬上限。
- entity creation 失败释放 reservation 且不扣钱/不写 cooldown。
- 原有 49 项或当前实际数量的 pacing assertions 全部保留并增加，而不是删测试换绿灯。
- 源码与最终 DLL `ModuleVersion`/输入基线可追溯。

构建：

```powershell
dotnet restore .\third_party\CS2-Bot-Improver-v1.4.3\nades-pacing\NadeSystem.csproj
dotnet build .\third_party\CS2-Bot-Improver-v1.4.3\nades-pacing\NadeSystem.csproj -c Release
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.3\nades-pacing\tests\NadePacingPolicy.Tests.csproj -c Release
```

### 阶段 F：从官方新整包生成 0.5.4 定制 ZIP

更新并复用现有结构化脚本：

- `scripts/update-panel-defaults.ps1`
- `scripts/replace-nadesystem.ps1`
- `scripts/generate-plugin-manifest.ps1`

需要调整：

- 输入基线改为固定官方 `v1.4.3` ZIP。
- Panel 名称/大小/SHA256 改为本方案第 3.2 节。
- Nade DLL 路径改到 `third_party/...v1.4.3/nades-pacing/...`。
- marker `version` 为 `0.5.4`，`generatedFrom` 为 `CS2-Bot-Improver-v1.4.3`。
- `update-panel-defaults.ps1` 的 Nades 正则接受 `less`。
- 报告改为 `docs/CS2BotImprover-defaults-diff-0.5.4.json`。
- 所有中间操作在临时副本完成，验证后原子替换目标 ZIP。

官方新包没有 `bot_randomizer_options.json`，因此必须从以下位置删除旧的必需条目/假设：

- `src-tauri/src/services/cs2.rs::REQUIRED_ZIP_ENTRIES`
- Panel 初始化/安装测试
- fixture 和脚本校验
- README/NOTICE 的旧四项配置描述

相对官方 `v1.4.3`，预期下游 ZIP 白名单差异为：

```text
CHANGED cfg/my_bot_normal_config.cfg
CHANGED cfg/my_bot_ffa_config.cfg
CHANGED addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll
ADDED   addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json
```

若实际策略决定不再固化两个 cfg 默认行，必须先说明并让用户确认；否则按当前 `0.5.3` 默认行为继续保留。

不要替换 `NadeSystem.pdb`，除非先向用户确认这项额外改动。若保留官方 PDB，应在证据中明确它不是定制 DLL 的匹配符号文件，运行时不依赖它。

生成后做两次独立重放，要求最终 ZIP SHA256、大小、entry 数和 marker payload 完全一致。Panel 必须仍是官方摘要。

### 阶段 G：更新资源验证、版本和来源

修改：

- `src-tauri/src/services/cs2.rs`
  - `CUSTOM_ZIP_SHA256`
  - `PANEL_FILE_NAME/PANEL_SHA256/PANEL_SIZE`
  - `official-panel-v1.4.3` app-data 目录
  - 日志/成功文案中的上游版本
  - `REQUIRED_ZIP_ENTRIES`
  - 相关 Rust tests 和临时夹具名
- `tests/fixtures/panel-v1.4.2/manifest.json`：新建/改名为 `panel-v1.4.3/manifest.json`，更新所有摘要与命令计数。
- `tests/installer-contract.spec.ts` 和其他硬编码版本测试。
- `README.md`、`NOTICE.md`、`docs/panel-v1.4.2-contract.md`：新增 `v1.4.3` 契约文档，历史文档可保留，不做伪造式全局替换。
- `src/components/AboutSourcesModal.vue` 的来源说明。

版本字段只改项目自己的版本：

- `package.json`
- `package-lock.json` 根 package 两处
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock` 中本项目 package
- `src-tauri/tauri.conf.json`
- `src/components/AppShell.vue`
- `src/views/InstallView.vue`
- 版本相关测试/marker/release manifest

禁止全局替换 `0.5.3`，历史文档、测试用旧版本样本和第三方依赖版本可能必须保留。

旧用户覆盖升级规则：

- 旧 marker `0.5.3` 会触发安装 `0.5.4`。
- 旧 `official-panel-v1.4.2` app-data 目录不删除；新版本使用独立 `official-panel-v1.4.3`。
- 旧 `bot_randomizer_options.json` 可残留但不再参与状态。
- 不得因为新包删除某文件就对整个插件目录做未知文件清理。
- 其他合法 Panel 偏好继续按当前捕获/恢复逻辑处理。

### 阶段 H：测试与 UI 验收

先运行分层检查：

```powershell
npm run workspace:check
npm run typecheck
npm run lint
npm test
npm run build:web
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets -- -D warnings
npm run verify
git diff --check
```

重点自动化：

- 八个 BotItem key 的 TypeScript/Rust 序列化一致。
- 官方 Panel 差分报告与本项目后端同输入得到同结果。
- 后端失败时 Pinia 恢复旧 snapshot，UI 不显示成功反馈。
- `less` 五层闭环及两个 cfg 精确一行。
- `br_reroll` 在命令数据中仅出现一次，可被搜索并通过 Tauri 剪贴板插件精确复制。
- 安装新 ZIP 后不再要求已删除的 `bot_randomizer_options.json`。
- 旧 `skins` 状态文件不会让应用启动失败。
- `0.5.3 -> 0.5.4` 插件门禁升级，不覆盖有效更高核心版本。
- 新 Panel 的提取路径、大小和 SHA256。
- 当前 updater/退出逻辑测试全部继续通过。

UI 检查：

- 视口：`720x620`、`960x700`、`1280x800`。
- light/dark 两主题。
- `prefers-reduced-motion`。
- Bot Items 八行纵向滚动正常，无横向滚动、截字、遮挡或嵌套卡片。
- Nades 五段控件文字完整，键盘可操作，focus 可见。
- 失败文案有 `role="alert"`，pending/restart 不只依赖颜色表达。

### 阶段 I：签名安装包与隔离升级验证

先确认当前自更新改动已经合并到工作树，并确认签名环境变量存在但不输出值：

```powershell
Test-Path Env:TAURI_SIGNING_PRIVATE_KEY
Test-Path Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

缺少签名密钥时不能声称完成自更新发布。可以构建本地候选 NSIS，但必须明确“无 updater `.sig`，不可推送生产 self-update”。

签名构建按当前项目实际配置执行；若普通 `bundle:desktop` 已配置 `createUpdaterArtifacts: true`，不要额外叠加冲突 config：

```powershell
npm run bundle:desktop
npm run release:manifest
```

安装包构建最多尝试 5 次。若冷编译仍在运行，检查 `cargo/rustc/makensis`，不要启动重复构建。

隔离验证至少覆盖：

1. 全新安装 `0.5.4`、启动 30 秒、正常退出。
2. 从已验证 `0.5.3` 安装器升级到 `0.5.4`。
3. 升级过程中 NSIS hooks 能关闭旧进程，现有 updater 退出确认不死锁。
4. 用户 sentinel、应用配置和非本项目文件保留。
5. 卸载后分别断言程序文件、安装目录和应保留用户数据，不把“目录仍存在”简单报告为完全通过。
6. 安装后资源 ZIP、Panel、marker 和定制 DLL 摘要与构建证据一致。

最终报告：

- NSIS 绝对路径、大小、SHA256、修改时间。
- `.sig` 绝对路径和是否通过 Tauri updater 签名验证，不输出私钥。
- `Get-AuthenticodeSignature` 状态；Tauri minisign 与 Windows Authenticode 是两件事，不得混称。
- `dist-release/.../updater-prod.json` 的版本、size、SHA256 与安装器一致。

## 6. 用户真实 CS2 验收

候选安装器交给用户后，由用户在真实游戏树执行。执行 AI 先提供游戏目录备份命令/路径，并让用户确认已退出 CS2。

最短验收：

1. 用 `0.5.4` 覆盖安装并在应用中安装新插件包。
2. 选择 BOT 模式启动 CS2，控制台执行 `bot_nades less`，再执行 `bot_nades`，确认回显为 `less` 且没有 usage error。
3. 连续进行至少 3 个正常回合，观察烟、闪、高爆、燃烧瓶节奏；保存 `game/csgo/console.log`。
4. 确认开局没有短时间集体烟雾刷屏，同一 Bot 不会连续快速投掷，紧急灭火/拆包烟仍能出现。
5. 在应用里逐个切换八个 Bot Items；按提示重启 CS2 后，确认实际效果与官方 Panel 对同一开关的行为一致，尤其检查可能联动的 BotRandomizer 项。
6. 在应用“命令”页搜索 `br_reroll`，点击复制并粘贴到 CS2 控制台执行；确认 Bot 在下一次安全出生时重新抽取外观。
7. 确认贴纸、挂件、新皮肤库正常，Deathmatch/Arms Race 不错误切刀。
8. 退出 CS2 后重新打开应用，确认 `less` 和八项状态能正确回读。

用户提供日志后，执行 AI 必须先做结构化检查再判断成功；中间 console 命令成功不等于最终游戏行为成功。

## 7. 发布门槛

用户未确认真实游戏验收前：

- 不更新生产 D1 latest。
- 不上传/启用 R2 self-update。
- 不删除旧 R2 对象。
- 不发布 GitHub Release。
- 不发布群公告。

用户确认后才进入正式发布：

1. 再次校验候选安装包和 `.sig` 没有变化。
2. 按 `docs/network-self-update-r2-plan-0.5.3.md` 的现有架构发布 `0.5.4`；执行时以当前代码/生产 schema 为准，不照搬可能过期的调查值。
3. 先上传对象和写完整 artifact 元数据，验证 feed/download，再切换 latest/总开关；旧对象最后删除。
4. 验证 custom release API、Tauri dynamic feed、GET/HEAD 下载、CORS、夸克回退和 `/rizhi`。
5. GitHub 普通 push 最多尝试 1 次，不 force push；Release 上传也只在用户确认后进行。
6. 官网 Worker 部署最多 3 次；没有必要时不部署 Worker，只更新 D1/R2 元数据。

## 8. 回退方案

### 8.1 开发回退

- 只回退本次 `0.5.4` 新增/修改内容，不重置现有 UI/updater 脏改动。
- 资源 ZIP 从阶段 A 的时间戳备份恢复；用户下载的官方 ZIP 始终未修改。
- NadeSystem 保留 `v1.4.2` 和 `v1.4.3` 两套来源目录，回退时不删除历史证据。
- Bot Items 后端按单独提交/差分回退，不恢复已证明错误的新包旧 JSON 必需项。

### 8.2 游戏目录回退

- 必须使用用户确认的完整 `game/csgo` 备份回放。
- 不调用宽泛递归删除处理未知插件/用户文件。
- 回退后重新核验 `gameinfo.gi`、loader、完整插件链和 `-insecure`，不能只恢复一个 DLL 就宣称完成。

### 8.3 线上回退

- 先关闭 R2 推送总开关，阻断新 self-update。
- D1 latest 恢复到上一个已验证版本，保留夸克渠道。
- 只有权威元数据和下载验证恢复后才处理对象清理。

## 9. 最终交付报告模板

```text
结果：0.5.4 候选/正式版是否完成
Git：分支、HEAD、工作区状态、是否 push
上游：tag/commit、官方 ZIP/Panel 摘要
Bot Items：八项模型、官方差分报告、本项目同结果证据
Nades less：前端/后端/cfg/命令/回读结果
br_reroll：命令数据/解析/搜索/复制/真实游戏结果
道具节奏：源码基线、策略测试、三类入口、DLL 摘要
定制 ZIP：路径、大小、SHA256、entry 数、相对官方白名单差异、重复构建一致性
自动化：npm verify、Rust、Clippy、dotnet、diff-check 结果
UI：三视口、双主题、reduced-motion 截图/断言路径
安装器：EXE/.sig/release manifest 路径、大小、SHA256、签名状态
隔离安装：新装、0.5.3 升级、退出、卸载、sentinel 结果
真实 CS2：用户是否完成、日志路径、less/八项/节奏结论
线上：D1/R2/feed/API/GitHub 是否执行；未执行必须明确
待确认：仅列出本方案 4.3 中仍未批准的额外改动
剩余限制：未完成或只能由用户完成的验证
```

## 10. 禁止事项

- 禁止用旧 `v1.4.2` 定制 DLL 覆盖新包。
- 禁止继续把 `bot_randomizer_options.json` 当作新包必需文件。
- 禁止根据公开 tag 的滞后 Panel 类型实现四项 Bot Items。
- 禁止只加 `less` 按钮而不改 Rust 白名单、cfg、命令数据和 NadeSystem。
- 禁止直接应用失败的旧 patch 后手工忽略冲突。
- 禁止先扣钱/计数再把 entity creation 失败当成功。
- 禁止乐观 UI 成功、浏览器内置弹窗、横向溢出或缩小文字硬塞八项。
- 禁止全局替换版本字符串、清理未知文件、删除旧 app-data 或回退现有脏改动。
- 禁止没有 `.sig` 就声称 Tauri 自更新可发布。
- 禁止把候选安装包直接切生产；真实游戏验收是正式发布门槛。
