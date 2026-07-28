# 0.5.5 Steam 启动修复与进页更新检查交接方案

## 1. 交接目标与执行边界

本文交给下一位【实际执行 AI】。两个 AI 不共享聊天上下文，实际执行时只以本文、当前工作区和实时运行证据为准。

仓库：`E:\CS2AS05`

制定方案时的基线：

- 分支：`main`
- 提交：`6f770257a36034b27e276cd8576b2c0fa97c26f1`（`release: CS2 Bot Improver 0.5.4`）
- 工作区：干净
- 本地/远端标签：`v0.5.4` 指向上述提交（本地为 annotated tag）
- 当前应用、Cargo、Tauri、侧栏和安装页版本：`0.5.4`
- 上游插件资源仍为 `ed0ard/CS2-Bot-Improver v1.4.3`

本次目标：

1. 开始 `0.5.5`，修复“已经识别 CS2 游戏目录，但启动时报 `[STEAM_NOT_FOUND] 未找到 steam.exe。`”的问题。
2. 把“安装与诊断”页的更新检查入口前置；用户每次进入该页面都立即触发一次自动检查。
3. 保持手动检查、推荐/普通更新的关闭语义、Tauri 签名自更新和夸克回退不变。
4. 完成版本一致性、自动化、签名打包、隔离验证和发布准备；真实 Steam/CS2 启动由用户验收。

不要新建分支。不要使用 `git reset`、`checkout`、`restore`、`clean` 回退当前或后来出现的已有改动。先重新执行 `git status --short` 并审阅差异，再增量修改。不要修改真实 CS2 目录来做自动化测试，不要结束归属不明的 Steam、CS2 或助手进程。

## 2. 用户问题与根因证据

用户截图：

```text
C:\Users\GOPtZ\AppData\Local\Temp\codex-clipboard-10cbb402-8842-4ce9-b940-84dca1e20f19.png
```

截图中已经选择：

```text
E:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive
```

界面同时显示 `Panel 可用`，点击启动后得到：

```text
[STEAM_NOT_FOUND] 未找到 steam.exe。
```

这不是 CS2 根目录识别失败。当前真实调用链是：

```text
src/views/OverviewView.vue
  -> src/composables/useCs2LaunchExperience.ts
  -> src/stores/panel.ts::launch
  -> src/services/tauri/panel.ts::launchPanelCs2
  -> src-tauri/src/commands/panel.rs::launch_panel_cs2
  -> src-tauri/src/services/panel.rs::launch_cs2
```

`src-tauri/src/services/panel.rs:1217` 的 `find_steam()` 只检查：

```text
%PROGRAMFILES(X86)%\Steam\steam.exe
%PROGRAMFILES%\Steam\steam.exe
```

但 `src-tauri/src/services/cs2_discovery.rs` 已经能从以下来源发现 Steam/CS2：

- `HKCU\Software\Valve\Steam` 的 `SteamPath`、`SteamExe`
- `HKLM\Software\Valve\Steam` 和 Wow6432Node 的 `InstallPath`
- 正在运行的 `steam.exe` 进程路径
- 多盘常见 Steam 根目录
- `steamapps\libraryfolders.vdf`
- App 730 manifest 和卸载注册表

因此根因是“CS2 目录发现”和“启动时 Steam 可执行文件发现”使用两套能力不一致的实现。自定义 Steam 安装目录不在 Program Files 时，前者成功、后者失败。

还有一个需要同时修正的副作用：当前 `launch_cs2_inner` 在查找 `steam.exe` 之前会初始化 Panel 默认值、可能自动安装插件并切换 `gameinfo.gi`。Steam 最终找不到时，启动虽然失败，游戏目录却可能已经发生写入。0.5.5 应在任何插件安装、配置初始化和模式切换前先证明 Steam 启动器存在。

## 3. 当前更新检查事实

当前 `src/views/InstallView.vue` 在安装、Panel、卸载区之间渲染 `<SupportActions />`。`SupportActions.vue` 的 `onMounted` 已调用 `check(false)`；由于 `AppShell.vue` 使用动态视图，进入安装页时组件会挂载，离开后会卸载。

因此源码已经具备一次“挂载即自动检查”的雏形，但契约不够明确：

- 更新入口位于页面较后位置，不符合“检查更新逻辑提前”的呈现要求。
- `tests/support-actions.spec.ts` 只间接覆盖首次挂载，没有明确断言参数为 `false`。
- 没有覆盖“离开页面再进入会重新检查”。
- 没有覆盖快速重复进入时仍由 `features/software-updates/state.ts` 的 `inFlight` 合并网络请求。

