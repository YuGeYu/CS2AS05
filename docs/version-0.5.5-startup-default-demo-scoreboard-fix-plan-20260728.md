# CS2AS05 0.5.5 启动检查、默认目录与 Demo 记分板修复实施方案

> 文档日期：2026-07-28
> 交接对象：下一位【实际执行 AI】
> 本文可独立执行，不依赖此前聊天上下文。
> 本轮只调查并制定方案，没有修改产品运行代码、没有重建安装包、没有发布。
> 版本继续保持 `0.5.5`，不要因本次修复升级到 `0.5.6`。

## 1. 目标和完成定义

必须同时完成以下三组要求：

1. 程序每次冷启动时都执行一次自动更新检查；人机预设中的道具频率首次默认是 `less / 较少`；BOT 物品首次默认八项全开。
2. 玩家选择并验证 CS2 游戏目录后，自动把该 **CS2 根目录本身** 加入对局复盘扫描目录，默认深度 `5`。
3. 新旧 Demo 的记分板不能再因为当前实现固定返回空数组而永远为空。最新版实测 Demo 必须显示完整 10 人记分板；旧 Demo 要走兼容解析和 BOT 身份回退，不能继续把解析器能力缺失误报成“文件未提供身份数据”。

完成不以“代码已改”或“时间线仍能显示”为准，而以本文第 12 节三份真实 Demo 的重放结果、缓存迁移结果和安装版冷启动结果为准。

## 2. 当前工作区和操作约束

仓库：`E:\CS2AS05`，分支 `main`，当前工作区已有大量未提交的 0.5.5 功能改动和未跟踪文件。开始前执行：

```powershell
git status --short --branch
git diff --check
```

不要 `reset`、`checkout`、`restore`、`clean`，不要覆盖或删除既有改动。当前安装器：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe
```

它在本次修复前构建，修复后必须重建，不能继续把旧 EXE 当作修复候选。不要新建分支。真实 CS2 对局验证由用户执行；执行 AI 可以只读解析用户现有 `.dem`。

## 3. 已确认的运行事实

### 3.1 更新检查现在存在，但所有权不可靠

当前链路：

```text
AppShell 初始 current=overview
  -> OverviewView
  -> SupportActions.vue onMounted
  -> check(false)
  -> checkForSoftwareUpdates(false)
  -> requestSoftwareUpdate({ manual: false })
```

因此正常首次打开概览页时通常会检查更新，但检查逻辑属于会被动态视图挂载/卸载的 `SupportActions`，不是应用启动生命周期。离开再回概览还会重复触发；将来调整首页结构也可能静默丢失启动检查。

正确修复是把自动检查提升到 `App.vue` 下常驻的单一协调器，`SupportActions` 只展示共享状态和提供手动操作。自动检查继续传 `manual=false`；用户点击“检查更新”才传 `manual=true`。保留 `state.ts` 的 in-flight 去重、普通/推荐版本关闭语义和现有 Tauri 签名更新流程。

### 3.2 Panel 首次默认值已经有正确实现，重点是守住契约

`src-tauri/src/services/panel.rs` 当前已经规定：

- `bot_aim mixed`
- `bot_nades less`
- BOT Items 的八个官方 key 缺失时视为 `true`
- 新安装测试明确断言八项全开和两个 cfg 各只有一条 `bot_nades less`

`src/stores/panel.ts::refresh` 会先执行 `initializePanelDefaults(root)`，再读取 snapshot。不要为了“确保默认值”在 Vue 页面写 `?? true` 或每次启动强制覆盖磁盘，这会破坏旧用户的显式选择。

本次只需补强启动/安装路径测试，确保：全新安装使用 `less + 八项 true`；已有 `normal/off/全关` 等合法选择继续保留；CS2 运行中初始化为 `deferred`，退出后后续 refresh 才落盘。

### 3.3 默认 Demo 根目录尚未与 CS2 选择联动

当前 `addDemoRoot(path, scanDepth=2)` 只在用户手动添加目录时调用。数据库实测只有用户后来添加的：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo
scan_depth=5
```

