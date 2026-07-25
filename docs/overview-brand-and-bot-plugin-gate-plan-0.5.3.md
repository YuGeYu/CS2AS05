# 0.5.3 标识更新与 BOT 启动插件版本门禁方案

本文件是完整交接依据。执行 AI 不需要查看用户截图；以下所有 UI 目标均由文件路径、DOM 结构、CSS 类名和精确文本定义。

状态：待实际执行 AI 实施
方案日期：2026-07-25
工作区：`E:\CS2AS05`
当前基线：`main` / `9c3564106249edfa421a6cd0dbefdd52adde00df`（0.5.3，尚未推送）
远端基线：`origin/main` / `13b721839494fb1bdeef9bdd70e8b3d20a4b732d`（0.5.2）
目标发布：仍为 `0.5.3`，不升版；实施后把新增改动合并进现有 0.5.3 提交，再普通推送一次。

## 1. 交付目标

本方案只处理本次新增需求，不重复实现上一轮已经进入基线的启动特效和“猜你想选”扫描：

1. `src/components/AppTitlebar.vue` 中 `.titlebar-brand` 的第一个子元素 `.titlebar-mark` 从文字 `CS2` 改为本程序正式图标。
2. `src/components/AppShell.vue` 中 `.sidebar-brand > div > small` 的完整文本从 `Panel 0.5.3` 改为只有 `0.5.3`。
3. 用户点击“启动 CS2”且当前模式为 BOT 时，在真正启动 Steam 前检查 CS2 目录中的本项目插件版本；插件版本当前程序版本或更高时直接继续启动，否则先自动安装内置定制资源包，再继续启动。
4. 在线模式启动路径不增加插件检查；“安装与诊断”页面的手动安装按钮、文案、禁用条件和 IPC 语义不做版本门禁改造。

真实 CS2 游戏内效果由用户验收；执行 AI 负责完成静态、单元、桌面启动链验证，并提供可复现的实机步骤。不得把“Steam 进程已接受参数”冒充“游戏已进入”。

## 2. 已调查的当前实现

### 2.1 需要改的 UI

- `src/components/AppTitlebar.vue` 当前 `.titlebar-brand` 的 DOM 为 `<div class="titlebar-brand" aria-hidden="true"><span class="titlebar-mark">CS2</span><span class="titlebar-name">CS2 人机增强助手</span></div>`。只替换 `.titlebar-mark` 节点的内容/元素类型；标题栏拖动、主题、最小化、最大化、关闭逻辑均在同一组件，不能因换图标而破坏。
- `src/components/AppShell.vue` 当前侧栏品牌的 DOM 为 `<div class="sidebar-brand"><span><PackageCheck /></span><div><strong>CS2 助手</strong><small>Panel 0.5.3</small></div></div>`。只把该 `<small>` 的文本改成 `0.5.3`；不得修改 `strong` 和 `PackageCheck` 图标。
- 正式 Tauri 图标已有于 `src-tauri/icons/icon.png` 及 `icon.ico` 等文件。前端不能直接假定 `src-tauri` 是 Vite 静态目录，建议新增 `src/assets/app-icon.png` 并以已有正式图标为唯一来源。
- `src/views/InstallView.vue` 仍显示资源包标题 `0.5.3 定制资源包`。这是安装页面的资源包说明，不是本次侧栏版本节点；除非版本生成机制需要统一，否则保持其现有文案。

### 2.2 BOT 启动链

- `src/views/OverviewView.vue::launch` 调用 `useCs2LaunchExperience.start(...)`。
- `src/composables/useCs2LaunchExperience.ts` 调用 `usePanelStore.launch(root, mode)`，已有请求、进程探测、30 秒计时和点击退出状态机。
- `src/stores/panel.ts::launch` 调用 `launch_panel_cs2`，并维护 `mutationKey` 与 `lastError`。
- `src-tauri/src/commands/panel.rs::launch_panel_cs2` 当前只接收 `root_path`、`mode`，没有 `AppHandle`。
- `src-tauri/src/services/panel.rs::launch_cs2` 当前顺序是：检查 `cs2.exe` -> `set_mode` -> 查找 `steam.exe` -> `-applaunch 730`；BOT 模式附加 `-insecure -console -condebug`。
- `src-tauri/src/services/cs2.rs::install_bot_package` 当前会检查 CS2 未运行、校验内置 ZIP 摘要、清理旧定制文件并解压。它是“安装与诊断”页面使用的手动安装命令，不能直接在此函数入口加入版本短路。

