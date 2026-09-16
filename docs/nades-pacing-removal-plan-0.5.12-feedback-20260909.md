# bot_nades 道具节奏限制取消方案

日期：2026-09-09  
工作区：`E:\CS2AS05`  
反馈来源：0.5.12 正式版本玩家反馈

## 结论

玩家反馈与当前源码一致：`bot_nades max` 看起来正常，是因为 `TryReplay()` 的模式分支没有额外轮次上限；但随后所有投掷仍会进入我们新增的 `NadePacingPolicy.TryBegin()`。该策略对 `max`、`more`、`normal`、`less` 统一施加自定义节流，因此 `more` 到 `less` 的实际投掷量明显低于上游预期。

本次修复目标不是把某几个常量调大，而是删除 CS2AS05 自定义道具节奏层，恢复上游 NadeSystem 1.4.4 行为。这样才能让 `more`、`normal`、`less` 各自按上游定义生效，并避免未来继续出现“UI 显示一个模式，实际又被隐藏节流改写”的情况。

## 当前确认的额外限制

当前 `third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/NadePacingPolicy.cs` 及其接入的 `NadeSystem.cs` 包含：

- 单个 BOT 非紧急投掷至少间隔 5 秒；
- 同队非紧急投掷至少间隔 0.5 秒；
- 同一时刻每个 BOT/队伍只允许一个非紧急预留；
- 开局 15 秒内单 BOT 最多一次；
- 开局按队伍限制总投掷、烟雾和进攻道具数量；
- `less` 每个 BOT 每回合所有类型合计最多 4 枚；
- 每个 BOT 每回合闪光最多 2 枚、烟雾/高爆/燃烧弹各最多 1 枚；
- `normal` 计划烟雾按队伍额外限制为每回合 1 枚；
- 失败预留、投掷冷却、反击冷却、开局预算等状态共同影响后续候选。

其中部分逻辑对 `max` 也生效，不能只修改 `more` 或 `less` 的分支来解决问题。

## 设计原则

1. 以已固定的上游 CS2-Bot-Improver v1.4.4 NadeSystem 为唯一行为基线。
2. 不保留 `NadePacingPolicy` 的运行时调用、状态字段、计数器或冷却器。
3. 不把“取消限制”实现为无限制生成；仍保留上游已有的经济、弹药、有效候选、地图投掷点、回合状态和安全检查。
4. 不修改 `bot_nades` 五个值的名称和命令契约：`off`、`less`、`normal`、`more`、`max`。
5. 不修改 CS2 原生 `bot_allow_grenades`、`sv_bot_buy_grenade_chance` 等配置来伪造效果。NadeSystem 的购买和生成机制仍按上游实现工作。
6. 0.5.12 已经是公开版本，不能覆盖其安装器、更新签名、R2 对象或 GitHub Release；本次应先形成工作区候选和独立验收证据，再决定后续修复版本发布。

## 实施范围

### 1. 建立上游基线

使用仓库中已经保存的上游 v1.4.4 源码目录：

```text
third_party/CS2-Bot-Improver-v1.4.4/addons/counterstrikesharp/plugins/NadeSystem
```

与当前改造目录逐文件比较：

```text
third_party/CS2-Bot-Improver-v1.4.4/nades-pacing
```

记录：

- 上游 revision/commit；
- NadeSystem 源文件清单；
- `NadeSystem.csproj` 与依赖版本；
- DLL、deps、pdb、grenades JSON 的 SHA-256；
- 当前改造目录与上游目录的差异文件。

### 2. 移除自定义策略接入

在 NadeSystem 源码中删除或恢复以下内容：

- `NadePacingPolicy` 类型及所有实例字段；
- `TryBegin`、`Commit`、`Cancel`、`Reset` 调用链；
- `_pacingPolicy`、预留集合、待提交集合、按 BOT/队伍计数器；
- 5 秒 BOT 间隔、0.5 秒队伍间隔；
- 开局 15 秒预算、开局烟雾/进攻上限；
- `less` 四枚总量限制；
- `normal` 计划烟雾一队一回合限制；
- 自定义失败概率冷却、反击冷却和投掷后自定义登记逻辑；
- 仅为策略审计新增的 `[NadeAudit]` 或等价输出。

如果上游 1.4.4 同名文件已经存在，应直接采用上游版本，而不是在当前改造文件上逐处打补丁。这样可以避免遗漏隐藏在生命周期、特殊投掷和回放路径中的限制。

### 3. 恢复上游投掷路径

以下路径使用上游实现：

- 普通计划投掷；
- 开局计划投掷；
- 烟雾、闪光、高爆、燃烧弹和诱饵；
- 安包掩护、拆包掩护、灭火和反击投掷；
- 投掷候选排序、方向/视线判断和地图标签判断；
- 投掷物创建成功/失败后的资金与状态处理；
- radio、sound、chat 投掷通知；
- 回合开始、回合结束、换图和插件卸载时的状态清理。

“取消自定义节奏限制”不等于删除上述功能。上游行为必须完整保留。

