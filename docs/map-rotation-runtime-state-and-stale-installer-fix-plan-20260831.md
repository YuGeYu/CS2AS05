# MapRotation 默认值未生效与旧安装器摘要问题修复交接方案

日期：2026-08-31  
工作区：`E:\CS2AS05`  
角色：方案制定 AI 交给实际执行 AI  
状态：调查与方案；本轮未修改业务代码、未构建安装器

## 1. 用户现场与完成标准

用户在助手中关闭自动换图后，游戏内执行：

```text
lbtv_map_rotation
[Client] [MapRotation] enabled=1, current=de_vertigo, index=8, next=de_cache
```

这表示用户期望的默认值与游戏当前 MapRotation 内存状态不一致，必须修复并解释清楚：

- 助手写入的外部 JSON 必须是插件下一次 `Load()` 实际读取的同一个文件；
- CS2/插件运行期间修改 JSON 不得假装立即改变内存 `_enabled`，但界面必须明确显示“下一次载入生效”；
- 用户退出 CS2、设置 `enabled=false`、再次启动 BOT 后，游戏内首次 `lbtv_map_rotation` 必须返回 `enabled=0`；
- 设置 `enabled=true` 的同一流程必须返回 `enabled=1`；
- `lbtv_map_rotation 0|1` 仍只改变运行时内存，不回写 JSON；
- BOT 启动不得因用户的 true/false 配置触发 payload 摘要错误。

同时修复上一现场的旧资源问题：用户曾收到 `zipSha256=FC8868...`、期望 payload `76D9...`，而当前工作树 ZIP 已回读为 `8C9D...`、marker payload 为 `6880...`。最终安装器必须证明使用的是当前修复后的资源，而不是旧候选。

## 2. 已调查的当前代码事实

### 2.1 助手写入路径

- `src-tauri/src/services/map_rotation.rs` 固定写入：
  `game/csgo/addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`。
- 路径由 `cs2::normalize_root()` 加 `<root>\game\csgo` 构造，写入后通过 Rust `read_at()` 回读；默认/现有/fallback 状态由 `source` 区分。
- `set()` 在 `cs2::check_cs2_process()` 为 true 时拒绝写入 `[MAP_ROTATION_CS2_RUNNING]`；因此运行中不能修改文件。
- `src/components/MapRotationDefaultControl.vue` 位于 `OverviewView.vue` 唯一入口，当前已显示“只影响下一次 MapRotation 载入”。

### 2.2 插件读取路径与时机

- `third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs` 的 `Load()` 使用 `Server.GameDirectory` 拼接同一相对路径。
- `Load()` 将 JSON 的 `enabled` 复制到进程内 `_enabled`；`lbtv_map_rotation` 查询的是 `_enabled`，不是每次重新读取 JSON。
- 因此“助手写入 false 后游戏仍返回 enabled=1”有两种必须区分的情况：
  1. CS2/插件在写入前已经载入，属于预期的“下一次载入生效”时序；
  2. 退出后重新启动仍返回 1，才是路径、资源覆盖、启动前写入或插件加载实例不一致的真实缺陷。
- 当前方案不能只把按钮文案改成“已关闭”，必须通过新一轮 CS2 退出/启动和日志证明。

### 2.3 资源摘要现状

当前工作树已具备上一阶段部分改造：

- `scripts/generate-plugin-manifest.ps1` 已将 MapRotation JSON 放入 `mutableConfigEntries`，不再放入 `payloadEntries`；
- `src-tauri/src/services/cs2.rs` 的 `PluginMarker` 已有 `mutable_config_entries`，并拒绝将 MapRotation JSON 继续列为固定 payload；
- 当前源码 ZIP marker version 为 0.5.10，payload hash 为 `6880C768...`；
- 但用户现场错误仍使用旧 ZIP hash `FC8868...` 和旧 payload `76D9...`。这证明实际运行安装器/EXE 不是当前资源，或存在未重建的旧安装目录。

## 3. P0：修复默认值与运行时状态验证链路

### 3.1 加入可审计的配置写入回读

修改 `src-tauri/src/services/map_rotation.rs` 和 DTO：