### 2.3 当前缺口

- 内置 ZIP 当前没有本项目插件版本标记；`inspect_cs2_root` 只有 Metamod、CounterStrikeSharp、gameinfo 和备份文件状态，没有可比较的插件版本。
- `Cargo.toml` 的 Rust package、`package.json`、`src-tauri/tauri.conf.json` 当前均为 `0.5.3`。运行时版本真值应来自 Rust `env!("CARGO_PKG_VERSION")`，不能把 TypeScript 中的 `0.5.3` 作为判断依据。
- 现有安装过程没有事务回滚能力。BOT 启动自动安装必须先准备临时目录并验证，不能删除 `addons` 后再裸解压。

## 3. UI 实施方案

### 3.1 标题栏图标

1. 从 `src-tauri/icons/icon.png` 复制或生成 `src/assets/app-icon.png`；记录源文件和目标文件 SHA256，确认不是重新绘制的近似图标。
2. 在 `src/components/AppTitlebar.vue` 导入该资源，替换：

   ```vue
   <span class="titlebar-mark">CS2</span>
   ```

   为固定尺寸的 `<img class="titlebar-mark" src="..." alt="" />`。该节点不再渲染文字 `CS2`，并保留 `.titlebar-name` 作产品名文字。
3. 在 `src/styles/main.css` 将 `.titlebar-mark` 固定为约 28-32px 方形、`object-fit: contain`、不参与拖动区域布局抖动；浅色/深色主题均保持清晰，不加渐变光球或持续闪烁。
4. 不修改 `@mousedown` 拖动、双击最大化以及右侧窗口控制按钮。

### 3.2 侧栏版本文字

将 `src/components/AppShell.vue` 中匹配器 `aside.sidebar > div.sidebar-brand > div > small` 的文本改成 `0.5.3`。旧文本必须是 `Panel 0.5.3`，新文本必须是 `0.5.3`。其余导航名称和图标不变；若使用动态版本，必须统一由构建时版本注入。

### 3.3 UI 验证

- Vitest 按 DOM 选择器 `aside.sidebar .sidebar-brand small` 断言其文本等于 `0.5.3`，且 `wrapper.text()` 不包含 `Panel 0.5.3`。
- 组件测试按 `.titlebar-brand .titlebar-mark` 断言渲染的是 `<img>`，其 `src` 指向 `app-icon.png`，且该节点不再有文本节点 `CS2`。
- 在 960x700、720x620 的 light/dark 界面检查图标不变形、不遮挡产品名、不影响拖动和窗口按钮。

## 4. 插件版本标记设计

### 4.1 标记文件

在定制 ZIP 中新增应用自有标记：

```text
addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json
```

它不是 CounterStrikeSharp 的插件清单，只供助手读取；JSON 文件不会被当作 DLL 加载。建议结构：

```json
{
  "schema": 1,
  "product": "cs2-bot-improver",
  "pluginId": "cs2as05-custom-package",
  "version": "0.5.3",
  "payloadSha256": "<canonical payload digest>",
  "generatedFrom": "CS2-Bot-Improver-v1.4.2"
}
```

`version` 必须由当前构建版本生成，不要手写固定值。`payloadSha256` 对本项目管理的 DLL、配置、数据文件按规范化相对路径、字节长度、内容排序后计算，并排除标记文件自身，避免自引用循环。生成脚本和 ZIP entry 清单必须可重复执行。

### 4.2 版本判断规则