### 4. 资源包替换

重新构建上游 NadeSystem，并将以下文件整体替换为候选产物：

```text
addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll
addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.deps.json
addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.pdb
addons/counterstrikesharp/plugins/NadeSystem/grenades/*.json
```

只允许保留产品集成所需的包 marker、安装事务、完整性校验和资源清单改动；不得重新把节奏策略接回 DLL。

候选 ZIP 需要重新生成 manifest 与 SHA-256。任何资源摘要变化都必须在候选记录中明确列出，不能沿用旧 DLL 的摘要。

### 5. UI 与文案

UI 保留五档选择，不增加新的“无限制”档位。建议将说明调整为：

- `max`：上游最大投掷模式，不额外施加 CS2AS05 节奏限制；
- `more`：上游较积极投掷模式；
- `normal`：上游标准投掷模式；
- `less`：上游较少投掷模式；
- `off`：关闭 NadeSystem 投掷。

说明中不要承诺固定的每回合数量，因为实际结果仍受地图投掷点、BOT 经济、存活数量、视线、回合状态和原生游戏条件影响。

## 测试设计

### 静态测试

- 编译输入中不存在 `NadePacingPolicy.cs`；
- 候选 DLL 不依赖策略程序集；
- ZIP 中不存在策略源码、策略测试产物或旧 pacing DLL；
- NadeSystem 文件清单与上游 v1.4.4 基线一致；
- 五个 `bot_nades` 值都能注册、写入 cfg、读取和回显；
- 资源 manifest、DLL、deps、pdb 与 grenades JSON 摘要全部一致。

### 策略回归测试

删除或改写当前只验证自定义节奏上限的测试，例如：

- `BotAndTeamGaps`；
- `OpeningTeamAndTypeBudgets`；
- `NormalPlannedSmokeCap`；
- `LessTotalCapAndReset`；
- `EmergencyStillUsesHardLimit`；
- `ModeSpecificSmokeCapAndEmergencies`。

替换为上游行为契约测试：

- `more` 不受 CS2AS05 自定义 BOT/队伍间隔影响；
- `less` 不再被固定四枚总量截断；
- `normal` 不再有 CS2AS05 额外一队一烟限制；
- `max`、`more`、`normal`、`less` 只保留上游模式差异；
- 失败创建不会留下上游之外的额外预留状态；
- 特殊投掷、广播和回合清理仍保持上游行为。

### 真实 CS2 验收

由用户在相同地图、BOT 数量和回合条件下进行对照：

1. 使用候选包启动 `bot_nades max`，记录完整回合投掷事件。
2. 使用相同条件启动 `bot_nades more`，确认不再出现 5 秒 BOT 间隔、0.5 秒队伍间隔或策略预留阻塞。
3. 使用 `bot_nades normal`，确认仍有标准模式的上游决策，但没有新增的一队一烟限制。
4. 使用 `bot_nades less`，确认投掷减少来自上游 `less` 语义，而不是固定四枚硬截断。
5. 使用 `bot_nades off`，确认计划投掷关闭，必要的游戏原生流程不被破坏。
6. 检查控制台日志中没有旧策略专属审计行，并保留上游 NadeSystem 加载和投掷记录。

每档至少记录：地图、BOT 数量、回合长度、投掷类型与数量、失败原因、插件日志片段、候选 DLL SHA-256。

## 验收门槛

只有以下条件全部满足，才能称为“取消道具节奏限制”完成：

- 候选 DLL 的行为源自上游 v1.4.4，不是调大自定义策略常量；
- `NadePacingPolicy` 不再进入编译和发布包；
- `more` 到 `less` 不再受到项目自定义节奏/数量上限影响；
- `max` 仍保持高投掷表现，且不被隐藏硬限制截断；
- `normal`、`more`、`less`、`off` 的语义没有被修复 `max` 的过程破坏；
- 上游投掷通知、特殊投掷、经济检查、地图数据和生命周期完整保留；
- 自动化构建、资源摘要和候选 ZIP 校验通过；
- 用户完成真实 CS2 对照验收后，才讨论修复版本打包与发布。

## 回滚与停止条件

候选验证失败时：

- 保留旧 0.5.12 安装器和公开资源不动；
- 不覆盖玩家 CS2 目录；
- 不发布新的安装器、R2 对象、GitHub Release 或更新 feed；
- 将候选 ZIP、DLL 和日志移入独立 artifacts 目录；
- 回退仅针对未发布候选工作区，不使用破坏性 Git 操作。

如果上游 v1.4.4 源树、编译工具链或真实 CS2 日志无法证明一致，应停止在“候选未通过”状态，不把“编译成功”写成玩家问题已解决。

## 结果口径

本方案完成后，产品应描述为：

> `bot_nades` 由上游 NadeSystem 1.4.4 的五档语义控制。CS2AS05 不再额外限制 BOT 的投掷间隔、开局预算、每回合数量或队伍烟雾额度；实际投掷仍受地图数据、BOT 经济、存活状态、视线和 CS2 原生条件影响。
