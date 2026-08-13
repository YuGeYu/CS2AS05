# 皮肤工坊真实玩家双场景验收方案（修订版）

日期：2026-08-11。本文替代同名旧方案。验收对象必须是 **全新构建并安装的 Windows NSIS 安装程序**，不是源码目录中的裸 EXE，也不是手工移动文件后的开发环境。

## 1. 修订原因与当前事实

旧方案错误地把 `src-tauri\target\release\CS2BotImproverAssistant.exe` 和手工移动 PlayerSkinMod 当作用户主流程。那只能用于工程隔离诊断，不能代表普通玩家从下载安装助手、打开功能、点击部署到进入游戏的体验。

当前事实：

- 2026-08-11 只执行了 `npm run build:desktop`，该脚本实际是 `tauri build --no-bundle`，只生成裸 EXE。
- `src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.6_x64-setup.exe` 是皮肤工坊落地前的旧安装器，SHA-256 `590AC0D6...`，严禁用于本轮验收。
- 当前没有可交给玩家测试的新版安装程序。下一位实际执行 AI 必须先完成第 3 节，才能通知用户开始测试。
- 安装器只把 PlayerSkinMod 固定资源装进助手自身安装目录；只有玩家在“皮肤工坊”点击“部署 / 更新插件”后，插件才会写入 CS2。因此同一个新安装程序天然支持两个玩家场景：先不点击部署测试原功能，再点击部署测试皮肤功能。

## 2. 玩家视角的唯一主流程

```text
下载新的 NSIS 安装程序
  -> 双击安装/覆盖升级
  -> 从开始菜单启动“CS2人机增强助手”
  -> 选择或自动识别 CS2
  -> 不安装玩家皮肤，验证全部原有功能
  -> 在皮肤工坊点击“部署 / 更新插件”
  -> 通过助手配置并应用装备
  -> 从助手启动 -insecure 本地 BOT 对局
  -> 验证皮肤功能和原功能共存
```

玩家不执行 PowerShell，不进入项目目录，不复制/移动 DLL，不编辑 JSON，不手动访问 `addons`。文件/hash/log 检查由执行 AI 在玩家操作前后完成，只作为幕后证据。

## 3. Gate 0：执行 AI 必须先交付全新安装程序

### 3.1 独立候选版本

当前正式/旧候选已经占用 `0.5.6`，且 NSIS 输出目录含多个历史文件。为了让玩家不会装错，执行 AI 应把本轮测试候选统一升为 `0.5.7`（只作为未发布测试候选）：

- `package.json::version`
- `src-tauri/Cargo.toml` 中本 package 的 version
- `src-tauri/tauri.conf.json::version`
- 让 Cargo 更新 `Cargo.lock` 中本 package 条目；禁止全局替换依赖版本。

UI 标题/关于页显示的版本必须与 `0.5.7` 一致。不得覆盖、删除或冒充旧 `0.5.6` 安装器。

### 3.2 构建前检查

执行 AI 先确认：

- `src-tauri/resources/skin-forge/PlayerSkinMod/PlayerSkinMod.dll` 存在且 SHA-256 为 `47BF3733D3091D3EAB9E4B86052CBF77A002CFEBAACDEB8D8C43136FF2ADFDEA`。
- `tauri.conf.json::bundle.resources` 包含 `resources/skin-forge`。
- typecheck、lint、Vitest、Web build、Rust fmt/check/test/clippy 全通过。
- 没有改变现有 CounterStrikeSharp、Bot Improver 资源和用户 Demo。

然后执行：

```powershell
Set-Location E:\CS2AS05
npm run bundle:desktop
```

最多构建 5 次。网络/签名慢时等待；失败就保留完整错误，不得把裸 EXE交给用户代替。

### 3.3 安装器交付路径

新构建成功后的原始路径必须是：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.7_x64-setup.exe
```

执行 AI 再把安装器、SHA-256 文本和测试说明复制到独立交付目录，例如：

```text
E:\CS2AS05\workspace\player-acceptance\skin-forge-0.5.7-20260811\
  CS2人机增强助手_0.5.7_x64-setup.exe
  CS2人机增强助手_0.5.7_x64-setup.exe.sha256
  TESTING.md