- Rust 侧以 `env!("CARGO_PKG_VERSION")` 为当前程序版本，并在启动时解析标记版本；禁止使用字符串字典序。
- 使用固定版本的 `semver` crate，或实现严格的 `major.minor.patch` 数字解析。至少拒绝空版本、非数字核心版本、缺字段和溢出值。
- 为兼容测试版本，采用“数字核心版本”比较：已安装插件的 `major.minor.patch` 大于或等于当前程序核心版本即可通过；同核心版本的 `-alpha`、`-beta`、`-test.N` 允许通过，构建元数据忽略。若团队改用完整 SemVer，则测试版本必须使用高于当前版本的数字核心，例如 `0.5.4-beta.1`，不能把 `0.5.3-beta` 当成高于 `0.5.3`。
- 版本通过还不够：标记的 `product`、`pluginId`、schema、必需文件和 `payloadSha256` 必须全部有效。标记缺失、损坏、摘要不匹配或版本不可解析均视为“不可信，需要安装当前内置包”。
- 已安装插件版本核心高于当前且标记和 payload 有效时，必须保留该测试版/更高版，不能被 0.5.3 资源包降级覆盖。

### 4.3 服务 DTO 与错误

在 `src-tauri/src/models/cs2.rs` 增加内部/可序列化状态（可只在 Rust 内部使用）：

```text
PluginVersionStatus = Missing | Invalid(reason) | Valid { version, payload_valid }
LaunchResult += pluginAction: unchanged | installed
LaunchResult += pluginVersion: string
```

稳定错误码建议：

- `[BOT_PLUGIN_VERSION_INVALID]`：版本字段缺失或格式错误。
- `[BOT_PLUGIN_PAYLOAD_INVALID]`：标记与实际文件摘要不一致。
- `[BOT_PLUGIN_AUTO_INSTALL_FAILED]`：自动安装或回滚失败。
- `[BOT_PLUGIN_HIGHER_VERSION_UNTRUSTED]`：发现更高版本但无法证明其完整性时，禁止静默覆盖，要求用户走“安装与诊断”页明确安装。

最终错误应写入现有应用 `runtime.log` 和 `panel.lastError`/Toast，不向 CS2 控制台写检查日志。

## 5. BOT 启动门禁实施

### 5.1 后端入口

修改 `src-tauri/src/commands/panel.rs::launch_panel_cs2` 接收 `AppHandle`，传入 `services/panel.rs::launch_cs2`。不要在前端先调用一次检查再另起一次安装命令，否则存在竞态和绕过窗口。

在 `services/panel.rs::launch_cs2` 内只对 `mode == "bots"` 执行：

1. 检查 `cs2.exe` 未运行；正在运行则返回现有 `[CS2_RUNNING]`，不写文件。
2. 调用 `cs2::inspect_bot_plugin_version(root)`。
3. 状态为当前或更高且 payload 有效：`pluginAction = "unchanged"`，不触碰游戏文件。
4. 状态缺失、较低或不可信：调用内部 `cs2::ensure_bot_plugin_current(app, root)`，验证内置 ZIP 摘要、路径安全、manifest 和目标文件，安装当前 0.5.3 包，最后再次读取标记确认版本与摘要。
5. 再次检查 `cs2.exe` 未运行，执行现有 `set_mode`，然后按原契约查找 Steam 并启动 `-applaunch 730 -insecure -console -condebug`。
6. 任意检查/安装失败都不得执行 `set_mode` 或 `Command::spawn`；返回结构化错误并让现有启动特效关闭。

在线模式必须继续走原有“检查进程 -> set_mode -> 启动 Steam”路径，不读取或安装插件。

### 5.2 安装事务与并发

- 自动安装复用现有 ZIP 校验和 `safe_zip_path`，但不能直接复用“先删除旧目录、再解压”的不可回滚顺序。
- 安装前确认 CS2 未运行；将本项目管理的目标文件复制到同级临时目录，完整解压、校验 payload 和 manifest 后再原子替换；安装前文件备份放在应用管理的临时备份目录。
- 写入 `CS2AS05.plugin.json` 必须是最后一步。失败时恢复备份并删除临时目录；恢复失败要报告明确错误，不能声称安装成功。
- 使用进程内互斥/`Mutex` 或等价 `LaunchCoordinator`，防止两个启动请求同时安装；第二个请求返回稳定的 busy 错误。仍需在安装前和 spawn 前各做一次进程检查，防止用户在检查后手动启动 CS2。
- 自动安装只管理本项目 ZIP 所有的文件。不要覆盖用户未知文件，不要删除 CS2 核心文件，不要在游戏已运行时尝试修复。

