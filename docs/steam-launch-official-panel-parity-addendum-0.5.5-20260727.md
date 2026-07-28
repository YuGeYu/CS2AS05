# 0.5.5 Steam 启动：原版 Panel 行为对齐补充方案

## 1. 文档定位

本文只处理一个问题：CS2 目录已经识别，但 CS2 人机增强助手启动时报 `[STEAM_NOT_FOUND]`，而同一用户使用原版 `Panel v1.4.3.exe` 可以正常启动。

本文是 [`version-0.5.5-steam-launch-and-update-entry-plan-20260727.md`](./version-0.5.5-steam-launch-and-update-entry-plan-20260727.md) 的窄范围补充。它只修订该文档的 Steam 根因判断、实现优先级和实机验收；版本、更新检查、插件 marker、签名和发布章节仍按原方案执行。

制定本文时，`E:\CS2AS05` 已出现另一执行过程产生的 0.5.5 业务改动。不要回退、覆盖或清理这些改动；先审阅当前 diff，再按本文验证和增量修正。

## 2. 新增的最高优先级证据

### 2.1 用户真实运行结果

用户在出现问题的同一环境中反馈：

```text
助手启动：失败，[STEAM_NOT_FOUND] 未找到 steam.exe
安装与诊断 -> 打开原版 Panel -> 原版启动：成功
```

这证明 CS2、Steam 会话和原版 Panel 启动链在该机器上可用。问题位于助手与原版 Panel 的启动实现差异，而不是 CS2 安装损坏或用户不会操作。

### 2.2 原版目标与恢复工程

目标原版：

```text
E:\破解\CS2BotImprover (5)\Panel v1.4.3.exe
size   5,844,480 bytes
sha256 3FD93DC7AF2702C50B9A7E4FCF1BB11387B107ABC863EE8A3067255022408CCD
```

行为等价恢复工程：

```text
E:\破解\Panel-v1.4.3-recovered\source
E:\破解\Panel-v1.4.3-recovered\README.md
E:\破解\Panel-v1.4.3-recovered\verify-recovery.ps1
```

恢复源码 ZIP：

```text
E:\破解\Panel-v1.4.3-recovered\Panel-v1.4.3-recovered-source.zip
size   194,099 bytes
sha256 BA5CA1388D3C58038EA765744869BCDED39D368C521C3F39007316F9076C52A9
```

`source/src-tauri/src/lib.rs::find_steam_executable()` 的决定性行为是：

1. 使用 `sysinfo` 刷新进程。
2. 查找名称大小写不敏感等于 `steam.exe` 的运行中进程。
3. 读取该进程的 `process.exe()` 完整路径。
4. 路径存在且是文件时立即使用。
5. 只有运行进程路径不可用时，才检查 `ProgramFiles(x86)`、`ProgramFiles` 和固定标准目录。

启动仍是原生进程调用：

```text
Command::new(steam).args(arguments).spawn()
online: -applaunch 730
bots:   -applaunch 730 -insecure -console -condebug
```

目标 PE 中可直接确认 `steam_launch.rs` 和 `steam.exe` 字符串；最早的公开兼容后端快照 `local-arena-5dc04e9-backend-snapshot` 也包含相同的“运行进程 EXE 优先”实现。恢复脚本本身主要验证目标 hash、20 个 IPC、8 项 Bot Items 和 5 档 Nades，并不单独证明每一行 Steam 算法；因此最终结论由“用户真实成功 + 目标字符串 + 恢复实现 + 兼容后端来源”共同支持，而不是只相信重建源码注释。

## 3. 修订后的根因

本项目 0.5.4 的 `src-tauri/src/services/panel.rs::find_steam()` 只检查：

```text
%PROGRAMFILES(X86)%\Steam\steam.exe
%PROGRAMFILES%\Steam\steam.exe
```

它没有读取运行中 `steam.exe` 的真实路径。