0.5.5 不需要引入新的路由器或全局更新框架，也不要把自动检查改成 `manual=true`。自动检查继续使用 `false`，从而保留普通/推荐更新的关闭规则；用户点击“检查更新”时才使用 `true` 并允许重新展示更新弹窗。

## 4. 线上状态快照（2026-07-27）

制定方案时只读验证到：

- `GET /api/software-updates/cs2-bot-improver?currentVersion=0.5.4&channel=prod` 返回 `200`、`hasUpdate=false`。
- 生产 latest 为 `0.5.4`，`selfUpdate.available=true`。
- 生产 0.5.4 安装包元数据：`69,384,887` bytes，SHA256 `7D6D9DCF62514E6356A275ABE936C1B6AFEDF79971327818146F31EFFB5E9B43`。
- `GET /api/software-updater/cs2-bot-improver/prod/windows/x86_64/0.5.4` 返回 `204`，符合当前版无需更新。
- GitHub `main` 与本地基线一致，远端已有 `v0.5.4` tag；GitHub Releases API 当前返回 `404`，即没有 GitHub Release 条目。
- 发布后端位于 `E:\cs2as`，当前有既存未跟踪证据目录，严禁清理；发布脚本为 `E:\cs2as\scripts\publish-self-update.mjs`。

这些是制定方案时的快照，实际发布前必须重新查询，不能把它们当作持续有效的生产状态。

## 5. 实施方案

### 5.1 第一阶段：建立 0.5.5 版本基线

精确修改项目自身版本字段：

- `package.json` 顶层 `version`
- `package-lock.json` 顶层 `version` 与根 package 的 `version`
- `src-tauri/Cargo.toml` 的当前 crate `package.version`
- `src-tauri/Cargo.lock` 中仅 `cs2-bot-improver-assistant` package 的版本
- `src-tauri/tauri.conf.json::version`

禁止全局替换 `0.5.4`。依赖版本、历史文档、兼容性 fixture 和“0.5.4 新增命令”等文本不是当前应用版本。

可顺手消除两个容易漂移的可见硬编码：

- `src/components/AppShell.vue` 导入 `appConfig`，侧栏显示 `appConfig.appVersion`。
- `src/views/InstallView.vue` 导入 `appConfig`，页头、资源包标题和提示使用当前版本插值。

不需要改产品名、Tauri identifier、上游版本、Panel 文件名、更新 channel 或 updater 公钥。

### 5.2 第二阶段：统一 Steam 可执行文件解析

优先在 `src-tauri/src/services/cs2_discovery.rs` 增加可复用的 crate 内接口，不要在 `panel.rs` 再复制一套注册表代码：

```rust
pub(crate) fn find_steam_executable(root_hint: Option<&Path>) -> Option<PathBuf>
```

建议的确定性优先顺序：

1. 正在运行且 `process.exe()` 可回读的 `steam.exe` 完整路径。
2. `HKCU\Software\Valve\Steam\SteamExe` 的完整文件路径。
3. `HKCU\Software\Valve\Steam\SteamPath` 加 `steam.exe`。
4. HKLM 32/64 位视图的 `InstallPath` 加 `steam.exe`，包括 Wow6432Node。
5. 从已经规范化的 CS2 root 反推 `<library>\steam.exe`。对于 `<library>\steamapps\common\Counter-Strike Global Offensive`，只把 `<library>\steam.exe` 作为候选；它适用于 Steam 主目录本身也是该库的情况，不假设每个附加库都有 Steam EXE。
6. 当前 `common_steam_roots()` 的多盘常见目录加 `steam.exe`。

候选必须：

- 去重后按上述顺序检查。
- `is_file()` 为真。
- 文件名大小写不敏感地等于 `steam.exe`。
- 不执行来自 VDF、注册表或环境变量的任意命令行字符串，只把它们当路径。

同时把 `discover_locations()` 使用的 Steam 根目录来源改为共享辅助函数，避免以后再次产生两套规则。保留现有 VDF 解析、App 730 manifest 排序和 10 秒深度扫描行为。

不要使用以下回退：

- 不直接启动 `game\bin\win64\cs2.exe`，这会绕开 Steam app/session 语义。
- 不调用 `window.open`、浏览器 scheme 或网页跳转启动游戏。
- 不从 PATH 盲目执行字符串 `steam`。
- 不因为找不到 Steam 而扫描整块磁盘上的任意 `steam.exe`。

### 5.3 第三阶段：调整启动事务顺序

修改 `src-tauri/src/services/panel.rs::launch_cs2_inner`：