但产品已选择的 CS2 游戏根目录是：

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive
```

两者不是同一路径。用户要求的默认目录是玩家选择的 CS2 根目录本身，深度 5，而不是只建议 `game\csgo\replays`。

### 3.4 记分板为空的直接根因不是 Demo 缺数据

`src-tauri/src/services/demo.rs::parse_report` 当前明确：

```rust
wanted_player_props: vec![],
parse_ents: false,
// ...
players: vec![],
```

`parse_ents=false` 会令 vendor parser 在 `second_pass/entities.rs::parse_packet_ents` 入口直接返回；玩家姓名、SteamID、队伍和事件身份映射依赖 `CCSPlayerController` 实体。前端的空状态只是显示这个固定空数组。

此外，`import_file` 对同路径、同 size、同 mtime 且 `status=done` 的记录直接返回，完全不比较报告 schema/parser adapter/metrics 版本。现有 `retry_demo_parse` 最终仍调用它，所以旧的空记分板报告连点击“重新解析”也会被缓存短路。

## 4. 真实样本调查证据

本机权威数据库：

```text
C:\Users\GOPtZ\AppData\Roaming\com.aipc.cs2botimprover\demo-review\demo-review-v1.sqlite3
```

### 4.1 最新版实测 Demo

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260728-1717-de_inferno-advent.dem
size=63,819,366
map=de_inferno
database result=9 rounds / 79 kills / done / players=[]
header patch_version=14172
```

对照实验：当前事件模式 `parse_ents=false` 耗时约 `136ms`，`roster=[]`，所有 `user_name/user_steamid/attacker_name/attacker_steamid` 都是 `null`。临时诊断改为 `parse_ents=true` 并请求终局统计字段后耗时约 `1391ms`，稳定得到 10 名玩家的姓名、SteamID64 和队伍编号，击杀/伤害事件身份也全部恢复。说明最新版文件本身有数据，主修复路径已被真实样本证明。

### 4.2 两份旧 Demo

```text
D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\replays\auto-20260615-2018-de_anubis-259.dem
size=26,962,991
current event-only result=done, 1 round / 129 kills
entity-enriched diagnostic=IllegalPathOp

D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\cs2_insight_preview.dem
size=54,936,092
current event-only result=done, 16 rounds / 111 kills
entity-enriched diagnostic=only one real player in roster; BOT event identities remain null
```

第二份旧样本暴露 vendor 的 BOT 过滤：`first_pass/stringtables.rs` 和 controller 收集逻辑会丢弃 `steamid == 0`；BOT 又不能共用 `SteamID=0` 作为唯一键。第一份旧样本说明不能只打开实体解析后宣布所有旧 Demo 已修复，必须有旧协议容错和 userinfo/BOT 回退。

上游 `LaihoE/demoparser` 的远端 HEAD 在调查时仍为固定提交 `ba39cc44cd5abfd7f34df2b3c0a7dd3630048311`，没有更新版可直接升级解决这些问题。

## 5. 总体实现决策

采用“单次业务解析请求、内部多来源合并、明确质量状态”的方案：

1. **身份主来源**：实体解析后的 controller roster，真人以 `SteamID64` 为稳定主键。
2. **BOT/旧 Demo 回退**：保留 `userinfo` 中 `steamid=0` 的记录，用 `user_id/controller slot` 形成局部稳定键，绝不把所有 BOT 合并成一个 `SteamID=0` 玩家。
3. **统计主来源**：若终局 controller 属性可读，使用 `kills_total/deaths_total/assists_total/damage_total/headshot_kills_total` 的最终值。
4. **统计回退**：按带身份的 `player_death/player_hurt` 聚合 K/D/A、伤害和爆头；每个字段携带来源/完整性，不能把推测值伪装成官方终局值。
5. **旧协议容错**：严格实体解析遇到已确认的 `IllegalPathOp` 时，允许在单个 `svc_PacketEntities` 消息边界丢弃该消息并等待后续 full packet 恢复；只对该已知错误启用，记录 warning。不能在未知 bit offset 上继续读。
6. **最终降级**：如果实体路径仍失败，使用保留 BOT 的 userinfo + 原始 game event userid 映射构建事件记分板。只有所有身份来源都确实为空时才标记 `scoreboardStatus=unavailable`，并展示具体 parser 错误/缺失来源，不再使用当前误导性固定文案。
7. 第三方原始来源和本地补丁分开。业务聚合放在 `src-tauri/src/services/demo/` 适配层；vendor 只做通用 parser 能力补丁，并在 `third_party/demoparser/UPSTREAM.md` 逐项记录。