用户的 CS2 位于：

```text
E:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive
```

原版 Panel 能启动，最符合现有证据的解释是：Steam 当时已经运行，原版通过 `process.exe()` 得到了实际自定义安装路径；本项目忽略进程路径后只查 Program Files，因此报错。

这比“必须依赖注册表”更窄、更有运行证据。注册表、多盘常见目录和 CS2 root hint 仍可作为增强回退，但它们不是本次用户差异的首要修复依据。

## 4. 最小修复契约

### 4.1 必须对齐的原版行为

实际执行 AI 必须保证 Steam 候选顺序的第一类来源是：

```rust
running processes
  -> process.name().eq_ignore_ascii_case("steam.exe")
  -> process.exe()
  -> path.is_file()
```

当 Steam 正在从自定义目录运行时，必须直接使用该完整路径，不得被 Program Files、注册表陈旧值、附加 Steam 库路径或其他常见目录抢占。

随后才允许回退：

1. Steam 注册表中的 `SteamExe`。
2. `SteamPath` / `InstallPath` 加 `steam.exe`。
3. 从 CS2 root hint 反推的 Steam 库根候选。
4. 多盘标准/常见 Steam 路径。

这些增强回退可以保留，但必须满足：候选去重、文件名严格为 `steam.exe`、`is_file()` 为真，不执行任意注册表命令行字符串。

### 4.2 不复制原版的不安全顺序

恢复源码中的原版 `launch_cs2()` 在 `find_steam_executable()` 前调用 `apply_launch_mode()`，即找不到 Steam 时可能已经修改 `gameinfo.gi`。

下游助手不应复制这个顺序。保持 0.5.5 原方案的安全增强：

```text
校验 mode
-> 检查 CS2 未运行
-> 规范化并验证 CS2 root
-> 解析 Steam 可执行文件
-> 只有解析成功后才初始化配置/安装插件/切换模式
-> 再检查 CS2
-> Command::new(steam) 启动
```

这属于在原版可用行为之上的安全改进，不是兼容性偏离。

### 4.3 不采用的方案

- 不直接启动 `game\bin\win64\cs2.exe`。
- 不使用 `steam://`、`window.open`、浏览器或网页跳转。
- 不要求用户先手填 Steam 路径。
- 不全盘递归搜索任意 `steam.exe`。
- 不把“原版能启动”解释为应该直接嵌入或常驻运行原版 Panel。

## 5. 对当前 0.5.5 工作树的审阅要求

制定本文时，当前未提交实现已经出现：

- `src-tauri/src/services/cs2_discovery.rs::find_steam_executable`
- `running_steam_executables()` 位于候选列表首位
- `src-tauri/src/services/panel.rs` 在配置/插件/模式写入前解析 Steam
- Steam 解析顺序的 source-order 测试

这说明实现方向与本文一致。实际执行 AI 不应因为本文重新实现第二套 resolver，只需确认以下缺口：

1. `running_steam_executables()` 确实返回进程的完整 EXE，而不是父目录。
2. 进程候选确实排在注册表、root hint 和 common paths 前。
3. 找到进程路径后不会继续选择后续陈旧候选。
4. `process.exe()` 返回 `None` 或权限不可读时能安全继续回退。
5. 最终参数保持 online/BOT 两套原契约。
6. 找不到 Steam 时，游戏目录在字节级保持不变。

如果六项都满足，则本问题不需要扩大重构范围。

## 6. 自动化验证

### 6.1 单元测试

现有候选选择测试还需要明确证明“来源优先级”，而不只是证明“第一个有效路径被选中”。建议把候选聚合拆成可注入来源的纯函数，至少覆盖：

1. `running`、`registry`、`root hint`、`common` 四类候选都有效时，选择 `running`。
2. `running` 路径有效但注册表指向另一个真实 `steam.exe` 时，仍选择 `running`。
3. `running process.exe() == None` 时选择注册表/标准回退。
4. 运行进程路径存在但文件名不是 `steam.exe` 时拒绝。
5. 大小写和斜杠变体去重，不改变来源优先级。

