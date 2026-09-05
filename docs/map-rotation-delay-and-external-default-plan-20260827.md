# MapRotation 自动换图延迟与外部默认开关交接方案

日期：2026-08-27  
交接对象：下一个【实际执行 AI】  
范围：只处理 MapRotation 自动换图的赛后延迟和“下一次插件启动默认是否开启”的外部配置；不改地图顺序、比分判定、`lbtv_map_next` 手动换图或游戏内运行时开关语义。

## 1. 用户目标与完成标准

玩家反馈最终结算画面尚未看完就换图。自动换图必须在当前最终结算判定后再等待 **15 秒** 才执行；这是现有 5 秒（延迟评估分支）或 3 秒（`EventGameEnd` 兜底分支）各自增加 10 秒后的结果。自动换图默认仍为开启。

助手未运行 CS2 时，玩家/助手可通过编辑外部配置文件改变“插件下一次载入时”的默认开关。CS2 运行期间，`lbtv_map_rotation 0|1` 仍是即时控制，且其行为、回复和权限边界不变。外部文件不得在运行中热覆盖 `_enabled`，也不得把运行时指令状态写回文件。

完成必须同时满足：

1. 代码、配置路径和安装包中的默认值一致且可审计。
2. 15 秒只作用于自动调度；`lbtv_map_next` 仍立即执行；重复事件仍由 `_changeScheduled`/serial 防重。
3. 配置缺失、JSON 损坏、字段缺失或非法类型时安全回退为 `enabled=true`，记录明确日志，不阻止插件加载。
4. 已存在的玩家配置在插件包重装/升级时保留，不被默认文件覆盖。
5. 有静态契约测试和至少一次构建/包内容校验；真实 CS2 赛后画面和玩家实机最终确认由用户完成，不能用静态测试冒充。

## 2. 已调查的当前事实（执行基线）

- 插件源码：`third_party/CS2-Bot-Improver-map-rotation/addons/counterstrikesharp/plugins/MapRotation/MapRotation.cs`。
- `Load(bool hotReload)` 当前硬编码 `_enabled = true`，并注册 `OnMapStart`、`EventRoundEnd`、`EventGameEnd`。
- 最终回合检测 `EvaluateRoundEndAfterScoreUpdate` 当前调用 `ScheduleNextMap(..., delaySeconds: 5.0f)`。
- `OnGameEnd` 兜底当前调用 `ScheduleNextMap(..., delaySeconds: 3.0f)`。
- `lbtv_map_rotation` 当前无参数显示状态，有 `0/1` 参数即时切换 `_enabled`；`lbtv_map_next` 直接执行 `changelevel`。
- 资源包：`src-tauri/resources/CS2BotImprover.zip` 已含 `addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll`，同时已有 `addons/counterstrikesharp/configs/plugins/README.txt`，但无 MapRotation JSON。
- 安装/升级实现主要在 `src-tauri/src/services/cs2.rs`；资源包应通过现有事务安装，不新增绕过事务的写入。
- 当前工作树有大量既有改动；执行 AI 必须保留无关改动，不 reset/checkout/clean。
- `package.json` 当前版本为 `0.5.9`；本任务不要求升版本或发布。

## 3. 推荐配置契约

新增文件（随资源包提供）：

```text
addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json
```

默认内容：

```json
{
  "enabled": true
}
```

契约要求：

