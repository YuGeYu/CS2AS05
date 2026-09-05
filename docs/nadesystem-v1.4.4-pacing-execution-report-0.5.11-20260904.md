# NadeSystem v1.4.4 节奏恢复执行报告

日期：2026-09-04  
工作区：`E:\CS2AS05`

## 已完成

- 源码来源：用户提供的 `E:\dow\CS2-Bot-Improver-1.4.4.zip`，已解压到 `workspace/vendor-cache/CS2-Bot-Improver-v1.4.4-source/CS2-Bot-Improver-1.4.4`。
- v1.4.4 官方 release tag commit：`7491e175f83e612dbb1c742c2241d454ed4c15ad`。
- 在 `third_party/CS2-Bot-Improver-v1.4.4/nades-pacing/` 保存并构建完整 v1.4.4 NadeSystem 源码；`ModuleVersion = 1.2.1`。
- `NadeSystemPlugin.Audio.cs` 及 v1.4.4 的 radio/sound/chat 投掷通知逻辑完整保留；`AnnounceGrenadeThrow` 未删除或绕过。
- 旧 `NadePacingPolicy.cs` 仅作为策略文件接入 v1.4.4 `Replay` 计划投掷路径，特殊道具仍使用官方路径；未把旧版 NadeSystem DLL 覆盖回包内。
- 当前编译产物 `NadeSystem.dll` SHA-256：`2668B41B019F2BDBB7C89051136B95EFD0E46A7A044553408FDF33048B4A2654`。
- 当前资源 ZIP：`src-tauri/resources/CS2BotImprover.zip`，SHA-256：`634BC9B854A0F39349474EC73463E4CAED6454BB0344DA7C89B9A1D2D268FE7F`。
- 当前 payload manifest SHA-256：`54A7940DCFD861F1C3E6A01904A0CB21B8DC9EDACD39E1A021C3AC33F4F3D0C8`；BotVision 保持 version `0.2.2`、原 provenance 与 source hash，未升级、降级或覆盖。
- `gameinfo.gi` 未修改，保持 v1.4.4 上游内容。
- 历史 `NadePacingPolicy.Tests.csproj` 已执行：成功、0 警告、0 错误；该测试位于 v1.4.3 目录，仅作为策略回归证据。
- 定向契约测试已通过：2 个文件、7 个测试全部通过；`npm run typecheck` 通过；`cargo check --manifest-path src-tauri/Cargo.toml` 通过。
- 本机 NSIS 安装器已生成并复制到：
  - `E:\CS2AS05\artifacts\nadesystem-v1.4.4-pacing-0.5.11-local-test-20260904\CS2人机增强助手_0.5.11_x64-setup.exe`
  - 大小 `118242860` bytes
  - SHA-256 `DB1770019CE647E099010885C752A22078E47E1614C5FC304C32F787FFF62356`
  - Authenticode：`NotSigned`（按本机测试要求，签名不作为阻断条件）

## 当前结论

`v1.4.4 NadeSystem 节奏恢复已通过自动化，本机安装器已生成；BotVision 保持原状，gameinfo.gi 保持上游，待真实 BOT 验收。`

真实 CS2/BOT 进程内行为尚未由本轮自动化确认。安装后请用 High 难度 BOT 验证手雷投掷节奏，以及 v1.4.4 官方新增的 radio、sound、chat 提示。
