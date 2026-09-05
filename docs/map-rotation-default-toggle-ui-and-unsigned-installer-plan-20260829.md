# 自动换图默认开关入口与 0.5.10 未签名安装器交接方案

日期：2026-08-29  
工作区：`E:\CS2AS05`  
角色：方案制定 AI 交给实际执行 AI  
状态：调查与实施方案；本轮未修改业务代码、未构建安装器

## 1. 用户目标与完成标准

用户已经要求插件提供外部默认配置，但在助手界面中找不到配置按钮。本轮要补齐“助手程序配置入口”，并生成不注入私钥的安装程序供功能验证。**按钮只能放在概览页，禁止在“安装与诊断”页增加入口。**

- 玩家在助手概览页能看到并操作“自动换图默认开关”；
- 默认仍为开启；按钮实际读写外部 JSON，而不是只改 Vue 内存状态；
- 配置修改只作为 MapRotation **下一次插件载入的默认值**。CS2 运行中不热覆盖插件内存状态，游戏内 `lbtv_map_rotation 0|1` 逻辑保持不变；
- CS2 未运行时修改配置，下一次 BOT/插件启动能读到 `enabled=false/true`；
- 生成一个可安装的 0.5.10 NSIS 安装器，不使用 `TAURI_SIGNING_PRIVATE_KEY`，并明确这是未签名/仅本地功能验证候选，不是正式发布包；
- 既有用户配置升级时不得被内置默认 JSON 覆盖。

## 2. 已调查的真实基线

### 2.1 插件和资源已经存在

- 插件源码：`E:\CS2AS05\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.cs`。
- 插件已从 `Server.GameDirectory` 读取：
  `addons/counterstrikesharp/configs/plugins/MapRotation/MapRotation.json`。
- `Load()` 缺失、损坏、非 object 或 `enabled` 非 bool 时回退 `true`；正常默认配置内容是 `{"enabled":true}`。
- 自动换图延迟已统一为 15 秒；`lbtv_map_next` 仍立即换图；`lbtv_map_rotation 0|1` 只修改运行时内存，不回写 JSON。
- `src-tauri/resources/CS2BotImprover.zip` 已含 MapRotation DLL 和 JSON；安装事务对该 JSON 已有“目标存在则保留”的设计。
- 详细既有证据见 `docs/map-rotation-delay-and-external-default-execution-report-20260827.md`。

### 2.2 助手界面目前缺入口

- `src/views/OverviewView.vue` 当前有启动模式、BOT 难度、自动录制和启动 CS2，没有 MapRotation 设置。
- `src/views/InstallView.vue` 当前有资源包安装、写前备份一次性选项、原版 Panel、清除数据和卸载，没有自动换图设置。
- `src-tauri/src/commands/cs2.rs` 当前是目录、进程、关闭、安装/卸载等命令；没有读取/写入 MapRotation JSON 的独立命令。
- `src/services/tauri/` 目前没有 MapRotation service/type，不能通过浏览器 `fetch` 或 localStorage 实现本机文件配置。

## 3. 推荐实现位置与交互

### 3.1 入口位置

在 `src/views/OverviewView.vue` 的启动控制区域下新增一个完整的“地图轮换”设置行，作为唯一用户入口和运行控制。**不得修改 `src/views/InstallView.vue` 来增加按钮、开关、导航入口或重复说明。**不要复制两套状态逻辑，抽成：

- `src/components/MapRotationDefaultControl.vue`：展示和操作；
- `src/services/tauri/mapRotation.ts`：IPC 封装；
- `src/types/mapRotation.ts`：DTO；
- `src-tauri/src/commands/map_rotation.rs` 或职责相近的 `commands/panel.rs`：命令；
- `src-tauri/src/services/map_rotation.rs`：路径、JSON 校验、原子写和状态读取。

保持现有 UI 约定：Lucide 图标、最小 44px 点击区、可见 focus、浅色/深色/palette 语义 token、`aria-live` 状态；使用开关控件而不是把文字做成伪按钮。文案建议：

- 标题：`自动换图默认状态`；
- 开启：`下一次插件载入时自动换图`；
- 关闭：`下一次插件载入时保持当前地图`；
- 辅助说明：`只影响下一次 MapRotation 载入；游戏运行中请使用 lbtv_map_rotation 0|1。`；
- 路径折叠显示，提供“打开配置所在目录”仅作为可选辅助，不用浏览器打开或编辑文件。

### 3.2 状态和操作

