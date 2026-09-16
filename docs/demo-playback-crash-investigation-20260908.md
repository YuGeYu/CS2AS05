# 自动录制 Demo 播放闪退调查（2026-09-08）

## 结论先行

### 最终处置决定（用户确认）

经进一步研究，现场 `counterstrikesharp.dll` 的 `0xc0000005` 主要发生在关闭 CS2 阶段，不能据此认定自动录制 Demo 已损坏。该问题约影响 1% 玩家，当前没有足够证据证明它是 Demo 播放闪退的根因；本次调查到此结束，暂不修复、暂不重做上游整包，并接受少量玩家继续遇到该低频问题。

本报告以下内容保留为调查证据和未来复查入口，不代表当前已确认存在 Demo 文件缺陷，也不构成 0.5.13 发布阻断项。

当前没有证据支持“必须基于上游新整包重做自动录制”。现有实现只是向 BOT/本地托管对局的两个受管 cfg 写入：

```cfg
tv_enable 1
tv_autorecord 1
```

这属于 CS2 原生 CSTV 自动录制，不是自定义 Demo 编码器。项目播放链路也只是通过 Steam 启动 CS2 的 `playdemo`。因此，Demo 播放闪退更可能来自以下两个方向：

1. CS2 在录制或播放时加载的 CounterStrikeSharp/插件组合发生原生访问冲突；
2. CS2 在换边、换图或退出阶段崩溃/中断写盘，留下可识别 header 但未完整收尾的 Demo。

上游整包只有在“同一个完整 Demo 在干净 CS2 可播放，但在当前插件整包中稳定崩溃，并能进一步定位到具体插件”这一证据成立后才值得重做。

## 已核对的代码事实

- `src-tauri/src/services/panel.rs::replace_demo_recording_block` 只维护 `tv_enable` 和 `tv_autorecord`，没有修改 Demo 二进制内容。
- `src-tauri/src/demo/playback.rs` 的主播放路径会检查 CS2 未运行、校验 `.dem`、必要时复制到 `game/csgo/replays/_cs2as_play_*.dem`，再以 `-applaunch 730 -insecure -novid +playdemo` 启动。
- `src-tauri/src/services/demo.rs::launch_demo_at_tick` 是另一条播放入口，使用 `-applaunch 730 +playdemo <路径> +demo_gototick <tick>`；两条路径参数和路径语义不完全一致，应在后续修复中统一。
- 解析器 panic 已在 Rust 侧隔离为 `DEMO_PARSER_PANIC`，这只能保护助手进程，不能阻止 CS2 客户端播放异常 Demo 时退出。
- 当前扫描/解析已等待 size、mtime 稳定，并在解析前后复核 fingerprint；但这仍不能证明 Demo 具备 CS2 播放所需的完整结束结构。

## 现场 Windows 证据

读取本机 Windows Application 日志（2026-09-05 至 2026-09-08）得到多次相同模式：

- 故障应用：`D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe`
- 故障模块：`...\game\csgo\addons\counterstrikesharp\bin\win64\counterstrikesharp.dll`
- 异常：`0xc0000005`（访问冲突）
- 错误偏移：`0x00000000001f7bf6`

可复现的时间对应关系包括：

| WER 崩溃时间 | 相邻自动 Demo | Demo 时间 | 观察 |
|---|---|---|---|
| 2026-09-06 19:03:16 | `auto-20260906-1902-de_inferno-advent.dem` | 19:03:11，2,052,953 bytes | 崩溃约 5 秒后留下文件 |
| 2026-09-06 19:05:34 | `auto-20260906-1904-de_inferno-advent.dem` | 19:05:29，5,450,598 bytes | 崩溃约 5 秒后留下文件 |
| 2026-09-07 15:45:41 | 同时段存在自动 Demo | 15:45 附近 | 同一故障模块和偏移 |
| 2026-09-07 15:49:55 | 同时段存在自动 Demo | 15:49 附近 | 同一故障模块和偏移 |

