# 0.6.4 上游资源对齐记录

## 范围

0.6.4 将核心 infrastructures 完全对齐到 `ed0ard/CS2-Bot-Improver v1.4.5`。本版本不修改上游 Windows 资源包中的任何文件，不对 DLL、Panel、配置、游戏资源或行为树做下游改写。

## 来源与归档

- 上游项目：https://github.com/ed0ard/CS2-Bot-Improver
- 上游版本：`v1.4.5`
- Windows 资源包来源：`E:\dow\CS2BotImprover (9).zip`
- 源码来源：`E:\dow\CS2-Bot-Improver-1.4.5.zip`
- 项目内资源包：`src-tauri/resources/CS2BotImprover.zip`
- 项目内源码归档：`third_party/CS2-Bot-Improver-v1.4.5-source.zip`

## 完整性证据

以下 SHA-256 在复制前后逐字节核对：

| 文件 | SHA-256 |
| --- | --- |
| `CS2BotImprover (9).zip` | `AE37B86533ABFE0547C5AD4346D478CD846727509FC092842A81240EB0130140` |
| `src-tauri/resources/CS2BotImprover.zip` | `AE37B86533ABFE0547C5AD4346D478CD846727509FC092842A81240EB0130140` |
| `CS2-Bot-Improver-1.4.5.zip` | `C5B38A6421607D756D741F47C7423DE9C2A3256AEEC9D009F180C4970E2A0E3B` |
| `third_party/CS2-Bot-Improver-v1.4.5-source.zip` | `C5B38A6421607D756D741F47C7423DE9C2A3256AEEC9D009F180C4970E2A0E3B` |

## v1.4.5 能力边界

上游版本说明包含：修复损坏功能、Rush 行为树兼容、与真人一致的 FOV、579 个贴纸与 Starjunk 95 音乐盒、下包后守包、跟随玩家无线电指令、`bot_aim body` 爆头率调整、Panel Rules 菜单、Linux 支持及相关修复。上述能力由上游资源包提供，本项目本轮不重复实现、不声称已完成真实游戏验收。

## 验收边界

- 已完成：源 ZIP 复制、源码快照归档、版本与安装页来源文案同步、SHA-256 与 ZIP 可读性核验。
- 为保持官方 ZIP 原样，运行时兼容了没有下游 marker/manifest 的上游包；v1.4.5 包提供 Online 与 WithBots 两份静态 `gameinfo.gi`，当前安装层按原有功能需求从 WithBots 基线补齐 SkinOnly 兼容副本，不改 ZIP。
- 上游 Panel v1.4.5 SHA-256：`9C6BD8E2503AFC9CAEB5DD64C8B8BF0EC5967BF50CD442015E7CEBEB69038410`，大小 `6,032,896` bytes；已同步更新内置提取校验契约。
- 未完成：真实 CS2 Windows/Linux 游戏内行为验收；需要在目标设备上由用户执行。
- 发布构建只代表本地构建产物成功，不等同于真实游戏效果或签名/上传完成。

## Panel 文件完整性修复

上游 Panel 会额外检查 11 个模式配置文件和根 gameinfo.gi。0.6.4 修复后，安装事务会从 v1.4.5 ZIP 原样复制这些文件，并在事务完成后逐项回读；任意一项缺失都会回滚并报告 [UPSTREAM_PANEL_FILES_MISSING]，不会再显示安装成功。

## 下游实验组件清理与功能边界

0.6.4 不再把自动换图 `MapRotation` 或 `CS2BotLlmChat` 放进资源包、Tauri command 或 BOT 物品页。覆盖安装时如果检测到旧版本留下的这两类目录，会先移动到本次事务备份；安装成功后随备份策略处理，安装失败则恢复原目录。卸载官方 BOT 包时只清理上游 v1.4.5 自身的插件、配置、覆盖难度和 gameinfo 备份，不删除 CounterStrikeSharp 或 Inventory Simulator。

模式切换、难度、瞄准预设、投掷物预设、掉落刀具、机器人物品档案、常用指令、规则和开发者信息统一以随包的 `Panel v1.4.5.exe` 为权威入口；助手保留“指令研究所”和 Inventory Simulator 官方资源部署页，避免两套配置写入同一份游戏文件。

## 0.6.4 自动化验证

- `cargo fmt --check --manifest-path src-tauri/Cargo.toml`：通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：`102 passed, 0 failed, 4 ignored`。4 项忽略测试均需要真实 CS2、真实 Demo 或用户本机数据库，不能由本地编译替代。
- `npm run verify`：通过，包含工作区登记、`vue-tsc`、Oxlint、ESLint、Vitest 和 Web 构建；Vitest 为 `56 files / 197 tests passed`。
- 官方资源包契约：官方 ZIP SHA-256 与上表一致；不存在 `MapRotation`、`CS2BotLlmChat`、下游 marker 或 `gameinfo.manifest.json`；Panel、Online/WithBots `gameinfo.gi` 和 Inventory Simulator 资源校验通过。

## 0.6.4 本机测试安装器

已执行 `npm run bundle:desktop`，成功完成前端构建、Rust release 编译和 NSIS 打包，安装器路径为：

`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe`

- 大小：`119,955,256` bytes
- SHA-256：`8E83D11D14D54304A4EF020D11BC69B22B7B27E6745BC9B043E9CD726A461857`
- Windows Authenticode：`NotSigned`
- Tauri updater `.sig`：本次未生成

NSIS 安装器本身已经生成，可用于本机和内部玩家测试。构建命令最后返回退出码 `1` 的唯一原因是当前环境只有 updater 公钥，没有配置 `TAURI_SIGNING_PRIVATE_KEY`；没有用伪造签名覆盖这个状态。真实 CS2 游戏内行为、Panel 启动、Inventory Simulator 加载及安装升级兼容性仍需在目标设备上执行验收。