```

交给用户前必须明确报告：绝对路径、字节数、SHA-256、生成时间、文件版本、Authenticode 状态、Tauri updater `.sig` 是否存在。未签 Windows Authenticode 可以作为本地测试候选，但必须明确提示；没有对应 `.sig` 时不得发布到生产更新源。

### 3.4 安装器资源证明

执行 AI 必须在隔离目录实际安装一次候选，确认安装目录内可解析 PlayerSkinMod 资源，并从安装后的程序调用“检查插件”不会出现 `[PLUGIN_RESOURCE_MISSING]`。只看 NSIS 构建成功不够。

若新安装器不存在、仍叫 `0.5.6`、生成时间早于皮肤工坊实现，或内置资源缺失，立即停止，不能让用户测试。

## 4. Gate 1：执行 AI 准备真实玩家起点

当前开发机的 CS2 目录已经由工程阶段部署过 PlayerSkinMod。为了模拟“普通玩家第一次安装助手、尚未点击玩家皮肤部署”，执行 AI 必须在用户开始前准备一次可恢复的干净起点：

1. 确认 CS2 和助手全部退出。
2. 完整备份当前 `PlayerSkinMod` 目录和 loadout 到验收证据目录。
3. 从 CS2 中移除该工程阶段插件，使助手检查结果为“未部署”。
4. 不触碰 CounterStrikeSharp、CS2BotImprover、RayTrace、用户 Demo 和 Steam 文件。
5. 记录操作前后的目录清单/hash，并准备一键回滚。

这是执行 AI 的测试夹具准备，不是玩家步骤，也不算产品验收。用户拿到安装程序之后全程只操作安装器、助手和 CS2。

若执行 AI 无法安全准备干净起点，应该提供 Windows 沙盒/独立测试账户/另一套 CS2 副本，而不是要求玩家手工移动 DLL。

## 5. 玩家阶段一：安装助手，暂不安装玩家皮肤

### 5.1 安装/覆盖升级

用户只双击第 3.3 节交付目录中的 `CS2人机增强助手_0.5.7_x64-setup.exe`：

1. 完成 NSIS 安装；若系统显示未知发布者，截图后确认这是明确标记的本地未签候选再继续。
2. 从安装完成页或开始菜单启动助手，不运行源码目录 EXE。
3. 在“关于/版本”确认显示 `0.5.7`。
4. 如果是覆盖旧版，确认旧的 CS2 根目录、用户设置和 Demo 记录仍在；如果是干净安装，按界面选择 CS2 根目录。
5. 安装器和程序启动无白屏、崩溃、缺 WebView2、缺资源或权限错误。

执行 AI在后台核对实际进程路径来自安装目录，而不是 `E:\CS2AS05\src-tauri\target\release`。

### 5.2 不部署 PlayerSkinMod 的界面验收

用户进入“皮肤工坊”，只查看状态，不点击“部署 / 更新插件”和“应用装备”。期望：

- 页面明确、友好地显示“玩家皮肤尚未部署”。
- 不应只显示含糊的“待检查”。
- 告诉玩家需要先部署、CS2 必须退出、仅用于 `-insecure` 离线/本地环境。
- 不因为插件缺失阻塞概览、人机预设、Bot 物品、刀具、命令、对局复盘和安装诊断。

如果普通玩家无法判断插件究竟有没有安装，记为 UX 失败并截图。

### 5.3 原有功能完整回归

用户依次通过界面操作：

| 页面/流程 | 玩家操作 | 通过标准 |
| --- | --- | --- |
| 概览 | 查看环境，选择 BOT 模式 | 状态正确，无皮肤插件错误 |
| 人机预设 | 选择并应用一个预设 | 操作成功、反馈清楚 |
| Bot 物品 | 查看/调整一个可恢复配置 | 可保存、无异常 |
| 刀具 | 查看并选择原有刀具配置 | 不依赖 PlayerSkinMod 也可用 |
| 命令 | 复制一条命令 | 有明确复制反馈 |
| 对局复盘 | 打开录像库 | 原记录不丢失、不崩溃 |
| 安装与诊断 | 查看 CounterStrikeSharp/Bot Improver | 现有环境显示正常 |

然后必须从助手“概览”启动 `-insecure` 本地 BOT 对局，至少完成 2 回合并在 CT/T 各重生一次。确认 Bot 出生、购买、移动、射击、回合推进和正常退出均正常。此阶段不应加载 PlayerSkinMod。

阶段一结论：`未安装玩家皮肤时原有功能 PASS/FAIL`。

## 6. 玩家阶段二：只通过助手安装玩家皮肤

### 6.1 玩家点击部署

退出 CS2，回到安装版助手：

1. 打开“皮肤工坊”。
2. 点击“部署 / 更新插件”。
3. 观察按钮忙碌、成功/失败反馈和最终插件状态。
4. 玩家不得手工选择 DLL、复制文件或安装 CounterStrikeSharp。

通过标准：助手自动定位当前 CS2；只部署固定 PlayerSkinMod 三项资源；不覆盖 CounterStrikeSharp；显示 PlayerSkinMod `1.8.0` 和“已就绪”；失败信息能告诉玩家如何恢复。

执行 AI在后台核对目标文件/hash，但这不是玩家要做的事情。

### 6.2 玩家配置装备

用户完全通过“皮肤工坊”完成：

- 切换 CT/T。
- 选择武器和皮肤，而不是要求用户查询 paintKit/defindex。
- 设置磨损、种子、命名标签和 StatTrak。
- 分别选择 CT/T 刀具、手套和角色。
- 选择音乐盒。
- 添加/编辑/删除贴纸。
- 添加/编辑/删除挂件。
- 切换自定义/随机模式。
- 点击“应用装备”。

这里有当前已知阻断：源码界面目前主要是数字输入，只提供 AK47/M4A1S，贴纸只能增加空槽，没有贴纸参数和挂件控件，也没有完整的上游图文选择目录。普通玩家不应该查 defindex/paintKit。

因此在构建新安装器之前，执行 AI 必须先补齐这些 UI/功能；否则即使安装器能装、DLL 能加载，玩家阶段二也只能判 `PARTIAL/FAIL`。禁止让用户手改 JSON 绕过界面。

UI 补齐时使用 UI/UX Pro Max 与 Emil Skills 协同：延续现有中文桌面风格，使用可搜索图文选择器、分队切换、明确字段说明、可见 focus、键盘可达、错误就地反馈、窄窗口不溢出，动效克制且支持 reduced motion。

### 6.3 游戏内验证

用户从安装版助手启动 `-insecure` 本地 BOT 对局，至少 CT 2 回合、T 2 回合，验证：

- CT/T 武器皮肤不串队。
- 刀型/刀皮正确，动画不崩。
- 手套模型/涂装正确，无明显 UV 错位。
- CT/T 角色正确。
- 音乐盒生效。
- 贴纸、挂件、命名、StatTrak 正确。
- 游戏运行时从助手修改并应用，重生后 watcher 热重载生效。
- `skin_menu`、`skin_random`、`skin_reset` 正常。
- 原有 Bot Improver 的出生、购买、移动、射击、回合逻辑仍正常。

玩家只需要截图或录屏，不需要读取日志。执行 AI根据测试起止时间回读 CounterStrikeSharp 新日志和 loadout，形成“玩家界面操作 + JSON 回读 + 新日志 + 游戏内可见”证据。

阶段二分别给出：`部署 PASS/FAIL`、`皮肤功能 PASS/PARTIAL/FAIL`、`与原功能共存 PASS/FAIL`。

## 7. 玩家阶段三：安装程序生命周期

为了验证交付物而不是只验证某次启动，还要通过 Windows 正常流程完成：

1. 关闭助手和 CS2。
2. 再次运行同一 `0.5.7` 安装器进行覆盖安装/修复安装。
3. 启动助手，确认设置、Demo 和皮肤配置没有意外丢失，内置插件资源仍可部署/检查。
4. 从 Windows“已安装的应用”卸载助手，确认卸载程序可运行。
5. 验证卸载助手不会删除 CS2、Demo、CounterStrikeSharp 或已经部署到 CS2 的玩家皮肤插件；若产品设计希望同步卸载皮肤插件，必须由助手提供明确的独立按钮和二次确认，不能静默删除。
6. 重新安装候选，确认可以正常启动和识别环境。

## 8. 证据与最终判定

执行 AI建立：

```text
E:\CS2AS05\workspace\player-acceptance\skin-forge-0.5.7-20260811\evidence\
  installer\
  stage-A-no-skin\
  stage-B-with-skin\
  lifecycle\
  report.md
