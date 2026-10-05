# DemoTracer 资源来源

本目录保存 CS2 人机增强助手 0.6.4 使用的 DemoTracer 上游来源快照与校验记录。

- 上游仓库：[unicbm/demotracer](https://github.com/unicbm/demotracer)
- 上游发行页：[v1.5.3](https://github.com/unicbm/demotracer/releases/tag/v1.5.3)
- GUI：`demotracer-gui-v1.5.3.exe`
- Playback：`demotracer-css-v1.5.2.zip`
- 源码快照：`demotracer-1.5.3-source.zip`
- 第一方许可证：AGPL-3.0-only。上游 `LICENSE`、第三方通知和版权信息随源码快照保留。

## 集成边界

助手只在安装时解包 Playback 清单列出的 `addons/` 文件，并把写入前的原始文件保存到
`game/csgo/cfg/cs2as05-demotracer-install-backups`。ownership 清单位于
`game/csgo/cfg/cs2as05-demotracer-install-ledger.json`。卸载时只有摘要仍与助手最近一次写入一致的文件才会被恢复或移除；玩家或其他插件改动过的文件会跳过并保留。

DemoTracer Playback 与 CS2-Bot-Improver v1.4.5 并行安装。重玩开始时由 DemoTracer 的
`dtr-hider` 按上游逻辑切换 Hider，换图后恢复原 BotHider；助手不会删除或替换人机增强的
Panel、BotController 或 BotHider 文件。

## 上游运行要求

- Windows x64 CS2 服务器
- Metamod 2.0 build 1469+，plugin API 18
- 启用 KHook 的 CounterStrikeSharp
- CounterStrikeSharp managed API `1.0.371` 或更新版本
- DemoTracer `dtr-controller` ABI 22，minor 至少 1

本项目的网页构建和安装包构建不能代替真实 CS2 回合重演验收；最终的 `.dtr` 导出和
BOT 重演仍需在玩家本机执行。
