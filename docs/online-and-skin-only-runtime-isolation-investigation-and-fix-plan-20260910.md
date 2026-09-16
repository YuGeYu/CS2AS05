# Online 与只开换肤运行时问题调查及修复方案

日期：2026-09-10  
目标版本：0.5.13 后续修复构建

## 用户反馈

同一台机器出现两个问题：

1. 选择 Online 启动时提示：

   ```text
   [GAMEINFO_RECOVERY_REQUIRED] 当前 gameinfo 与已验证基线不一致，请先在 Steam 中验证游戏文件。
   ```

2. 选择“只开换肤”后：
   - 游戏仍显示 `BOT Difficulty: Low [1/3]`。
   - BOT 仍使用 CS2AS05 插件头像，没有恢复原生头像。
   - BOT 道具指令确实消失。

## 现场证据

当前玩家目录的 gameinfo 摘要：

| 文件 | SHA-256 | 语义 |
|---|---|---|
| `gameinfo.gi` | `AEFB44F51339F8422DA9F36EAC87E26E87D92129847BEDF2130227F8A0DFE780` | 当前 SkinOnly |
| `backup/Online/gameinfo.gi` | `B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8` | 官方基线 |
| `backup/WithBots/gameinfo.gi` | `3CA9C2342366EC08428916F1F60D2935AD9354EB2916C7F0253EB1404F5132CC` | BOT 模式 |
| `backup/SkinOnly/gameinfo.gi` | `AEFB44F51339F8422DA9F36EAC87E26E87D92129847BEDF2130227F8A0DFE780` | SkinOnly |
| `gameinfo.gi.official.bin` | `B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8` | 官方基线摘要 |

当前 sidecar 的四个摘要均与对应文件一致，说明官方基线本身没有损坏。

当前 Metamod/插件目录仍存在：

```text
addons/BotHider
addons/BotVision
addons/RayTrace
addons/metamod
addons/counterstrikesharp/plugins/InventorySimulator
addons/counterstrikesharp/plugins/RoundDamageRecap
```

CS2AS05 已移动的目录只有 `addons/BotController` 以及 CounterStrikeSharp 下列 BOT 插件；`BotHider`、`BotVision`、`RayTrace` 仍在 Metamod 加载路径中。SkinOnly gameinfo 仍然保留 `csgo/addons/metamod`，因此这些 native 扩展会继续加载。

## 根因一：Online 启动状态判断错误

`ensure_online_gameinfo_current()` 当前调用 `gameinfo_state()`，并依据 `state.status` 拒绝 `recoveryRequired`。这个判断把“活动文件当前是 bots/skin_only 等待切换”与“Online 官方基线损坏”混在一起。

Online 启动的正确条件应是：

- `backup/Online/gameinfo.gi` 存在且摘要等于 sidecar 的 `online_sha256`。
- `gameinfo.gi.official.bin` 存在且摘要等于 sidecar 的 `official_sha256`。
- sidecar 结构和资源版本可解析。

不应要求当前活动 `gameinfo.gi` 在启动前已经等于 Online；当前活动文件本来就可能是 BOT 或 SkinOnly，Online 启动正是要把它切回官方文件。

## 根因二：只开换肤隔离范围不完整

当前实现只隔离：

- `addons/BotController`
- CounterStrikeSharp 下的 BOT 管理插件目录

但上游 BOT 包还包含以下 native Metamod 扩展：

- `addons/BotHider`
- `addons/BotVision`
- `addons/RayTrace`
- 对应的 `addons/metamod/*.vdf` 加载入口

这些扩展仍会在 SkinOnly 中加载，导致：

- BOT 头像/隐藏逻辑仍由 BotHider 接管。
- BOT 视觉或状态逻辑仍由 BotVision 接管。
- RayTrace 及其依赖继续驻留并可能被 BOT 插件使用。
- 仅移动 CounterStrikeSharp 插件不足以恢复原生 BOT 行为。

`BOT Difficulty: Low [1/3]` 还可能来自 CS2 原生界面显示的当前难度变量或已有 `botprofile.vpk`，因此不能仅凭该文本判断某一个插件仍在运行；但插件头像现象与上述 native 扩展未隔离相互印证。

## 修复目标

- Online 可从 BOT/SkinOnly 状态直接切回官方模式，只要官方基线文件本身完整。
- SkinOnly 只保留 Inventory Simulator 和必要的 CounterStrikeSharp/Metamod 核心，不加载任何 CS2AS05 BOT 行为扩展。
- 切回 BOT/Online 时完整恢复之前受管的 native 扩展和 CounterStrikeSharp 插件。
- 不删除玩家文件，不覆盖玩家自定义难度和配置。
- 所有隔离动作可回滚、可识别、可诊断。

