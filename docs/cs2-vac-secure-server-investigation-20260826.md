# CS2 在线模式 VAC 安全连接失败调查报告

日期：2026-08-26  
对象：最新版本 Counter-Strike 2、Steam 启动入口、CS2 人机增强助手在线/BOT 启动链  
截图：`C:\Users\GOPtZ\Pictures\Screenshots\屏幕截图 2026-08-26 192918.png`

## 1. 现象与结论

截图中的弹窗是 **Valve 反作弊（VAC）安全服务器连接失败**：系统检测到游戏文件“无签名或签名无效”，因此不能加入 VAC 安全服务器。它不是“CS2 进程无法启动”的错误。

用户补充的“可以进入大厅，开始在线游戏时才报错”与该结论一致：大厅启动阶段只证明 `cs2.exe` 能运行；匹配/加入安全服务器时，VAC 才对客户端文件和第三方模块状态执行安全校验。

**当前最高可信根因：助手的 BOT/插件环境仍存在于 CS2 安装目录或曾被加载，导致在线会话的客户端不满足 VAC secure 条件。** 直接触发点不是 Steam `-applaunch 730`，而是安装目录中的非官方/未签名文件、Metamod/CounterStrikeSharp 组件或运行时补丁状态。Steam 启动助手后仍复现，是因为关闭助手不会卸载已写入磁盘的插件文件，也不会把已修改的 CS2 安装恢复为官方完整状态。

## 2. 证据

### 2.1 截图证据

- 弹窗标题：`Valve 反作弊`。
- 原文含义：检测到部分游戏文件无签名或签名无效，无法加入 VAC 安全服务器，要求检查启动选项和游戏安装后重启。
- 该错误属于 VAC secure join gate；不是大厅加载、Steam 登录或 `gameinfo.gi` 解析错误。

### 2.2 代码启动链

`src-tauri/src/services/panel.rs` 的 `launch_cs2_inner`：

1. 检查 `cs2.exe` 未运行并解析 Steam。
2. BOT 模式自动校验/安装插件包，并切换 `gameinfo.gi`。
3. 在线模式复制 `backup/Online/gameinfo.gi`，然后执行 Steam `-applaunch 730`。
4. BOT 模式额外传入 `-insecure -console -condebug`；在线模式不传 `-insecure`。

因此，“在线按钮没有传 `-insecure`”并不等价于“客户端已恢复为官方 VAC 环境”。在线模式只切换了活动 `gameinfo.gi`，不会删除已安装的 Metamod、CounterStrikeSharp、DLL、VDF 或其他插件文件。

### 2.3 本机安装目录快照

调查时 CS2 根目录为：

`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive`

- `game\csgo\gameinfo.gi` SHA-256：`F25BAD2B636A3FEF1ED1CB961F022069C20FFBCCD49F06A01700E9DC2F593A63`
- `game\csgo\backup\Online\gameinfo.gi` 与活动文件同哈希，说明当前活动模式文件是 Online 版本。
- `game\csgo\backup\WithBots\gameinfo.gi` SHA-256：`5BCE022578E3BF4...`，明显不同并包含 `csgo/addons/metamod` SearchPath。
- 安装目录仍存在 `game\csgo\addons\metamod`、`game\csgo\addons\counterstrikesharp` 及大量插件日志/组件。
- 目录中存在大量 `gameinfo.gi.backup-*`、配置和 `botprofile.vpk.backup-*`，证明助手/插件环境曾被反复切换和写入。

“活动 gameinfo 当前是 Online”只能说明 SearchPath 没有显式加载 Metamod；不能证明所有第三方文件已移除，也不能证明本次进程没有加载其他注入/第三方模块。VAC 的文件签名校验范围也不等同于 `gameinfo.gi` 的 SearchPath。

### 2.4 CS2/插件运行日志

`game\bin\win64\counterstrikesharp.log` 在 2026-08-26 14:00:18 记录：

```text
cs2.exe -steam -insecure -console -condebug ... -allow_third_party_software ...
```

随后 CounterStrikeSharp 明确记录：

- 复制并读取 `engine2.dll`、`server.dll`。
- 加载 .NET runtime、CounterStrikeSharp API。
- 加载 BotAI、BotAimImprover、BotRandomizer、InventorySimulator、MapRotation、NadeSystem 等插件。
- BotAI 日志记录 `Applied 42/42 patches`。

这证明该安装曾以非 VAC 安全模式启动，并对游戏运行时应用了第三方插件/补丁。关闭助手或再次从 Steam 启动不会逆向清除这些磁盘组件。

`game\csgo\console.log` 在 19:28:55 的命令行显示为：

```text
-steam -nojoy -freq 165 -high -tickrate 128 -allow_third_party_software -worldwide -console -condebug
```

该记录没有 `-insecure`，但仍含 `-allow_third_party_software`，且它只反映本次启动参数，不构成对安装目录完整性或 VAC 可用性的证明。

