# Nades 上游 1.4.4 对齐重建准备（2026-09-06）

## 结论

玩家反馈成立。当前“最多（`bot_nades max`）”实际受到了项目自定义节流分支的额外限制，不能等同于上游 1.4.4 的 `max` 行为。

当前 `src-tauri/resources/CS2BotImprover.zip` 内的 `NadeSystem.dll` 为 82,944 bytes，SHA-256 为 `2668B41B019F2BDBB7C89051136B95EFD0E46A7A044553408FDF33048B4A2654`。它与 `third_party/CS2-Bot-Improver-v1.4.4/nades-pacing/bin/Release/net10.0/NadeSystem.dll` 一致，**不是**旧的 1.4.3 DLL。

但 `nades-pacing` 不是上游发布目录 `addons/counterstrikesharp/plugins/NadeSystem` 的原始源树：前者新增 `NadePacingPolicy.cs` 并修改了 NadeSystem 的决策、回放和生命周期代码。安装包当前正使用这个改造分支的产物。

## 已定位的直接限制

`nades-pacing/NadePacingPolicy.cs` 对全部模式调用 `CanReserveHardLimit()`。即使模式是 `max`：

- 每个 BOT 每回合闪光最多 2 枚；
- 每个 BOT 每回合烟雾、HE、燃烧弹各最多 1 枚；
- 这不是上游 `TryReplay()` 的 `max` 分支原有的“no limits”。

同一改造还引入以下全局节流，`max` 也会被影响：

- 单个 BOT 的非紧急投掷至少间隔 5 秒；
- 同队非紧急投掷至少间隔 0.5 秒；
- 同一时刻每个 BOT/队伍只允许一个非紧急预留；
- 开局 15 秒内，每个 BOT 最多一次、同侧还有总量/烟雾/进攻道具上限。

因此即使 `bot_nades max` 已真正进入插件，它仍不可能表现为上游 `max` 的高频、无轮次数量限制行为。

## 不是根因的内容

1. `bot_nades` 不是原生 CS2 的普通 convar，而是 NadeSystem 在 `Load()` 中通过 `AddCommand("bot_nades", ...)` 注册的插件命令。
2. 上游默认 cfg 同样设置 `bot_allow_grenades 0`、`sv_bot_buy_grenade_chance 0` 及多项原生购买权重为 0；这是因为 NadeSystem 自行扣款并生成投掷物，不能将原生购买开关改成 1 当作本问题的修复。
3. 当前 ZIP 中已有 NadeSystem 的 52 个地图/类型 JSON 和 1.4.4 pacing DLL；问题不应先归因于“没有投掷点 JSON”或“仍打包 1.4.3 DLL”。

## 还需在重建前固定的差异

上游原始源树为：

`third_party/CS2-Bot-Improver-v1.4.4/addons/counterstrikesharp/plugins/NadeSystem`

当前实际编译源树为：

`third_party/CS2-Bot-Improver-v1.4.4/nades-pacing`

两者在 `NadeSystem.cs`、`NadeSystemPlugin.ZoneDetection.cs`、`NadeSystemPlugin.Lifecycle.cs`、`NadeSystemPlugin.Decision.cs` 等核心文件均不相同。后续不能只把 `NadePacingPolicy` 的少数常量调大；那会继续保留对 `max` 行为的非上游约束，也无法证明完整能力等同。

## 重建范围

目标是用上游 1.4.4 的 NadeSystem 行为作为唯一基线，再单独、可审计地叠加确有必要的 CS2AS05 集成，不从 `nades-pacing` 复制决策/节流逻辑。

1. 建立一个新的、可重复构建的上游基线目录；源文件逐项与 `addons/counterstrikesharp/plugins/NadeSystem` 比对，记录上游 revision、文件清单、SHA-256 和依赖版本。
2. 从基线构建 `NadeSystem.dll`，将 DLL、deps、pdb、全部 `grenades/*.json` 一并写入候选 ZIP。
3. 仅保留产品集成所必需的改变，例如包 marker、安装事务、完整性校验和 UI 状态回读；不修改 NadeSystem 的命令、决策、回放、冷却、购买和投掷实现。
4. UI 中将 `max` 表述为“上游最大投掷模式”，不再把任何自定义节流描述成上游能力。
5. 将对 `bot_nades` 的应用从“只写 cfg 即假定成功”提升为“写入后重启 BOT 对局，并用服务器控制台/插件日志确认 `bot_nades set to max`”。插件加载顺序尚需真实 CS2 验证。

## 验收门槛

### 静态门槛

- 新 DLL 的构建输入、输出 SHA、deps、插件目录 JSON 数量与上游清单逐项可回读。
- `NadePacingPolicy.cs` 不在候选编译输入、DLL 依赖或 ZIP 内容中。
- 对上游 NadeSystem 源的差异必须为零；任何有意集成差异独立列出并有测试。
- `bot_nades off|less|normal|more|max` 五个值都能通过命令注册、cfg 写入和解析测试。

### Windows/真实 CS2 门槛

- 在至少一张带上游数据的官方地图上，日志出现 NadeSystem 加载及当前地图投掷点加载数量。
- 设置 `max` 后，控制台明确回显 `bot_nades set to max`，不是只看助手 UI 显示值。
- 在相同地图、BOT 数量、回合设置与上游 1.4.4 对照环境下，记录完整回合的投掷事件，确认不存在 CS2AS05 额外的每 BOT 每种 1 枚、开局一次或 5 秒间隔限制。
- `normal`、`more`、`less`、`off` 保持上游各自语义；不要用修复 `max` 的方式破坏其他档位。

## 当前边界

本文件只完成调查和重建准备，不替换 DLL、不改 ZIP、不构建安装器、不覆盖玩家 CS2 目录、不发布。真实游戏内投掷频率仍必须由用户执行最终验收。