不要引入 Python、Node sidecar 或浏览器文件 API。当前 Rust parser 已证明最新版路径可用；先完成最小 Rust 补丁。只有安全的消息边界容错仍无法让指定旧样本产生任何身份时，才评估独立 parser worker，不能先引入大型运行时。

## 6. 数据契约

把报告升级为 `schema_version=2`，建议 DTO 至少调整为：

```rust
pub struct DemoPlayer {
    pub key: String,                 // steam:<steamid64> 或 bot:<user_id/controller>
    pub steam_id: Option<String>,    // BOT 必须是 None，不能写 "0"
    pub user_id: Option<i32>,
    pub name: Option<String>,
    pub is_bot: bool,
    pub team_number: Option<i32>,    // 2=T, 3=CT；未知保持 None
    pub team: Option<String>,
    pub kills: Option<u32>,
    pub deaths: Option<u32>,
    pub assists: Option<u32>,
    pub damage: Option<u32>,
    pub headshots: Option<u32>,
    pub identity_source: String,     // controller/end_message/userinfo/event
    pub stats_source: String,        // controller_total/event_aggregate/partial
}

pub struct DemoDataQuality {
    pub scoreboard_status: String,   // complete/partial/unavailable
    pub warnings: Vec<String>,
    pub entity_parse_status: String, // strict/recovered/failed
}
```

`DemoReport` 增加 `data_quality`；`parser_adapter_version` 升到 `2`，`metrics_version` 升到清晰的新值，例如 `scoreboard-v2`。前端 TypeScript 类型同步修改。

身份合并顺序固定：有效 SteamID64 > controller/user slot > 同一事件内 user id > 名称最后回退。名称不是全局唯一键；玩家改名、同名 BOT、重连都不能仅按名称盲目合并。观战者不进入主记分板。队伍编号只按可靠来源映射，未知时单独显示“未识别队伍”。

事件聚合规则必须写成纯函数并单测：

- `player_death`：victim deaths +1；有效 attacker 且不是自杀时 kills +1；assister 有效时 assists +1；是否扣除 teamkill 必须与 controller total 对照后冻结，不能猜。
- `player_hurt`：优先用 controller `damage_total`；事件回退时明确采用并测试 `dmg_health` 的口径，避免过量伤害造成统计漂移。
- `headshot=true` 的有效击杀计入 headshots。
- world/self kill、空 attacker、断线重连、同名玩家和 BOT `steamid=0` 分别有测试。

## 7. 后端文件级实施

### 7.1 拆分 Demo 业务层

当前 `src-tauri/src/services/demo.rs` 已过度集中。至少拆出：

```text
src-tauri/src/services/demo/
  mod.rs
  parser_adapter.rs
  scoreboard.rs
  repository.rs
  roots.rs
```

若为控制变更量暂不拆全部，也必须让 `parse_report` 调用独立、可单测的 `build_scoreboard(output)`，不要继续在一个函数内混合文件、parser、回合和数据库逻辑。

### 7.2 ParserInputs

主解析请求：

```rust
parse_ents: true
wanted_player_props: [
  "kills_total",
  "deaths_total",
  "assists_total",
  "damage_total",
  "headshot_kills_total",
  "team_num"
]
wanted_events: [
  existing round/bomb events,
  "player_death", "player_hurt",
  "player_connect", "player_disconnect"
]
```

仍用 `ForceSingleThreaded` 和 `catch_unwind`。最新版 63.8MB 样本的 1.39 秒是当前性能基线；单 Demo 目标不超过 5 秒，后台并发仍为 1，不能阻塞 Tauri/UI 主线程。

### 7.3 Vendor 最小补丁

集中检查并修改：

- `first_pass/stringtables.rs`：保留非空姓名的 BOT userinfo，不因 `steamid==0` 丢弃。
- `second_pass/game_events.rs`：entity 映射失败时按原始 userid 回退到 userinfo，并输出 user id、姓名和可选 SteamID。
- controller roster：真人按 SteamID 去重；BOT 按 controller/pawn/user slot 去重，不执行 `should_remove(Some(0))`。
- `PlayerEndMetaData` 或新的公开 roster DTO 暴露 `user_id/controller id/is_bot`，避免业务层猜键。
- 对 `IllegalPathOp` 的恢复只能位于长度已知的 packet-entities 消息边界；统计 dropped packet 数并暴露 warning。不要广泛吞掉 `MalformedMessage`、OOM 防护或所有 parser error。