1. 校验 `mode`。
2. 获取现有 BOT 启动互斥锁。
3. 检查 `cs2.exe` 未运行。
4. `normalize_root(root_path)`，证明选择的 CS2 目录有效。
5. 调用共享 `find_steam_executable(Some(&root))`；找不到则立即返回 `[STEAM_NOT_FOUND]`。
6. 只有上述只读前置条件都通过后，BOT 模式才初始化默认值并执行插件版本门禁/必要的自动安装。
7. 再检查一次 `cs2.exe`，然后 `set_mode`。
8. 用已经解析并验证过的 Steam 路径执行现有参数：在线模式 `-applaunch 730`；BOT 模式再附加 `-insecure -console -condebug`。

不要改变 `LaunchResult` 前端契约。不要增加用户手填 Steam 路径作为首选方案；注册表/进程/常见路径足以覆盖本次报告，手填入口会产生新的持久化、校验和支持成本。

错误仍保留稳定 code `[STEAM_NOT_FOUND]`，但可把中文信息改成可操作描述，例如：

```text
[STEAM_NOT_FOUND] 未找到 Steam 客户端。请确认 Steam 已安装，或先启动一次 Steam 后重试。
```

日志不得记录注册表原始内容，只记录最终采用的 Steam 路径或经过了哪些来源类别。

### 5.4 第四阶段：更新检查前置与进页触发

修改 `src/views/InstallView.vue`：

- 将 `<SupportActions />` 移到 `installer-header` 之后、目录扫描和插件安装之前。
- 保持当前全宽工具区，不新增嵌套卡片、不改变整体配色、不使用浏览器原生 `alert/confirm/prompt`。
- 窄窗口下复用现有响应式规则，按钮文字必须换行或自适应，不能溢出。

修改/保留 `src/components/SupportActions.vue`：

- 组件每次挂载执行且只执行一次 `void check(false)`。
- 手动按钮继续调用 `check(true)`。
- 自动检查期间显示“正在检查更新...”，并禁用重复检查按钮。
- 自动失败只显示当前内联可恢复消息，不阻止目录、安装、诊断和卸载功能。
- 发现新版本时继续使用现有 `shouldPresentRelease`；不要让普通版本在用户已关闭后每次进页强制弹窗。
- 离开再进入页面应发起新的检查；同一个仍在进行的请求由 `checkForSoftwareUpdates` 的 singleton `inFlight` 复用，不重复联网。

不要把检查移动到应用启动或 `AppShell` 全局 `onMounted`。用户要求的触发边界是进入“安装与诊断”页；全局检查会改变当前产品行为、使弹窗在其他操作页出现，并可能与页面组件产生双请求和双弹窗。

### 5.5 第五阶段：同步内置插件 marker

这是 0.5.5 必做项。当前 BOT 启动门禁用 `CARGO_PKG_VERSION` 与游戏内 `CS2AS05.plugin.json::version` 比较；内置 ZIP marker 当前也是 `0.5.4`。如果只把应用升到 0.5.5，门禁会在每次 BOT 启动时把 0.5.4 插件判为旧版并反复覆盖。