### 5.3 前端状态

- `usePanelStore.launch` 继续是唯一启动 IPC 封装；清空旧错误、保留 `mutationKey = 'launch'` 直到 command 完成，并返回扩展后的 `LaunchResult`。
- `useCs2LaunchExperience` 的请求阶段文案可显示“正在检查 BOT 插件”；收到 `pluginAction=installed` 可在 Toast 或状态文案短暂显示“已更新 BOT 插件”。不把版本检查暴露成额外按钮，也不让用户在特效期间重复启动。
- 自动安装失败沿用现有错误 Toast，30 秒、进程探测、任意点击退出特效的语义不变。退出特效不会取消后端安装 promise，也不会触发第二次启动。
- `InstallView.vue`、`useCs2Store.install`、`install_bot_package` IPC 不增加版本检查、自动短路或新禁用条件；手动安装仍按原按钮点击执行。

## 6. 文件级改动清单

### 必改

- `src/components/AppTitlebar.vue`、`src/components/AppShell.vue`、`src/styles/main.css`。
- 新增 `src/assets/app-icon.png`，并记录与 `src-tauri/icons/icon.png` 的摘要关系。
- `src-tauri/src/commands/panel.rs`、`src-tauri/src/services/panel.rs`、`src-tauri/src/services/cs2.rs`、`src-tauri/src/models/cs2.rs`。
- `src-tauri/resources/CS2BotImprover.zip` 及 ZIP entry/hash fixture。
- `src-tauri/Cargo.toml`/`Cargo.lock`（仅在采用 `semver` 时增加固定依赖）。
- `src/stores/panel.ts`、`src/services/tauri/panel.ts`、`src/features/panel/types.ts`、`src/composables/useCs2LaunchExperience.ts`（仅扩展结果/状态，不改变特效退出条件）。
- `README.md`、`NOTICE.md` 或资源说明中关于包内容的准确描述。

### 新增测试/脚本

- `scripts/generate-plugin-manifest.ps1` 或等价可复现脚本。
- Rust 临时目录 fixture：版本相同、更高测试版、较低版、缺失标记、坏 JSON、payload 篡改、旧目录回滚。
- `tests/app-brand.spec.ts`、`tests/bot-plugin-gate.spec.ts`，以及 `src-tauri` 启动服务单元测试。

### 明确不改

- `src/views/InstallView.vue` 手动安装按钮的逻辑和用户文案。
- 在线模式启动逻辑、Panel 配置项、NadeSystem 投掷策略、已完成的 `[NadeAudit]` 移除结果、目录推荐扫描停止条件。
- 官方 `Panel v1.4.2.exe`、其大小和 SHA256。

## 7. 自动化测试与验收

### 7.1 版本和安装单元测试

- 当前版本 `0.5.3` + 已安装 `0.5.3`、`0.5.3-test.1`、`0.5.4-beta.1`：通过且不安装。
- 当前版本 `0.5.3` + 已安装 `0.5.2`、缺失、坏 JSON、payload 被改：自动安装当前包。
- 已安装有效 `0.6.0-test`：通过且绝不降级。
- 更高版本但 payload 无法验证：不静默覆盖，返回 `[BOT_PLUGIN_HIGHER_VERSION_UNTRUSTED]`。
- 安装中途模拟写入/摘要/重命名失败：恢复原文件，manifest 不留下半成品，CS2 不启动。
- 两个并发 BOT 启动请求：只有一个进入安装/启动，另一个得到 busy 错误。

### 7.2 启动链测试

- BOT + 当前插件：调用顺序为检查 -> `set_mode` -> Steam spawn，不调用安装。
- BOT + 低版本/缺失插件：顺序为检查 -> 安装 -> 再检查 -> `set_mode` -> Steam spawn。
- Online：不调用版本读取和安装。
- CS2 已运行、Steam 未找到、ZIP 摘要错误、安装失败：均不 spawn Steam，`panel.lastError` 有稳定错误码。
- 扩展 `LaunchResult` 不破坏现有 `options/insecure` 断言；启动特效仍在 command 返回前显示，30 秒/点击/进程退出条件不变。