更新 `third_party/demoparser/UPSTREAM.md`，记录每个本地补丁、动机、覆盖样本和测试。不要改上游 commit 声明。

### 7.4 数据库迁移和强制重解析

将 `PRAGMA user_version` 升级，不要只保留 `CREATE TABLE IF NOT EXISTS`。`demo_files` 增加可查询字段：

```text
report_schema_version
parser_adapter_version
metrics_version
scoreboard_status
```

迁移后把以下记录视为 stale：

- `report_schema_version < 2`
- `parser_adapter_version != 2`
- `report_json` 中 `players=[]` 且旧 adapter 为 1

修改 `import_file` 签名或内部选项，区分 `force_reparse`。fingerprint 相同只在版本相同且 scoreboard 不 stale 时短路。`retry_demo_parse` 必须传 `force_reparse=true`。

应用启动后把 stale 记录按 mtime 从新到旧排队、单并发后台重解析；不要在 Tauri setup 主线程同步解析所有历史文件。UI 显示“需更新解析/重新解析中”。失败保留旧时间线 JSON 作为最后可用结果，同时记录新 parser 错误，不能事务中途清空旧报告。

### 7.5 默认 CS2 根目录

新增幂等 command，例如：

```text
ensure_default_demo_root(root_path, scan_depth=5)
```

后端必须先复用 `cs2::normalize_root/inspect` 验证确为 CS2 根目录，再 canonicalize。语义：

- 若不存在，插入 enabled root，深度 5，`origin=selected_cs2_root`。
- 若已存在且是自动 root，保持 enabled 并同步深度 5。
- 若同路径是用户手动配置，保留用户后来显式设置的 enabled/depth，不在每次启动覆盖。
- 选择新 CS2 根目录时加入新 root；旧自动 root 默认禁用而非删除，用户手动 root 不动。
- 同路径大小写、斜杠差异不得重复。

数据库 `demo_roots` 增加 `origin`；旧行迁移为 `manual_legacy`。前端在 `cs2.selectRoot` 成功后或 AppShell 对规范化 root 的 watcher 中调用一次该 command，然后刷新 roots/watcher。不要只依赖浏览器 localStorage 事件。

`src/services/tauri/demo.ts::addDemoRoot` 的手动默认深度也改为 `5`，目录 UI 新建项默认显示 5。扫描器继续硬限制最大 5。默认根较大，扫描必须在 blocking worker 中，按最近文件优先且不能读取非 `.dem` 文件内容。

## 8. 启动更新和默认值实施

### 8.1 常驻更新协调器

建议新增 `src/features/software-updates/coordinator.ts` 或 Pinia store，并在 `App.vue` 的唯一 `onMounted` 中：

1. `initializeSoftwareUpdaterState()`。
2. 调用一次 `checkForSoftwareUpdates(false)`。
3. 保存 checking/message/latestRelease/presentedRelease 到共享响应式状态。
4. 根据 `shouldPresentRelease` 决定是否打开全局 `SoftwareUpdateModal`。

把 modal 所有权提升到 `App.vue` 下常驻组件；`SupportActions.vue` 只消费共享状态并调用 `check(true)`、下载、稍后安装等 action，不再在自身 `onMounted` 自动检查。这样 Intro 是否显示、当前导航页是什么都不会影响检查。

加单例启动防护，Vue 开发模式或组件重挂载不能产生第二次自动 fetch。一次失败不阻止应用启动，状态区显示可恢复错误；手动检查仍可重试。

### 8.2 Panel 默认契约

保留后端权威默认：

```text
mode=bots
difficulty=Low
aim=mixed
nades=less
botItems=profiles,agents,music,weapons,knives,gloves,stickers,charms 全 true
```

检查 `install_bot_package -> restore_preferences -> initialize_panel_defaults` 的真实调用顺序，确保安装 ZIP 后八个官方 key 都显式写为 JSON `true`。前端 `PresetsView`、`BotItemsView` 继续只显示 snapshot；snapshot 未就绪时保持 disabled/loading，不用视觉 fallback 假装全开。

## 9. 前端报告交互

修改 `src/views/DemoReviewView.vue`：