- 进入页面时调用 `get_map_rotation_default(rootPath)`；显示 `enabled`、配置路径、`source=existing|default|fallback`、`updatedAt`（若可取）。
- CS2 `running/unknown` 时开关禁用，提示“请先退出 CS2，再修改下一次载入默认值”；不能热写配置。
- CS2 `stopped` 且目录完整时，用户切换开关调用 `set_map_rotation_default(rootPath, enabled)`；成功后以 Rust 返回值更新，不做乐观成功。
- 文件不存在时，首次读取显示“尚未建立，当前默认开启”；用户点击后由 Rust 创建标准 JSON。
- JSON 损坏或字段类型错误时，不覆盖用户文件；显示“配置损坏，已回退开启”，提供明确的“恢复为默认开启”按钮，恢复动作须经过用户点击并生成一次受控备份/日志。
- 不持久化到 localStorage；根目录选择沿用现有 store/持久化机制。

## 4. 后端 IPC 契约

建议 DTO（serde camelCase）：

```text
MapRotationDefault {
  rootPath: string,
  configPath: string,
  enabled: boolean,
  source: "existing" | "default" | "fallback",
  writable: boolean,
  warning: string | null
}
```

命令：

```text
get_map_rotation_default(root_path: String) -> Result<MapRotationDefault, String>
set_map_rotation_default(root_path: String, enabled: bool) -> Result<MapRotationDefault, String>
reset_map_rotation_default(root_path: String) -> Result<MapRotationDefault, String>
```

实现约束：

1. 通过 `cs2::normalize_root()` 校验根目录；最终路径必须 canonical/验证位于 `<root>\game\csgo\addons\counterstrikesharp\configs\plugins\MapRotation\MapRotation.json`，禁止任意路径参数。
2. `get` 使用 Rust `serde_json` 解析 object，`enabled` 缺失视为默认 `true`；记录 `source`，不自动修改损坏文件。
3. `set` 只接受 bool，写入格式化 JSON（可附 `schema:1`，但插件读取必须兼容该字段），使用临时文件 + flush/sync + 原子 rename + 写后回读。
4. 写入前仅在用户明确点选“保留本次备份”时保留写前副本；默认遵守当前全局备份策略，不产生长期 `.backup-*` 垃圾。失败回滚仍保留短生命周期安全副本。
5. `set/reset` 必须先调用结构化 CS2 进程快照；不是 `steam.exe` 检查。运行中返回稳定 `[MAP_ROTATION_CS2_RUNNING]`，不写文件。
6. 插件运行时不轮询此 JSON，不把 `lbtv_map_rotation 0|1` 写回文件；下一次 `Load()` 才重新读取默认值。
7. `src-tauri/src/lib.rs` 注册命令；`src/services/tauri/mapRotation.ts` 在非 Tauri 浏览器模式返回“仅桌面程序可用”，不能偷偷使用浏览器 File System API。

## 5. 安装升级和资源包规则

- 保留 `src-tauri/src/services/cs2.rs` 对 MapRotation JSON 的目标已存在则保留逻辑；增加测试证明升级 0.5.10 不覆盖玩家的 `enabled=false`。
- 新安装目标没有 JSON 时，从 ZIP 安装 `{"enabled":true}`；安装完成后读回并显示 `source=existing`。
- ZIP 必须同时包含 DLL、JSON、插件 marker 和 payload digest；重新合并资源时不能只替换 JSON 而跳过 marker 更新。
- 若执行 AI 修改了插件源码或 DLL，先运行：

```powershell
dotnet build .\third_party\CS2-Bot-Improver-map-rotation\addons\counterstrikesharp\plugins\MapRotation\MapRotation.csproj -c Release --nologo
```

然后重新生成/合并 ZIP，使用 `System.IO.Compression.ZipFile` 回读 JSON、DLL、marker 和 SHA-256。没有源码变化时不得无意义重编 DLL。

## 6. 自动化测试

至少新增或更新：

- `tests/map-rotation-settings.spec.ts`：组件显示默认开启、读取 existing/fallback、CS2 running 禁用、切换传参、错误提示、`aria-label`、键盘操作、无 localStorage。
- `tests/map-rotation-contract.spec.ts`：源码命令名、路径、JSON 字段、运行时指令不回写、15 秒延迟和手动立即换图回归。
- Rust `map_rotation` service tests：路径越界、缺失、损坏、非 bool、true/false、原子写回读、CS2 运行阻止、升级保留用户 false、清理失败告警。
- 安装事务测试：全新目录得到默认 JSON；已有 false 在安装/更新后仍是 false。