插件日志显示 CounterStrikeSharp 启动后加载 `BotAI`、`BotAimImprover`、`BotControllerImpl`、`BotRandomizer`、`BotState`、`MapRotation`、`NadeSystem` 等插件，并应用了 42 个 Bot AI 原生 patch。该证据把排查重点从“文件太小”转移到了“插件加载/原生 patch 与 CS2 生命周期或换边状态的交互”。

玩家只打几回合产生几百 KB 或几 MB Demo 是正常现象，不能用大小单独判定损坏。文件带 `PBDEMS2` header 也只证明格式头存在，不能证明完整收尾。

## 当前未被证明的部分

- 现有本机 Demo 是否能被当前 CS2 客户端稳定播放，尚未完成真实游戏内回放验收。
- WER 记录证明的是 CS2/CounterStrikeSharp 崩溃，尚不能单独证明崩溃发生在“播放”而不是“录制、换图或退出”。
- 尚无崩溃转储调用栈，不能仅凭固定偏移把责任归给某一个上层插件。
- vendored demoparser 能否解析某个短 Demo，与 CS2 客户端能否播放不是同一个成功标准。

## 推荐的隔离验证矩阵

由用户在隔离环境执行真实 CS2 测试，每组使用同一张地图、同样换边步骤、同样 Demo：

1. 干净官方 CS2，关闭自动录制：确认换边本身是否崩溃。
2. 干净官方 CS2，开启 `tv_enable/tv_autorecord`：确认原生录制是否崩溃。
3. 当前插件整包，关闭自动录制：确认插件/换边是否单独崩溃。
4. 当前插件整包，开启自动录制：比较崩溃率、Demo size/mtime 和 WER 模块。
5. 录制完成后，在不加载 BOT 插件的干净环境播放同一 Demo。
6. 再在当前插件环境播放同一 Demo。

每组至少重复 3 次，并记录：CS2 启动参数、实际 PID、换边回合、Demo 最终 size/mtime、是否出现 WER、故障模块/偏移、插件日志最后 30 秒。不要上传完整 Demo 或密钥；诊断包只保留 hash、size、mtime、错误摘要。

## 修复优先级

### P0：先保护玩家，不误报

- 给 Demo 增加“录制中/未完整结束/可播放”三态，而不是用文件大小猜测。
- 扫描和退出后 watcher 只在 CS2 进程可靠停止、文件 fingerprint 稳定、轻量完整性检查通过后才标记可播放。
- 对不完整文件保留原文件和诊断摘要，但禁用“播放”按钮，提示“录像已生成但未完整收尾”。

### P1：统一播放与隔离环境

- 合并两条播放入口的参数构造、路径转换和临时文件清理。
- 增加“纯 Demo 播放”实验开关：在有原子备份、恢复和 VAC/Online 门禁的前提下，验证不加载 BOT 插件是否能播放同一 Demo。未经实机验证不直接修改真实 `gameinfo.gi`。

### P2：定位插件冲突

- 若矩阵证明只有插件环境崩溃，按插件二分禁用：先 `BotAI` 原生 patch，再 `BotRandomizer`/换边外观，再其余托管插件。
- 重点审计换边事件中的实体、装备、外观重绑和原生函数 patch 生命周期；收集崩溃转储后再决定修具体插件还是升级上游核心。

## 是否重做上游整包

当前决策：**暂不重做**。原因是上游 v1.4.4 没有独立的 CSTV Demo 录制/播放实现，整包替换不能直接修复 CS2 原生 Demo 或 CounterStrikeSharp 访问冲突，反而会扩大变更面。

只有在隔离矩阵满足以下全部条件时才启动整包评估：

- 同一完整 Demo 在干净 CS2 可播放；
- 当前整包环境在换边/播放稳定崩溃；
- 关闭某个插件后崩溃消失；
- 上游新版本明确包含该插件/核心的兼容修复；
- 新整包通过插件最小组合、版本签名、回滚和真实 CS2 验收。

本报告不触发签名构建、上传或 0.5.13 发布流程。