- 按 `team_number/team` 分组显示 T、CT、未识别队伍，真人和 BOT 都有独立行。
- BOT SteamID 列显示 `BOT`，不要显示 `0` 或伪造 SteamID。
- 数值列使用等宽数字并保持固定最小列宽；长名称省略并提供 `title`。
- `partial` 显示明确的 warning，例如“已从事件恢复 10 名玩家；部分终局统计不可用”，颜色不是唯一提示。
- `unavailable` 显示具体原因和“重新解析”按钮，不再写“此文件未提供可验证的玩家身份数据”。
- 报告 schema 旧时显示“报告由旧解析器生成，需要重新解析”，而不是空记分板。

继续使用现有 Lucide 图标、紧凑桌面工具样式、44px 最小交互区、可见 focus、`aria-live` 错误反馈；不新增 Hero、卡片套卡片、浏览器原生目录能力或装饰动画。720x620 时记分板允许表格容器水平滚动，页面本身不能横向溢出。

## 10. 自动化测试矩阵

### 10.1 更新检查

- App 冷启动恰好调用一次 `checkForSoftwareUpdates(false)`。
- Intro 打开、非概览导航、SupportActions 重挂载都不影响启动检查且不产生第二次 fetch。
- 手动按钮调用 `true`，自动/手动并发共享 in-flight。
- 网络失败不阻塞 App，随后手动重试成功。

### 10.2 默认值

- 全新 fixture：两个 cfg 均唯一 `bot_nades less`，八个 core key 均为 `true`。
- 升级 fixture：显式 `normal/off`、八项全关或混合值全部保留。
- CS2 运行中初始化 deferred，不写磁盘；停止后 refresh 初始化。
- 页面只回读 snapshot，不存在 `?? true` 伪默认。

### 10.3 默认根目录

- 选择规范 CS2 root 自动创建 depth 5 root。
- 重启幂等，不重复。
- 切换 root 禁用旧自动 root，不动手动 root。
- 手动改 depth 后重启不被强制覆盖。
- 非 CS2 目录、大小写重复、根不存在分别有稳定结果。

### 10.4 Parser/scoreboard

- controller roster：10 真人、两队、SteamID64 不重复。
- 多个 BOT：`steam_id=None` 且 key 不同，不被合并。
- 同名、改名、重连、观战者、自杀、world kill、teamkill、空 assister。
- strict entity pass、`IllegalPathOp` 消息级恢复、userinfo-only fallback。
- v1 空玩家 `done` 报告自动标 stale；强制 retry 真正改写 v2。
- 解析失败事务保留旧报告；单个坏 Demo 不影响队列。

Rust 测试优先使用脱敏/最小 fixture 或聚合纯函数。真实大 Demo 可做 `#[ignore]` 本机集成测试并在交付前显式运行，不能把 60MB 用户文件提交到 Git。

## 11. 实施顺序

1. 保存当前 git 状态、数据库 schema/行快照和三个 Demo 的 size/hash/header 证据。
2. 先写 parser diagnostic/integration tests，分别复现最新 10 人成功、旧 `IllegalPathOp`、BOT `steamid=0` 丢失。
3. 完成 vendor userinfo/BOT/消息边界容错补丁，再做业务 scoreboard 纯函数。
4. 升级 DTO/report schema 和数据库 migration，修复 stale/force reparse。
5. 实现默认 CS2 root depth 5 的后端幂等 command 及前端选择联动。
6. 把更新检查提升到应用级协调器，补守护默认值测试。
7. 完成 UI 的 complete/partial/unavailable 状态和分队记分板。
8. 跑自动化、三份真实 Demo、Web 构建，再重建 0.5.5 NSIS 安装器。
9. 隔离安装后验证冷启动检查、默认根/深度、旧报告迁移。最后由用户再打一场 BOT 新 Demo 做真实验收。

## 12. 必须通过的真实样本验收

对三个文件先记录 SHA256，复制到只读测试位置或原路径只读打开，不修改真实 Demo。

### A. 最新版 `auto-20260728-1717-de_inferno-advent.dem`

- 仍能解析地图、回合和时间线。
- 记分板恰好 10 名非观战玩家，姓名、SteamID64、T/CT 均非空。
- K/D/A、伤害、爆头有值，并抽查事件聚合与 controller total 的差异。
- `scoreboardStatus=complete`；entity strict 成功；解析时间和峰值内存留档。