版本字段更新完成后执行：

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\generate-plugin-manifest.ps1
```

该脚本只重写 ZIP 内：

```text
addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json
```

验收要求：

- marker `version` 为 `0.5.5`。
- `payloadSha256` 仍为 `9435A14E08F852FFE695DF1125310B623B7FF445CA7612A66D59AFE727317723`，除非实际执行时另有经过审阅的插件 payload 改动。
- payload entry 清单不变。
- 记录新 ZIP size/SHA256。
- 把 `src-tauri/src/services/cs2.rs::CUSTOM_ZIP_SHA256` 更新为新 ZIP 摘要。
- README 当前资源摘要更新为新值。
- 新建 0.5.5 资源证据 JSON，明确相对 0.5.4 只有 marker 版本/归档摘要改变；不要覆盖历史 `docs/CS2BotImprover-defaults-diff-0.5.4.json`。

`scripts/update-panel-defaults.ps1` 是 0.5.4 默认值构建脚本，本次没有变更 Aim/Nades/刀具默认值，不要为版本号更新误跑它或全局改写历史报告。

## 6. 测试与验收

### 6.1 Rust 单元测试

在 `cs2_discovery.rs` 对路径解析抽出可注入候选的纯函数，至少覆盖：

1. 第一个候选缺失、第二个自定义目录 `steam.exe` 有效时选择第二个。
2. 从临时 `<library>\steamapps\common\Counter-Strike Global Offensive` root hint 找到 `<library>\steam.exe`。
3. 文件名不是 `steam.exe` 时拒绝。
4. 重复候选只检查一次，优先级稳定。
5. 所有候选无效时返回 `None`。

注册表和进程枚举只做薄适配；不要让测试依赖本机真实 Steam 注册表。现有 `cs2_discovery` VDF/扫描测试必须全部保留。

在 `panel.rs` 或契约测试中证明 `find_steam_executable` 的调用顺序早于：

- `initialize_panel_defaults_at`
- `ensure_current_bot_plugin`
- `set_mode`

理想验证是抽出可注入的只读前置准备函数；如保持当前函数结构，则增加窄范围 source-order 契约测试，并在真实回归中对失败前后的 `gameinfo.gi`、Panel state 和插件 marker 做字节比较。

### 6.2 前端测试

更新 `tests/support-actions.spec.ts`：

- 首次挂载明确断言 `checkForSoftwareUpdates(false)` 一次。
- 手动按钮随后断言 `true`。
- 自动检查未结束时按钮禁用。
- 失败消息不移除其他支持按钮。

新增 AppShell/导航行为测试：

1. 初始概览页不检查更新。
2. 第一次进入“安装与诊断”调用一次 `false`。
3. 离开后再次进入再调用一次 `false`。
4. 前一次 promise 未完成时快速离开再进入，底层 `requestSoftwareUpdate` 只有一个实际 fetch；两个调用共享 `inFlight`。

保留并重跑：

- `tests/software-updates.spec.ts`
- `tests/software-updater-state.spec.ts`
- `tests/software-update-modal.spec.ts`
- `tests/app-close.spec.ts`
- `tests/launch-experience.spec.ts`
- `tests/bot-plugin-gate.spec.ts`

### 6.3 自动化命令

```powershell
git diff --check
npm run verify
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path .\src-tauri\Cargo.toml
```

如检测到真实 `cs2.exe` 正在运行而触发受保护测试失败，不得强制结束用户进程；保存输出并让用户在方便时退出游戏后重跑。不要使用 Vitest 不支持的 `--runInBand`。

### 6.4 UI 验证

按现有桌面 utility 风格验证亮/暗主题：

- `720x620`
- `960x700`
- `1280x800`

检查：更新区位于页头之后；首次进入立即显示检查中；长错误消息和“安装已下载版本并重启”不溢出；弹窗不被标题栏/导航遮挡；键盘 focus 可见；`prefers-reduced-motion` 下 spinner/切页遵循现有规则。

使用真实网站更新源的效果可由 Codex 内置浏览器验证 Web API；桌面 Tauri 网络、自更新 resource 和系统进程启动必须用打包后的程序验证，不能把普通 Vite 页面结果当作桌面验收。

### 6.5 用户真实 Steam/CS2 验收

这一阶段由用户执行。实际执行 AI 提供安装包和最短步骤，不替用户声称成功：

1. 在出现截图问题的机器安装 0.5.5，Steam 保持未运行。
2. 选择同一 `E:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive`。
3. 在线模式启动一次，确认 Steam/CS2 打开且没有 `[STEAM_NOT_FOUND]`。
4. 退出 CS2，BOT 模式启动一次，确认参数仍含 `-insecure -console -condebug`。
5. 再覆盖测试 Steam 已运行、自定义安装目录和 Program Files 标准安装三种场景。
6. 进入安装与诊断页，确认立即检查；离开再进入，确认再次检查；手动按钮仍可检查。
7. 保存助手诊断日志和 `console.log`，只需报告是否启动成功及参数，不需要做新的三回合 Nade 平衡验收。

若 Steam 客户端确实不存在，预期仍返回 `[STEAM_NOT_FOUND]`，且失败前后以下文件/目录保持不变：

- `game/csgo/gameinfo.gi`
- `game/csgo/cfg/cs2as05-panel-state.json`
- `game/csgo/addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json`

## 7. 构建、签名与发布

### 7.1 候选构建

只在自动化通过后执行：

```powershell
npm run build:desktop
```

无签名的 no-bundle 二进制只用于本地验证，不能发布为 self-update。

### 7.2 最终签名安装包

确认当前 shell 已安全注入：

```powershell
Test-Path Env:TAURI_SIGNING_PRIVATE_KEY
Test-Path Env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD
```

然后：

```powershell
npm run bundle:desktop
$env:RELEASE_CHANNEL = 'prod'
npm run release:manifest
```

必须得到 `CS2人机增强助手_0.5.5_x64-setup.exe` 和同名 `.sig`。记录路径、size、SHA256、Tauri signature 验证结果和 Authenticode 状态；Tauri updater 签名与 Windows Authenticode 是两件事。

对最终 EXE 重跑隔离新装、启动、0.5.4 -> 0.5.5 升级、退出和卸载验证。升级必须保留 sentinel/app data；卸载分别报告程序文件删除与用户数据保留。

### 7.3 发布顺序

发布前重新抓取生产 D1/R2/API/feed 快照。`E:\cs2as` 有既存未跟踪证据，禁止清理或提交无关内容。

1. 用户创建并提供 0.5.5 夸克分享链接；实际执行 AI 不代建网盘链接。
2. 在 `E:\cs2as` 先 dry-run：

```powershell
npm run self-update:publish -- --installer "<0.5.5-exe>" --sig "<0.5.5-exe>.sig" --version 0.5.5 --quark-url "<用户提供链接>" --title "v0.5.5 更新" --summary "修复自定义 Steam 安装目录下无法启动 CS2，并提前安装与诊断页的更新检查。" --items "修复已识别 CS2 目录但提示未找到 steam.exe|进入安装与诊断页即检查更新|保留官网直连自更新与夸克回退"
```

3. 核对 key 必须为 `software-updates/cs2-bot-improver/prod/0.5.5/...`，摘要与最终 EXE 完全一致。
4. 远程发布前把安装包复制到纯 ASCII 本地路径，避免历史上中文路径 R2 PUT 显示成功但对象未更新的问题。
5. 远程发布最多尝试 3 次；必须 R2 GET 回读到本地并比对 exact size/SHA256 后，才能让脚本启用 D1 `updater_enabled`。
6. 验证：
   - 0.5.4 custom API 返回 `hasUpdate=true`、latest 0.5.5。
   - 0.5.4 Tauri feed 返回 `200`、version/signature/url 正确。
   - 0.5.5 custom API 返回 `hasUpdate=false`。
   - 0.5.5 Tauri feed 返回 `204`。
   - signed download HEAD/GET size 和 SHA256 正确，CORS 正确。
   - 夸克回退仍显示且 URL 正确。
7. 当前仓库普通 Git push 最多一次，不 force push。提交和 tag 可在一次 push 中发送；先确认提交不含私钥、真实游戏备份、临时证据和 `E:\cs2as` 的无关文件。
8. GitHub Release 当前不存在。若本次决定补建，上传动作最多一次，附 EXE、`.sig`、SHA256、变更说明和已知限制；不要用 GitHub Release 是否存在替代官网 D1/R2/feed 验证。
9. 群公告、内测邀请由用户执行。

## 8. 回退与停止条件

- Steam resolver 自动测试失败：不保留 direct-CS2 或 URI 临时回退，回到候选来源和排序验证。
- 找不到 Steam 时仍发生 `gameinfo.gi`、插件 marker 或 Panel state 变化：停止发布，修正启动事务顺序。
- 0.5.5 应用搭配 0.5.4 marker：停止构建，重新生成 marker 和 ZIP 摘要。
- 自动检查进入页面触发两次实际 fetch：停止 UI 验收，修正入口所有权或 `inFlight` 复用。
- 签名缺失、`.sig` 与 EXE 不匹配：只保留本地候选，不切生产。
- R2 回读、D1、custom API、Tauri feed 任一 size/hash/signature/version 不一致：保持或恢复 0.5.4 latest，不发布 0.5.5。
- 工作区出现无法归属的已有改动：保留现场并增量兼容；只有确实阻断目标时再询问用户。
- 真实 Steam/CS2 尚未验证：可以交付“0.5.5 签名候选”，不能报告截图问题已在真实机器解决。

## 9. 实际执行 AI 最终报告模板

```text
结果：0.5.5 本地候选 / 签名候选 / 正式发布
基线与 dirty work：branch、commit、保留的既有改动
Steam 修复：候选来源顺序、最终采用路径类别、失败前无写入验证
更新检查：首次进入、重复进入、in-flight 合并、手动检查
版本一致性：npm/Cargo/Tauri/UI/ZIP marker
资源：ZIP size/SHA256、marker version、payloadSha256
自动化：npm verify、fmt、clippy、cargo test
UI：720x620、960x700、1280x800，亮/暗/reduced-motion
安装包：路径、size、SHA256、.sig、Authenticode
隔离安装：新装、0.5.4 升级、退出、卸载、sentinel
用户实机：自定义 Steam 未运行/运行、在线/BOT 启动、进页更新检查
线上：D1、R2 回读、custom API、Tauri feed、夸克、GitHub Release
Git：commit、tag、push（是否执行）
剩余限制与回退证据路径
```

只有用户真实复现机器上的 Steam/CS2 启动通过，并且生产回读全部一致时，才可称为 0.5.5 正式修复完成。
