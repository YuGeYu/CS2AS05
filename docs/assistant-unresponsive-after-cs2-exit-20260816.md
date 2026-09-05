# CS2 退出后助手无响应事件核查

日期：2026-08-16

## 当前结论

状态：`partial / 未能独立复现助手 UI 无响应，因果关系未定`

用户明确确认 CS2 是主动关闭的，助手无响应是 CS2 正常退出之后观察到的现象。该现场事实优先于单条系统事件记录；本报告不把 CS2 归类为崩溃，也不把 WER 事件作为助手卡死的根因。

当前能确认的技术边界是：助手自身没有崩溃或 Application Hang 证据；Seelen UI 随后重启，说明桌面壳曾发生重启，但仅凭时间顺序不能证明它是唯一根因。助手代码存在一个需要重点验证的退出竞态：CS2 状态从 running 变为 stopped 后，Panel 进程轮询、同步 `initialize_panel_defaults`/`get_panel_snapshot` 调用，以及 Demo watcher 触发的扫描可能在同一时间窗口叠加。当前尚无耗时日志证明这些调用已经长期 pending，因此不能把它写成已证实的助手内部死锁。

## 证据

- Windows WER 中存在一条 `2026-08-16 17:52:06` 的 `cs2.exe` / `counterstrikesharp.dll` 访问冲突记录；它是系统旁证，不等于用户这次主动关闭 CS2 的退出方式，也不能证明与助手无响应存在因果关系。
- 对应 WER 文件：`C:\ProgramData\Microsoft\Windows\WER\ReportArchive\AppCrash_cs2.exe_7efca4bd6b91ab94795ad72da25f3a91289d91_3fc63fc1_2b8d2dac-1524-4a1f-bd1c-5fd73fb57cd6\Report.wer`。
- CounterStrikeSharp 日志出现 `CBaseModelEntity:m_CBodyComponent is not networked, but SetStateChanged was called`；这是插件侧警告，不能单独解释助手窗口无响应。
- `Seelen UI` 在 `17:56:44` 启动；只能证明桌面壳随后重启，不能证明它或助手是根因。
- 本轮没有发现 `ai_pc_fac.exe`、`CS2BotImproverAssistant.exe` 或 WebView 的 Application Error/Application Hang 事件。
- 当前助手空载进程检查为 `Responding=True`，窗口可正常渲染；桌面自动化点击复核被物理 Escape 中止，没有继续干扰用户桌面。

## 运行版本边界（已按 C 盘日志修正）

用户本次实际运行的是开发中的 `0.5.8`。工作树对应的开发可执行文件为：

`E:\CS2AS05\src-tauri\target\debug\ai_pc_fac.exe`

该文件资源版本和产品版本均为 `0.5.8`，最后构建时间为 `2026-08-16 16:44:21`。同一工作树的 release 二进制和 NSIS 安装器也均为 `0.5.8`，但本次不能仅凭应用注册表路径断言用户运行的是旧验收包。

此前通过应用注册表看到的另一路径是：

`E:\CS2AS05\workspace\player-acceptance\install-smoke-0.5.7-v1.8.2-final\CS2BotImproverAssistant.exe`

该路径属于旧验收目录，不能覆盖用户明确说明的开发态运行事实。

## C 盘日志核对

- `C:\Users\GOPtZ\AppData\Local\CS2人机增强助手\logs\runtime.log` 在 `2026-08-16 17:36:07` 写入 Steam 使用记录，并在 `18:04:42` 继续写入“扫描到 1 个 CS2 候选目录”和目录检查记录。
- 该日志没有记录助手崩溃、Application Hang、Tauri command error 或 WebView 崩溃；日志在事件后能够继续写入，至少说明日志写入线程/进程曾恢复或助手被重新启动。
- `C:\Users\GOPtZ\AppData\Local\com.aipc.cs2botimprover\logs\CS2人机增强助手.log` 是较早的 preflight/启动日志，最后记录为 `16:44:32`，没有覆盖本次退出窗口。
- Windows Application 日志在 `16:00` 之后没有筛出 `ai_pc_fac`、`CS2BotImproverAssistant` 或助手 Application Hang 事件。
- 因此 C 盘证据不能证明“助手进程持续卡死”；同时也不能证明窗口在用户观察的那几分钟内没有短暂失去响应，因为当前运行日志没有 UI 交互和命令耗时埋点。

## 代码风险点（待复现验证）

助手在 `AppShell.vue` 中每 2 秒刷新 Panel，在 `useCs2ProcessPolling.ts` 中每 10 秒检查 `cs2.exe`；CS2 退出后会触发 Panel 状态从 running 到 stopped 的刷新路径。Panel store 只对自身刷新做 Promise 去重，Tauri 的 `initialize_panel_defaults` 和 `get_panel_snapshot` 仍是同步 command，且没有显式超时。与此同时，Demo watcher 会在文件变化后延迟调用 `demo.scan()`。这构成“退出瞬间并发 I/O/命令拥塞”的可疑路径，但目前没有命令开始/结束耗时证据，不能称为已证实死锁或必然卡死。

## 本轮未做

- 没有修改代码。
- 没有重启 CS2、删除 Demo、修改插件或覆盖用户游戏目录。
- 没有把“偶发”或“必然”作为最终结论；当前证据确认用户主动关闭 CS2、随后观察到助手无响应，且用户运行的是开发态 `0.5.8`。C 盘日志未发现助手崩溃/挂起，且 `18:04:42` 仍有运行日志写入；助手内部命令竞态仍待 UI 响应采样和耗时证据确认。