### B. 旧版 `auto-20260615-2018-de_anubis-259.dem`

- 不再因 `IllegalPathOp` 整体失去记分板。
- 至少从安全恢复或 userinfo/event fallback 得到实际参与者行；不能仍是固定空数组。
- 若终局字段确实缺失，允许 `partial` 和 `--`，但必须明确哪些字段来自事件、哪些不可得。

### C. 旧版 `cs2_insight_preview.dem`

- 不只显示唯一真人；BOT 各自独立成行。
- BOT SteamID 显示 `BOT`，不会被合并为一个 `0`。
- 时间线中的 BOT actor/target 姓名不再全为 `--`。

### D. 缓存迁移

在现有数据库副本上启动新版：三个旧 `done + players=[]` 报告必须被识别为 stale 并后台重解析。点击重试必须绕过 fingerprint 快捷返回。重启后 v2 结果可稳定回读，不重复无限解析。

## 13. 验证命令

先执行聚焦测试，再执行简化全量验收：

```powershell
npm run typecheck
npx vitest run tests/software-update-check-state.spec.ts tests/support-actions.spec.ts tests/panel-data.spec.ts
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo test --manifest-path .\src-tauri\Cargo.toml scoreboard
cargo test --manifest-path .\src-tauri\Cargo.toml demo_root
cargo test --manifest-path .\src-tauri\Cargo.toml
npm run build:web
git diff --check
```

大型项目按当前环境要求简化 UI 验收，但仍检查 `720x620`、`960x700`、`1280x800` 的记分板无重叠、键盘 focus、亮暗主题和 reduced-motion。真实网站/UI 用 Codex 内置浏览器；桌面程序真实 CS2 效果由用户执行。

确认功能和安装隔离验证通过后才运行：

```powershell
npm run bundle:desktop
Get-FileHash '.\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.5_x64-setup.exe' -Algorithm SHA256
```

构建后核对 EXE、`.sig`、version、资源 ZIP、安装/升级/卸载。不要自动发布生产；若后续明确要求发布，GitHub push 最多一次，网站部署最多三次。

## 14. 停止条件和禁止的假修复

出现以下任一情况不得宣称完成：

- 只把 `players: vec![]` 改成 roster，却没有 BOT/旧 `IllegalPathOp` 测试。
- 最新样本只有姓名但 K/D/A/伤害仍全 `--`。
- 把 BOT 全部用 SteamID `0` 合并，或按名字全局合并同名玩家。
- 旧 v1 报告仍被 `status=done` 缓存短路。
- 自动检查仍只依附 Overview/SupportActions 的挂载。
- 每次启动强制把旧用户的 Nades 改成 `less` 或把 BOT Items 强制全开。
- 默认 root 加成 `game\csgo` 而不是玩家选择的 CS2 根目录，或默认深度仍为 2。
- 为容错而吞掉所有 parser error，导致损坏数据被标成 `complete`。
- 修复后仍交付本次修复前的旧安装器。

## 15. 实际执行 AI 最终报告模板

```text
结果：0.5.5 本地修复候选 / 可安装测试版
保留的既有改动：branch、HEAD、dirty files 摘要
启动更新：冷启动次数、manual=false、失败/手动重试、modal 所有权
默认值：新安装 less + 八项全开；升级显式值保留
默认目录：选择的 CS2 root、origin、depth=5、重启/切换幂等
报告 schema：v2、adapter/metrics 版本、stale migration 数量
最新版 Demo：SHA256、10 人身份、队伍、统计、耗时、quality
旧 Demo 1：IllegalPathOp 恢复路径、玩家数、缺失字段、quality
旧 Demo 2：真人/BOT 数、BOT key、事件姓名、quality
自动化：TS/Vitest/Rust/fmt/build-web/UI
安装器：绝对路径、size、SHA256、.sig、隔离安装结果
用户实机：新录制 Demo 是否再次通过
剩余限制：具体 parser warning，不得写笼统“文件没数据”
Git/线上：是否提交、push、部署；未要求时保持未执行
```

只有最新版 10 人记分板、新旧样本兼容回退、旧缓存强制升级、冷启动更新和默认 root/defaults 全部复现后，才可将本任务标记为完成。
