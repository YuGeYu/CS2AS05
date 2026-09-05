# MapRotation 插件 enabled 配置未生效修复交接方案

日期：2026-08-31  
工作区：`E:\CS2AS05`  
交接对象：实际执行 AI  
状态：调查完成，待插件侧执行；本轮不修改助手设置逻辑，不代表修复已完成。

## 1. 用户现场与完成标准

用户在助手中关闭自动换图，游戏仍返回并实际自动换图：

```text
[Client] [MapRotation] enabled=1, current=de_vertigo, index=8, next=de_cache
```

用户已进一步验证：退出 CS2，将外部配置写为 `enabled=false`，再次启动 BOT 后仍返回 `enabled=1`，且确实发生了自动换图。因此本轮根因边界锁定为 **MapRotation 插件读取/解析/初始化问题**，不是助手的写入按钮、Tauri IPC 或外部配置服务。

完成标准：

- 插件下一次 `Load()` 必须读取助手写入的同一个 `MapRotation.json`；JSON 中 `enabled=false` 时 `_enabled` 初始值必须为 false；`true` 时为 true；
- `lbtv_map_rotation` 无参数返回的 enabled 必须等于本次 `Load()` 解析到的内存值；
- `enabled=false` 时，回合结束和 `EventGameEnd` 均不得调度自动 `changelevel`；`lbtv_map_next` 仍可作为显式立即换图命令；
- `lbtv_map_rotation 0|1` 仍只改变当前进程内存，不回写 JSON；
- 配置缺失、损坏或类型错误时只能使用明确记录的默认值（当前默认 true），不得把解析失败伪装成成功读取 false；
- 真实 CS2 重启后，false/true 两条流程都必须闭环验证，不能只凭源码或助手 UI 通过。

## 2. 已调查事实与插件侧高概率根因

### 2.1 运行链路

- 插件源码：`third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs`。
- `Load()` 执行 `_enabled = LoadDefaultEnabled();`，之后 `lbtv_map_rotation` 读取 `_enabled`，自动换图由 `ScheduleNextMap()` 检查 `_enabled`。
- `LoadDefaultEnabled()` 使用 `Server.GameDirectory + addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`，路径与助手服务约定的相对路径一致。
- 读取 JSON 后调用 `JsonSerializer.Deserialize<MapRotationConfig>(stream)`；配置类只有 C# 属性 `public bool Enabled { get; set; } = DefaultEnabled;`。
- 助手生成/写入的 JSON 使用小写键 `{"enabled":false}`。`System.Text.Json` 默认属性名匹配区分大小写；若未设置 `PropertyNameCaseInsensitive=true` 或 `[JsonPropertyName("enabled")]`，小写 `enabled` 不会绑定到 `Enabled`，属性保持初始值 true。当前代码随后只判断 `config is null`，不会发现“对象有效但 Enabled 没有被绑定”，于是记录读取成功却返回 true。
- 该行为与用户现象完全吻合：无论外部文件写 false，插件都以属性默认 true 启动；因此状态命令为 enabled=1，回合结束实际自动换图。

上述为源码推导的最可能根因；实际执行必须用受控 JSON fixture 和真实插件日志验证，不能只凭推断改代码。

### 2.2 不能误改的边界

- 不修改 `src-tauri/src/services/map_rotation.rs` 的写入语义，除非测试证明它写出的 JSON 不是 `enabled` 布尔值；助手侧已有写后 read-back/hash 机制。
- 不把外部 JSON 变成运行时热更新；插件仍在 `Load()` 读取一次，运行中命令控制内存。
- 不删除 payload/marker 校验，不把 MapRotation JSON 重新放回固定 payload；它继续属于 `mutableConfigEntries`。
- 不改变 `lbtv_map_next` 立即换图能力、15 秒自动换图延迟、地图列表或回合结束判定。

## 3. 实际执行方案

### P0：修正 JSON 绑定并加强失败可见性

1. 修改 `MapRotation.cs` 的 `MapRotationConfig`：优先使用
   `[JsonPropertyName("enabled")] public bool Enabled { get; set; }`
   并保留显式 `PropertyNameCaseInsensitive=true`（二者择其一也可，但建议两层防御只保留一种清晰方案，避免重复语义）。
2. 反序列化后增加严格 schema 校验：JSON 须为 object，`enabled` 必须存在且为 JSON boolean；缺失、字符串、数字、null、数组均记录 `[MAP_ROTATION_CONFIG_INVALID]` 并走默认值，同时在日志中标明 reason。
3. `LoadDefaultEnabled()` 日志必须打印 canonical 绝对路径、`Server.GameDirectory`、文件 SHA-256、解析结果 `enabled=0|1`、source=`config|fallback`。禁止只打印相对路径和“Loaded”而无法判断是否真的绑定成功。
4. `Load()` 保持 `_enabled = LoadDefaultEnabled()` 的单次初始化；`OnMapStart` 不得重置 `_enabled=true`，`ScheduleNextMap` 和 `OnGameEnd` 必须继续尊重 `_enabled`。
5. 若发现已安装目录加载的是旧 DLL，修正构建/安装流程，使当前 `MapRotation.dll` 与源码/marker 版本一致；不能通过助手 UI 假装插件已更新。