- `MapRotationDefault` 增加 `configSha256`、`readBackEnabled`、`observedAt`、`loadSemantics: "next-plugin-load"`；
- `set/reset` 写入后必须重新读取 JSON、计算 SHA-256，并返回实际 `enabled` 和绝对路径；
- runtime.log 写入 `[MAP_ROTATION_DEFAULT_UPDATED] enabled=0|1 path=... sha256=... observedAt=...`；
- 写入前、写入后都记录 CS2 结构化进程快照，证明写入期间没有 CS2 进程。

前端 `MapRotationDefaultControl.vue` 只使用 Rust 回读结果更新 UI；显示“已写入，下一次插件载入生效”，同时显示配置路径和最近回读值。不能在点击瞬间乐观地把游戏当前状态称为 disabled。

### 3.2 统一插件路径并增加启动前对账

在插件 `MapRotation.cs` 和 Rust service 各自增加规范化路径日志：

- 插件 Load 日志输出解析后的绝对配置路径、`enabled`、文件 SHA-256、`Server.GameDirectory`；
- 助手启动 BOT 前日志输出相同根目录、配置绝对路径、配置 SHA-256 和预期 enabled；
- 真实验收时必须证明两条路径字符串 canonical 后相同，不能只看相对路径文本。

如发现 Steam 启动使用的 CS2 根目录与助手选择目录不同，停止并修正目录绑定；不得在多个 Steam library 之间猜测或写入所有找到的目录。

### 3.3 明确并实现“下一次载入”语义

- CS2 运行中：助手开关禁用并返回 `[MAP_ROTATION_CS2_RUNNING]`，不写文件；游戏内命令继续即时切换 `_enabled`。
- CS2 退出后：助手写 JSON true/false；写入成功只表示“下一次插件载入默认值已更新”。
- 启动 BOT 时：确保插件目录加载的是当前安装包 DLL；启动后读取 MapRotation Load 日志，再执行 `lbtv_map_rotation` 验证。
- 如产品确实需要“当前游戏立即关闭”，必须新增明确的游戏内命令桥接或重启插件方案；本轮不把外部 JSON 写入伪装成热更新。

### 3.4 防止助手启动流程覆盖配置

检查 `src-tauri/src/services/cs2.rs::install_game_files_transactionally()` 和 `src-tauri/src/services/panel.rs::launch_cs2_inner()`：

- 安装/自动更新固定资源时保留目标 MapRotation JSON；
- 启动 BOT 前不能无条件从 ZIP 复制默认 `MapRotation.json` 覆盖用户 false；
- 启动流程只允许切换 `gameinfo.gi`、安装固定 payload 和读取配置，不应重置 MapRotation JSON；
- 增加启动前后 JSON bytes/SHA-256 对账测试，确认 false 不会被自动安装恢复为 true。

## 4. P0：彻底解决旧安装器/marker 摘要漂移

### 4.1 资源生成一致性

执行 AI 必须重新运行：

```powershell
Set-Location E:\CS2AS05
powershell -ExecutionPolicy Bypass -File .\scripts\generate-plugin-manifest.ps1
```

然后回读 `workspace/runtime/plugin-manifest/result.json`，确认：

- version=0.5.10；
- MapRotation JSON 只在 `mutableConfigEntries`；
- payload hash 与 `src-tauri/resources/CS2BotImprover.zip` 回读一致；
- Rust `CUSTOM_ZIP_SHA256` 已同步当前 ZIP；
- marker 的 payload entry 集合与 Rust `payload_digest_from_files()` 完全一致。

### 4.2 四段 hash 对账

必须记录四个位置的 hash 和 marker：

1. 工作树 `src-tauri/resources/CS2BotImprover.zip`；
2. Tauri 编译/打包阶段实际嵌入的 ZIP；
3. NSIS 安装器安装后的资源/程序；
4. 用户选定 CS2 目录中的 `CS2AS05.plugin.json` 与固定 payload digest。

旧的 `FC8868...`、`76D9...` 只作为历史错误证据，不能继续出现在新候选任何一段。若安装器无法回读内置 ZIP，则至少通过构建日志、资源清单和安装后 marker/固定 payload 对账证明来源；不能仅凭安装器文件名。