```

最终必须分别报告：

```text
1. 新 NSIS 安装程序：PASS / FAIL
2. 安装/覆盖升级/启动：PASS / FAIL
3. 未部署 PlayerSkinMod 时原有功能：PASS / FAIL
4. 助手内一键部署 PlayerSkinMod：PASS / FAIL
5. 助手内完整皮肤配置体验：PASS / PARTIAL / FAIL
6. 真实游戏皮肤效果：PASS / PARTIAL / FAIL
7. 与 Bot Improver 共存：PASS / FAIL
8. 卸载/重装生命周期：PASS / FAIL
```

只有 1-8 全部 PASS，且安装器路径/hash、安装后进程路径、界面截图、JSON 回读、新日志和游戏画面齐全，才能称为“实际玩家完整操作验证完成”。

## 9. 给下一位实际执行 AI 的明确任务

下一位 AI 不能再次只给裸 EXE 或让用户移动文件。必须依次完成：

1. 补齐普通玩家可用的完整皮肤选择、贴纸和挂件 UI。
2. 统一升级 `0.5.7` 测试候选版本。
3. 跑自动化。
4. 执行 `npm run bundle:desktop`。
5. 把全新 NSIS 安装器复制到第 3.3 节的独立交付目录。
6. 报告安装器绝对路径、size、SHA-256、生成时间和签名状态。
7. 在用户开始前准备“CS2 尚未部署 PlayerSkinMod”的可恢复测试起点。
8. 用户从安装程序开始，按第 5-7 节只操作助手和 CS2。
9. 执行 AI回读磁盘与日志并写最终报告。

没有完成第 1-7 项时，不得把任务交给用户验收。