### 2.5 Steam 用户配置

`C:\Program Files (x86)\Steamzhen\userdata\1860677758\config\localconfig.vdf` 的 App 730 `LaunchOptions` 为：

```text
-nojoy -freq 165 -high -tickrate 128 -allow_third_party_software -worldwide -console -condebug
```

这说明启动参数由 Steam 用户配置持久化保存。助手退出后，Steam 入口仍会继承这些参数；更重要的是，Steam 入口不会删除插件 DLL/Metamod 文件。

## 3. 排除项与边界

- **不是单纯 `gameinfo.gi` 导致的启动失败。** 当前活动文件与 Online 备份一致，且游戏能够进入大厅；问题发生在 VAC 安全服务器接入阶段。
- **不能把“大厅能进”解释为“在线环境正常”。** VAC 安全校验发生在后续匹配/连接阶段，阶段不同，结论不能混用。
- `console.log` 中的资源缺失提示（例如 econ 图标）不是本弹窗的决定性证据。
- 14:28 左右的 `BotBuyPatch` `Schema target points to null` 是插件回调异常，说明 BOT 插件与当前游戏状态存在兼容性问题，但它不是截图所示 VAC 文件签名错误的直接证明；两者都支持“在线客户端不应加载 BOT 插件环境”的安全边界。
- 目前没有用户提供的 Steam“验证游戏文件”结果、VAC 模块列表或干净目录复测，因此不能声称已完成实机修复。

## 4. 复现与权威验证步骤

以下步骤应在退出 CS2 后执行，并保留 Steam 登录状态：

1. 在助手中执行 BOT 插件卸载/恢复官方文件；确认 `game\\csgo\\addons\\metamod` 和 `game\\csgo\\addons\\counterstrikesharp` 不再作为在线客户端组件存在。不要手工删除未知 Steam 核心文件。
2. 在 Steam 对 CS2 执行“验证游戏文件完整性”，等待下载/替换完成。
3. 清空 App 730 的自定义 Launch Options，至少移除 `-allow_third_party_software`、`-insecure`、`-console`、`-condebug`；先使用 Steam 默认启动参数。
4. 完全退出 CS2，再退出并重新启动 Steam，避免旧进程/旧命令行残留。
5. 仅从 Steam 启动 CS2，进入在线匹配。以“能成功进入 VAC secure 对局”为唯一成功标准；进入大厅不算通过。
6. 如需 BOT 模式，必须在独立的 BOT/离线流程中使用 `-insecure`，退出后再次恢复官方文件并重新验证，再进行在线匹配。

建议在每一步记录：`gameinfo.gi` 与 Online 备份哈希、App 730 LaunchOptions、`counterstrikesharp.log` 是否产生新记录、Steam 验证结果和在线匹配结果。

## 5. 修复建议

### P0：立即恢复可在线状态

- 将“在线模式”定义为 **官方文件 + 无第三方插件/注入 + 无 BOT 参数**，不能只复制 Online `gameinfo.gi`。
- 在线启动前增加硬门禁：检测 Metamod/CounterStrikeSharp/本项目插件及已知第三方 DLL；发现仍存在时拒绝在线启动，并提示先恢复官方文件。
- 在线启动不应继承 `-allow_third_party_software`、`-insecure`、`-console`、`-condebug` 等 BOT/调试参数。
- BOT 启动失败或 CS2 异常退出后，提供原子恢复和重启前状态检查；恢复失败时明确阻止在线入口。

### P1：防止再次发生

- 在助手中把“在线”和“BOT（不安全）”做成互斥状态，显示当前文件完整性/插件残留检查结果。
- 启动结果不能只报告 Steam 已接受 `-applaunch 730`；必须等待 `cs2.exe` 并记录最终命令行、模式和门禁结果。
- 增加验收测试：BOT 启动 -> 异常退出 -> 关闭助手 -> Steam 默认启动 -> VAC secure 匹配；任何一步失败都不发布为“在线可用”。
- 将本报告和后续真实复测结果放入 `docs/`，不要把本地安装目录中的密钥、账号或完整用户数据提交到 Git。

## 6. 最终判定

本次问题应定性为：**助手使用/保留了 BOT 插件化客户端环境，随后即使从 Steam 正常入口启动，客户端仍未恢复到 VAC 所需的官方完整状态，因此大厅可进入但在线匹配阶段被 VAC 拒绝。**

证据强度：

- VAC 弹窗类型：高。
- 插件/运行时补丁曾加载：高（CounterStrikeSharp 日志）。
- 当前活动 `gameinfo.gi` 为 Online：高。
- “第三方文件残留是唯一直接触发文件签名拒绝的对象”：中高，需完成 Steam 文件验证和干净 Steam 启动复测后最终确认。

本调查未修改 CS2 安装目录、未删除插件、未执行 Steam 文件验证，也未把“配置已恢复”冒充“在线匹配已修复”。