- 顶层必须是 JSON object；只读取布尔字段 `enabled`。
- `true` 表示插件每次 `Load` 时 `_enabled=true`；`false` 表示 `_enabled=false`。
- 文件不存在、无法读取、JSON 解析失败、顶层非 object、`enabled` 缺失/非 bool：记录 `[MapRotation]` 警告并回退 `true`。
- 可选未知字段忽略，便于未来扩展；不要把延迟做成此次用户可配置项，延迟固定 15 秒。
- 配置是“默认值”而非持久运行状态：`lbtv_map_rotation 0|1` 只改变内存 `_enabled`，不修改 JSON。
- 读取应使用 CounterStrikeSharp/游戏服务器可访问的配置根。优先采用项目已在插件中可稳定获得的 `Server.GameDirectory`（或同等官方 API）拼接 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`；如果 API 实际不存在，执行 AI 必须先以当前 CounterStrikeSharp 依赖编译验证，再选择该版本的官方路径 API，禁止猜测用户目录。

## 4. 实现步骤（严格顺序）

### 4.1 插件 C#

1. 在 `MapRotation.cs` 增加常量：配置相对路径、默认值 `true`、统一自动延迟 `15.0f`。
2. 增加最小 DTO，例如 `MapRotationConfig`，只含 `bool Enabled { get; set; } = true;`。使用项目目标框架可用的 `System.Text.Json`；反序列化失败捕获异常并回退。
3. 增加 `LoadDefaultEnabled()`/等价私有函数：读取文件、校验 object/bool、打印来源和最终值；读取失败打印原因但不能抛出到插件加载边界。
4. `Load` 中将 `_enabled = LoadDefaultEnabled();`，保留现有状态初始化、监听注册和 `lbtv_map_rotation` 命令文本。日志应包含 `Default enabled=0|1, config=...`，方便用户核对。
5. 两个自动分支都改用同一常量 15 秒：
   - `EvaluateRoundEndAfterScoreUpdate`：`ScheduleNextMap(..., delaySeconds: AutoChangeDelaySeconds)`。
   - `OnGameEnd` fallback：同上。
   日志中的 `delay=15.0s` 必须可见。
6. 不修改 `OnMapNextCommand` 的立即 `ExecuteChangeLevel`；不修改 `_changeScheduled`、`_scheduleSerial`、最终比分判定和 tie boundary 逻辑。
7. 不在 `OnMapRotationCommand` 写配置文件。运行时指令仍允许显示状态、传入仅 `0`/`1`，非法值仍回复 Usage。

### 4.2 默认配置与资源包

1. 在第三方源码树添加上述 JSON（建议同时在 `third_party/CS2-Bot-Improver-map-rotation/README.md` 说明路径、字段和回退行为）。
2. 更新 `BUILD.md`：先 `dotnet build ... -c Release --nologo`，再将 DLL 与 JSON 精确放入临时 ZIP；不要直接覆盖原 ZIP 后才验证。
3. 使用现有资源包合并/事务脚本模式，将：
   - `addons/counterstrikesharp/plugins/MapRotation/MapRotation.dll`
   - `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`
   合入 `src-tauri/resources/CS2BotImprover.zip`，保留其他 entry、目录和 README。
4. 安装语义必须是“默认文件仅在目标不存在时写入”。检查 `src-tauri/src/services/cs2.rs` 的事务覆盖/保留规则；若当前资源包安装会无条件覆盖该 JSON，应为 MapRotation 配置增加与现有用户偏好同等的保留规则，并写回归测试。不得影响其他插件配置。
5. 重新生成项目已有的插件 marker/manifest（如 `CS2AS05.plugin.json` 及其 payload entries/hash），但只在实际脚本要求时更新；不要手工伪造哈希。

## 5. 测试与证据门禁

### 5.1 静态/单元契约（建议扩展 `tests/map-rotation-contract.spec.ts`）

- 源码包含配置相对路径、`System.Text.Json`（或已验证的同等解析器）、默认 true、15 秒统一常量/两处分支引用。
- 源码仍包含 `lbtv_map_rotation [0|1]`、`ScheduleNextMap`、`lbtv_map_next` 立即路径。
- 资源 ZIP 包含 DLL 和 `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`，JSON 为 `{"enabled":true}`。
- 测试配置文件不存在/损坏/false/true 的解析回退逻辑（若 C# 单测难接入，至少把纯解析函数抽出并用可运行的 Rust/C# 测试覆盖）。

### 5.2 构建与包校验

在仓库根目录执行并记录输出：

```powershell
dotnet build .\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo
npm test -- --run tests/map-rotation-contract.spec.ts
cargo test --manifest-path .\src-tauri\Cargo.toml
```

使用 `System.IO.Compression.ZipFile` 回读 ZIP entry、JSON 内容、DLL SHA-256；再运行 `npm run typecheck`、`npm run lint` 或项目既定最小验证。若完整套件因既有脏改动失败，报告准确失败项，不删除或回退他人改动。

### 5.3 运行时验收（用户执行）

在 CS2 未运行时编辑目标游戏目录的 JSON：

```json
{"enabled":false}
```

启动服务器/插件后执行 `lbtv_map_rotation`，应返回 `enabled=0`；再执行 `lbtv_map_rotation 1`，应立即返回 `enabled=1`。停止 CS2 后改为 `{"enabled":true}`，下次载入应默认开启。运行中直接编辑 JSON 不应改变当前状态，除非插件按既有生命周期重新载入。

完成一局达到最终比分的本地对局：从“target score reached”或 `EventGameEnd fallback` 日志时间戳起，确认约 15 秒后才出现 `Executing command: changelevel ...`，期间能看到结算画面；手动 `lbtv_map_next` 仍立即换图。真实游戏/画面证据由用户提供，执行 AI 不能自行宣称已完成。

## 6. 失败处理与停止条件

- 找不到稳定的 CounterStrikeSharp 配置目录 API：停止修改，记录编译错误和候选 API，不把猜测路径写进发布包。
- 安装事务无法保留玩家 JSON：停止合包，先修复并测试保留语义；不得发布会覆盖用户开关的包。
- ZIP manifest/hash 任一不一致：保留原 ZIP，删除临时产物，报告差异；不得声称资源已更新。
- 不要构建安装器、上传 GitHub/R2、部署网站或改版本号，除非用户另行授权。

## 7. 交付清单

执行 AI 最终应回传：修改文件绝对路径、配置实际路径与示例、自动延迟证据（两分支均为 15 秒）、运行时指令未改变的证据、测试命令与结果、ZIP entry/DLL/manifest 哈希、未完成的真实 CS2 验收项，以及是否产生任何未提交构建产物。