### 7.3 UI 验收

- 标题标识节点 `.titlebar-brand .titlebar-mark` 渲染正式程序图标，不再渲染文字 `CS2`。
- 侧栏只显示 `0.5.3`，不存在 `Panel 0.5.3`。
- 720x620、960x700 的 light/dark 截图无重叠；标题栏仍可拖动，窗口控制可用。
- “安装与诊断”页手动安装按钮仍直接执行原 `install_bot_package`，不会因为已安装更高版本而变成灰色或跳过。

### 7.4 用户实机步骤

1. 在已安装 0.5.2 插件的 CS2 目录备份游戏目录和 `runtime.log`。
2. 安装本次 0.5.3 包，确认 BOT 模式首次启动前会自动安装/更新插件，然后正常拉起 CS2。
3. 退出 CS2 后放入一个有效的更高测试版 manifest（如 `0.6.0-test`）及匹配 payload，再点击 BOT 启动；确认不会被 0.5.3 降级覆盖。
4. 篡改 DLL 或 manifest 摘要后点击 BOT 启动；确认显示错误、游戏不启动，且恢复后可再次启动。
5. 切换在线模式启动；确认不触发插件自动安装。
6. 打开“安装与诊断”页手动点击安装；确认按钮仍按原逻辑覆盖安装，不依赖版本门禁。

## 8. 构建、资源和发布

按顺序执行：

```powershell
dotnet run --project .\third_party\CS2-Bot-Improver-v1.4.2\nades-per-bot-round-limit\tests\NadePacingPolicy.Tests.csproj -c Release
npm run verify
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

资源验证必须报告：

- `CS2AS05.plugin.json` 内容、manifest 生成输入和 SHA256。
- ZIP 全部 entry 的 `{name,size,sha256}` 差异；除新增 manifest 和预期定制 DLL 外不得出现解释不清的变化。
- 正式图标源文件与 `src/assets/app-icon.png` 的 SHA256/尺寸。
- 0.5.3 NSIS 安装包绝对路径、字节数、SHA256、生成时间。

版本和 Git：

1. `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 继续一致为 `0.5.3`；不要全局替换版本字符串。
2. 当前 `main` 已有未推送的 0.5.3 提交；实施完成后将新增改动 amend 进该提交，保持 `git log origin/main..HEAD --oneline` 只有一个完整 0.5.3 提交。
3. 推送前显示 `git status -sb` 和 `git log origin/main..HEAD --oneline`，只执行一次普通 `git push origin main`，禁止 force push。

## 9. 回退

- 图标异常：恢复原 `.titlebar-mark` 文字和 CSS，不影响启动链。
- 版本门禁异常：临时关闭 BOT 自动安装调用，但保留原启动命令；不要删除已安装插件或改动手动安装按钮。
- ZIP/manifest 校验失败：恢复原 ZIP 和 DLL 备份，不发布；不得用不完整包启动 CS2。
- 自动安装事务失败：只恢复本次事务涉及的本项目文件，保留用户配置和未知第三方文件。
- 发现高版本插件：没有有效 payload 证明时默认阻止自动降级，并引导用户在“安装与诊断”页明确处理。

## 10. 最终交付报告模板

```text
结果：0.5.3 是否完成
图标与版本：标题标识节点、侧栏仅 0.5.3 的截图/断言
插件门禁：当前/低版本/高版本测试、缺失和篡改场景结果
启动链：BOT 自动安装与在线模式不检查的调用顺序
手动安装：安装与诊断按钮未改变的验证
测试：npm verify、Rust fmt/clippy/test、插件 fixture
资源：manifest、ZIP entry 差异、图标摘要
安装包：绝对路径、大小、SHA256、时间
Git：0.5.3 amend 提交和一次 push 结果
限制：需要用户执行的真实 CS2 启动与高版本插件验收
```