### P0：插件侧单元/受控 fixture

建议将 JSON 解析抽为无 CS2 依赖的纯函数，例如 `ParseConfig(ReadOnlySpan<byte>, path)`，然后增加 C# 测试或最小受控 harness：

- `{"enabled":false}` -> `Enabled=false`、source=config；
- `{"enabled":true}` -> true；
- `{}`、`{"Enabled":false}`（若产品只接受小写，应明确拒绝或兼容并测试）、`{"enabled":"false"}`、`{"enabled":0}`、`[]`、损坏 JSON -> fallback 且有稳定 reason；
- 同一输入多次解析结果一致；读取只读，不写回 JSON；
- `lbtv_map_rotation 0|1` 的命令测试确认只修改内存状态。

若仓库当前没有 C# 测试框架，可使用 `dotnet build` 加独立 console fixture；fixture 必须只测试解析函数，不伪造 CS2 自动换图成功。

### P1：资源与安装回归

1. 运行 `scripts/generate-plugin-manifest.ps1`，确认 MapRotation JSON 仍只在 `mutableConfigEntries`；固定 payload digest 不因用户 enabled 值变化。
2. 审查 `src-tauri/src/services/cs2.rs`、`install_game_files_transactionally()` 和插件复制目录：升级/安装不得用 ZIP 默认配置覆盖用户 false；若发生迁移，先保存用户 JSON，再覆盖固定 DLL，最后恢复并回读 JSON。
3. 记录当前源码 ZIP、构建嵌入资源、安装后 DLL/marker 的版本和 SHA-256；旧候选 hash 只能作为历史证据，不得混入新验收。

## 4. 自动化验证命令

```powershell
Set-Location E:\CS2AS05
dotnet build .\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo
npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts tests/bot-plugin-gate.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 map_rotation panel
powershell -ExecutionPolicy Bypass -File .\scripts\generate-plugin-manifest.ps1
```

应新增/更新 `tests/map-rotation-plugin-config.spec.ts`，至少静态断言：

- 插件使用显式小写 `enabled` 绑定或大小写不敏感配置；
- 解析失败不会静默回到 true 并记录 invalid reason；
- `Load()` 初始化 `_enabled` 来自 `LoadDefaultEnabled()`；
- `ScheduleNextMap`/`OnGameEnd` 均保留 disabled gate；
- `lbtv_map_next` 不受 `_enabled=false` 阻断；
- 不存在通过助手 UI 改代码或绕过插件读取的假修复。

## 5. 真实 CS2 验收

使用用户实际目录或隔离但结构一致的目录，保存所有日志到：

```text
E:\CS2AS05\artifacts\map-rotation-plugin-config-fix-20260831\
```

1. 完全退出 CS2，确认没有旧 `cs2.exe`/BOT 进程；记录实际加载插件 DLL 的绝对路径和 SHA-256。
2. 写入 `MapRotation.json` 为 `{"enabled":false}`，记录文件绝对路径、bytes、SHA-256。
3. 从当前构建启动 BOT/CS2；保存插件 `Load()` 日志，必须同时出现 canonical path、文件 SHA 和 `enabled=0/source=config`。
4. 执行无参数 `lbtv_map_rotation`，必须返回 `enabled=0`；完成一局/触发 `round_end` 与 `EventGameEnd`，确认日志没有 `Scheduling next map`，且 15 秒后没有自动 `changelevel`。
5. 在同一运行中执行 `lbtv_map_next`，确认显式命令仍立即切换下一张图；这不证明自动轮换开启，只证明命令边界未被误改。
6. 退出 CS2，改为 `{"enabled":true}`，重新启动并执行无参数命令，必须返回 `enabled=1`；达到终局后确认出现一次 15 秒 `Scheduling next map`。
7. 运行中执行 `lbtv_map_rotation 0`，确认后续回合不再调度；退出后检查 JSON 仍为 true，证明命令未回写文件。再执行 `lbtv_map_rotation 1`，确认恢复运行时自动轮换。
8. 若 false 仍返回 1，优先比较三项：实际 DLL 路径/SHA、插件日志 path/SHA、助手写入 path/SHA；不得继续修改助手 UI 或猜测多个 Steam library。

## 6. 停止条件与交付报告

任一条件成立即停止并报告“插件修复候选/待真实验收”：

- `{"enabled":false}` 仍被解析为 true，或日志只显示“config valid”而无实际绑定值；
- 插件 Load 路径、助手写入路径或 SHA 不一致；
- 运行中命令回写 JSON、false 状态仍自动换图、true 状态不再轮换；
- 安装/升级覆盖用户配置，或 MapRotation JSON 重新进入固定 payload；
- 只有 C# 编译/静态测试，没有真实 `lbtv_map_rotation` 回读和自动换图日志。

执行 AI 最终需新增执行报告，写明：实际修改文件、JSON 解析 fixture 结果、插件 DLL/ZIP/marker hash、助手与插件路径/SHA 对账、false/true 两次 `lbtv_map_rotation` 回读、自动换图/`lbtv_map_next` 日志、命令不回写证据、测试结果、截图路径和未完成的用户实机验收项。证据闭环前不得宣称助手或插件问题已彻底解决。