最小验证命令：

```powershell
npm test -- --run tests/map-rotation-contract.spec.ts tests/map-rotation-settings.spec.ts
npm run typecheck
npm run lint
npm run build:web
cargo test --manifest-path .\src-tauri\Cargo.toml map_rotation panel cs2
```

## 7. 构建不带私钥的验证安装器

用户只需要功能验证，本轮不要读取、注入或保存私钥，也不要生成 updater 签名声明。执行顺序：

1. 确认工作树版本仍为 `0.5.10`，记录构建前 `git status --short`，保留所有既有脏改动。
2. 完成上述测试和 ZIP 回读；确认安装资源内 JSON 默认开启且 marker/payload 一致。
3. 清理仅本轮产生的旧 `src-tauri/target/release` 构建锁定进程（不得删除用户文件或其他工作树产物）；如无需清理则直接构建。
4. 在不设置 `TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`、`TAURI_SIGNING_PRIVATE_KEY_PATH` 的 PowerShell 进程执行：

```powershell
Set-Location E:\CS2AS05
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD -ErrorAction SilentlyContinue
Remove-Item Env:TAURI_SIGNING_PRIVATE_KEY_PATH -ErrorAction SilentlyContinue
npm run bundle:desktop
```

5. 若 Tauri 配置强制签名导致构建失败，使用项目已验证的“未签名本地构建”配置/环境开关；不得填假私钥、不得修改正式 updater 公钥、不得把失败安装器当成可用产物。必要时先执行 `npm run build:desktop` 生成无 bundle 的 EXE，再根据现有 Tauri 配置补齐未签名 NSIS 构建。
6. 构建完成后回读：

```powershell
$installer = Get-ChildItem .\src-tauri\target\release\bundle\nsis\*0.5.10*x64-setup.exe | Sort-Object LastWriteTime -Descending | Select-Object -First 1
Get-Item $installer.FullName | Select-Object FullName,Length,LastWriteTime
Get-FileHash $installer.FullName -Algorithm SHA256
```

安装器报告必须写明：`unsigned/local validation only`，没有 `.sig` 或 `.sig` 未生成不视为失败；Windows Authenticode 不得声称已签名。安装器最多尝试 5 次，遇到同一构建错误不要循环重试。

## 8. 功能验收矩阵

在隔离测试 CS2 目录中安装该未签名候选：

1. 首次安装：助手界面显示“自动换图默认状态：开启”，路径准确。
2. 退出 CS2 后点击关闭开关：JSON 回读为 `{"enabled":false}`，刷新助手仍显示关闭。
3. 启动 BOT/插件后执行 `lbtv_map_rotation`：默认状态为关闭；执行 `lbtv_map_rotation 1` 立即开启，且 JSON 仍为 false。
4. 退出 CS2，界面改回开启；下一次插件载入后默认开启。
5. CS2 运行期间尝试切换：按钮禁用或返回 `[MAP_ROTATION_CS2_RUNNING]`，JSON 不变。
6. 手工把 JSON 改为 false，再升级安装 0.5.10：文件保持 false，助手显示 false。
7. 手工写入损坏 JSON：助手显示回退开启/修复提示，不静默覆盖；点击“恢复默认”后才写入 true。
8. 完成一局：最终比分后约 15 秒自动换图；`lbtv_map_next` 仍立即换图。此项需真实 CS2/插件日志，不可用浏览器页面替代。
9. 浅色/深色、1280x800、980x640 和移动宽度：设置入口可见、无横向溢出、焦点可见、状态文字不被其他 UI 遮挡。

## 9. 停止条件与交付物

以下任一项出现即停止发布候选：界面开关只改内存不改 JSON、运行中可以写文件、升级覆盖 false、JSON 路径可逃逸根目录、插件 marker 未更新、无私钥构建仍引用签名变量、安装器无法确认版本、或真实插件仍读不到开关。

执行 AI 最终必须更新执行报告，至少包含：

- 变更文件绝对路径与 IPC 契约；
- 实际 JSON 路径和 true/false 回读；
- 测试命令和结果；
- ZIP entry、DLL、marker、安装器 SHA-256；
- 未签名状态和安装器绝对路径；
- 真实 CS2 日志/截图（若用户完成）及尚未完成的验收项。

在真实 CS2 验收完成前，状态只能写“未签名功能验证候选/待用户验收”，不得写“正式发布”或“自动换图问题已彻底解决”。