### 4.3 固定 payload 与可变配置契约

- MapRotation JSON 不参与 `payloadSha256`；true/false、空格和换行均不影响固定摘要；
- 固定 DLL、PDB、BotVision、NadeSystem 数据继续严格校验；
- `enabled=false` 是合法配置，不能返回 `[BOT_PLUGIN_PAYLOAD_INVALID]`；
- marker version 必须与程序精确为 0.5.10；
- 旧 marker 迁移时保存用户 JSON 值，替换固定 marker/payload 后恢复该值并回读。

## 5. 测试要求

### 自动化

新增/更新：

- `tests/map-rotation-settings.spec.ts`：true/false 回读、路径显示、运行中禁用、`loadSemantics` 文案、错误状态；
- Rust map_rotation 测试：写入原子性、SHA 回读、路径 canonical、CS2 running 拒绝、缺失/损坏/fallback；
- Rust cs2 测试：MapRotation JSON 不在 payload、mutable entry 存在、true/false 均通过固定 payload 校验、旧 marker 迁移；
- 安装事务测试：已有 false 在 BOT 自动安装和普通升级前后保持 false；
- 插件 C# 测试或受控 fixture：Load 读取 false/true，运行命令只反映内存值，不写回 JSON。

建议命令：

```powershell
npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts tests/bot-plugin-gate.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml cs2 map_rotation panel
dotnet build .\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo
```

### 真实 Tauri/CS2 验收

在退出 CS2 的隔离目录中：

1. 启动助手，选择正确的 `D:\SteamLibrary\...\Counter-Strike Global Offensive` 根目录；确认助手显示的配置路径与插件日志将使用的路径相同。
2. 设置 false，回读助手返回 `enabled=false`、configSha256 和文件内容。
3. 从助手启动 BOT；确认实际安装器资源 hash 是新值，安装后 marker version=0.5.10，固定 payload digest 与 marker 相同。
4. 等插件 Load 完成后执行 `lbtv_map_rotation`，必须返回 `enabled=0`；保存插件日志中的绝对配置路径和读取值。
5. 退出 CS2，设置 true，再次启动 BOT，命令必须返回 `enabled=1`。
6. 运行中执行 `lbtv_map_rotation 1` 或 `0`，确认只改变内存；退出后检查 JSON 没有被命令改写。
7. 在助手中切换 false 后不重启 CS2，若游戏仍显示 enabled=1，记录为符合“下一次载入”语义；只有重启后仍为 1 才判定缺陷。
8. 记录 UI 截图：概览页设置框在 1280x800、980x640、375px 内，长路径不越界；安装与诊断页不得出现重复入口。

## 6. 无私钥安装器

只构建本地功能验证候选，不读取或注入任何私钥：

```powershell
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PATH -ErrorAction SilentlyContinue
npm run bundle:desktop
```

构建后必须回读安装器路径、大小、SHA-256、文件版本和 `Authenticode=NotSigned`。如 updater 公钥使命令最终返回非零，但 NSIS 已生成，只有在四段资源对账通过后才可交付，并标记 `unsigned/local validation only`；不得把旧候选安装器改名冒充新包。

## 7. 停止条件与交付报告

出现以下任一情况立即停止：

- 重启 CS2 后 false 仍为 1，且助手/插件绝对路径或 SHA 不一致；
- 新安装器仍嵌入 `FC8868...` 或 marker payload `76D9...`；
- MapRotation JSON 仍进入固定 payload；
- 为规避错误而删除 payload 校验、降低版本门禁或覆盖用户配置；
- 无法证明安装器使用当前源码 ZIP；
- 设置页在任一视口仍越界；
- 真实插件日志缺失，无法证明 Load 读取值。

执行 AI 必须更新执行报告，包含：实际修改文件、助手写入 JSON 的绝对路径/内容/hash、插件 Load 日志、true/false 两次命令回读、四段资源 hash、测试结果、未签名安装器绝对路径与 SHA-256、截图路径及剩余用户验收项。证据未闭环前，状态只能写“第二阶段修复候选/待真实 CS2 验收”。