## 推荐实现

### 1. 分离 Online 基线校验

将 `ensure_online_gameinfo_current()` 改为只校验官方基线资源：

- 解析 sidecar。
- 比较 `backup/Online/gameinfo.gi` 与 `online_sha256`。
- 比较 `gameinfo.gi.official.bin` 与 `official_sha256`。
- 可选校验 WithBots/SkinOnly 摘要，但不以当前活动文件模式作为拒绝条件。

在通过后执行 `set_mode_with_app(..., "online")`，由 `write_mode_at()` 将活动文件切换为 Online。

`gameinfo_state()` 仍可报告当前模式，但不再把“已知 SkinOnly/BOT 活动状态”解释为官方基线损坏。

### 2. 扩展 SkinOnly native 隔离清单

将当前单一 `SKIN_ONLY_DISABLED_METAMOD_DIR` 改为受管目录映射，例如：

```text
addons/BotController -> addons/.cs2as-skin-only/BotController
addons/BotHider      -> addons/.cs2as-skin-only/BotHider
addons/BotVision     -> addons/.cs2as-skin-only/BotVision
addons/RayTrace      -> addons/.cs2as-skin-only/RayTrace
```

同时处理 `addons/metamod` 下与这些扩展对应的 VDF 入口：

- 优先移动/重命名受管 VDF 到 SkinOnly 隔离目录。
- 不移动 `addons/metamod` 核心目录本身。
- 不移动 Inventory Simulator 所需的 CounterStrikeSharp/Metamod 核心。

具体 VDF 文件名应从当前 ZIP 条目和实际安装目录逐一建立白名单，禁止按模糊通配符删除未知文件。

### 3. 状态记录与恢复

- 将 native 扩展和 VDF 的移动结果写入现有 SkinOnly 状态记录或独立版本化状态 JSON。
- 记录源路径、隔离路径、文件存在性和迁移版本。
- 进入 SkinOnly 时只移动清单内存在的文件。
- 切回 BOT/Online 时按记录恢复，不凭目录猜测未知文件。
- 如果目标路径已存在且内容不同，停止并报告冲突，不覆盖玩家文件。
- 失败时按已完成顺序回滚本次移动。

### 4. BOT Difficulty 的处理边界

- SkinOnly 不写入、不加载 CS2AS05 的 `overrides/botprofile.vpk`。
- 不把 `Low` 强行改成其它值，也不删除玩家的 Low/Medium/High 备份。
- 在界面上将 Difficulty 显示标记为“由 CS2/地图原生决定”，避免误导玩家认为这是 CS2AS05 当前接管的难度。
- 真实验收时分别确认：原生界面文本、插件头像、道具指令、地图关卡推进四项行为。

## 测试方案

### Rust 测试

1. 当前活动为 SkinOnly，官方备份和 official bin 正常，Online 基线校验通过并可切换。
2. 当前活动为 BOT，官方备份正常，Online 基线校验通过并可切换。
3. Online 备份或 official bin 摘要错误时仍拒绝启动。
4. SkinOnly 隔离 BotController、BotHider、BotVision、RayTrace 目录及对应 VDF。
5. 切回 BOT/Online 恢复所有被移动目录和 VDF。
6. 隔离冲突或中途失败时只回滚本次动作，不覆盖未知文件。

### 静态契约

- ZIP 条目、native 清单、VDF 清单与恢复映射一致。
- `gameinfo` 三状态摘要和 sidecar 兼容旧安装。
- `cargo check`、`cargo test`、`cargo fmt --check`、`npm run typecheck`、`git diff --check` 通过。

### 真实 Windows/CS2 验收

- 当前玩家目录从 SkinOnly 直接启动 Online，确认不再出现 `GAMEINFO_RECOVERY_REQUIRED`。
- Online 模式确认活动 `gameinfo.gi` 恢复为官方摘要。
- 再进入 SkinOnly，确认 Inventory Simulator 正常、BOT 原生头像恢复、BOT 道具指令不出现。
- 在创意工坊闯关地图确认 BOT 全灭后下一关正常推进。
- 切回 BOT 模式确认 CS2AS05 BOT 难度、头像、道具和插件功能恢复。

## 发布边界

当前 0.5.13 签名候选不包含本方案的 Online 基线校验重构和完整 native BOT 隔离修复。实现并通过真实 CS2 验收前，不应宣称 Online 与 SkinOnly 两个问题已解决，也不应覆盖线上发布资产。