不要让单元测试依赖开发机真实 Steam 是否运行。进程枚举只做薄适配，排序和选择逻辑使用注入候选测试。

### 6.2 启动事务测试

保留或增强顺序测试，证明 `find_steam_executable` 早于：

```text
initialize_panel_defaults_at
ensure_current_bot_plugin
set_mode
```

增加失败前后字节断言：

- `game/csgo/gameinfo.gi`
- `game/csgo/cfg/cs2as05-panel-state.json`
- `game/csgo/addons/counterstrikesharp/plugins/NadeSystem/CS2AS05.plugin.json`

如果难以在 Tauri `AppHandle` 单元测试中模拟完整启动，应把“解析启动器”保留为纯前置函数，并用隔离目录集成测试补足；不能只靠源码字符串顺序作为最终证据。

### 6.3 常规命令

```powershell
git diff --check
npm run verify
cargo fmt --manifest-path .\src-tauri\Cargo.toml -- --check
cargo clippy --manifest-path .\src-tauri\Cargo.toml --all-targets --all-features -- -D warnings
cargo test --manifest-path .\src-tauri\Cargo.toml
```

## 7. 用户机器上的决定性复现

真实验收只需要围绕本问题，不需要重复三回合 Nade、Bot Items 或刀具验收。

### 7.1 记录原版成功时的 Steam 路径

让用户在“原版 Panel 可以启动”的同一状态下运行：

```powershell
Get-CimInstance Win32_Process -Filter "Name='steam.exe'" |
  Select-Object ProcessId, ExecutablePath
```

只记录 `ExecutablePath` 是否位于非 Program Files 目录；公开报告可隐去用户名等个人路径段。

### 7.2 对照测试 0.5.5

1. Steam 保持运行，确认其进程路径与上一步一致。
2. 退出 CS2，只保留 Steam。
3. 在 0.5.5 选择原截图中的 CS2 目录。
4. 先在线模式启动，确认 Steam/CS2 打开。
5. 退出 CS2，再以 BOT 模式启动，确认参数包含 `-insecure -console -condebug`。
6. 核对助手运行日志采用的 Steam 路径与 `ExecutablePath` 一致。
7. 两次启动均不得出现 `[STEAM_NOT_FOUND]`。

### 7.3 回退场景

用户方便时再做，不作为本次根因修复的首要门槛：

- Steam 关闭后，注册表/标准目录能否找到自定义客户端。
- 标准 Program Files Steam 是否保持兼容。
- Steam 未安装时是否仍给出明确错误，且游戏目录没有变化。

## 8. 完成标准

只有同时满足以下条件，才能报告该用户问题已修复：

1. 自动化证明运行中 Steam 进程路径具有最高优先级。
2. 找不到 Steam 时不会先写游戏目录。
3. 用户问题机器上，0.5.5 日志采用的路径与运行中 `steam.exe` 的 `ExecutablePath` 一致。
4. 同一环境下在线和 BOT 模式均成功启动。
5. 启动参数与原版 Panel 契约一致。

若只有单元测试通过但用户尚未测试，只能称为“0.5.5 Steam 启动修复候选”。

## 9. 本问题最终报告模板

```text
结论：候选通过 / 用户实机通过
原版 Panel：目标 SHA256、启动成功证据
Steam 进程：路径类别（自定义/Program Files），隐去个人路径
助手 resolver：running -> registry -> root hint -> common
实际采用来源：running process.exe / fallback
参数：online / bots
失败前无写入：gameinfo、Panel state、plugin marker
自动化：resolver、事务顺序、npm、fmt、clippy、cargo test
用户实机：在线启动、BOT 启动、日志路径一致性
剩余限制：未测试的 Steam 关闭回退场景
```
