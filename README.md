# CS2 人机增强助手

当前版本为 `0.5.3`。本仓库是基于 [ed0ard/CS2-Bot-Improver v1.4.2](https://github.com/ed0ard/CS2-Bot-Improver/releases/tag/v1.4.2) 开发的独立下游桌面项目，并非上游官方发行版。

- 本项目：[YuGeYu/CS2AS05](https://github.com/YuGeYu/CS2AS05)
- 上游项目：[ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)
- 上游基线：[v1.4.2 / `97fd57d2`](https://github.com/ed0ard/CS2-Bot-Improver/tree/97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc)

本程序提供以下操作：

- 快速扫描、使用可停止的“猜你想选”深度查找，或手动选择 CS2 游戏目录。
- 安装或覆盖更新内置的定制 `CS2BotImprover.zip`。
- 原生管理在线/BOT 模式、难度、Aim、Nades、Bot 物品和丢刀配置，并按当前模式启动 CS2；启动期间显示最长 30 秒的可关闭状态特效。
- 提供 40 支队伍预设、完整命令搜索与可靠剪贴板复制。
- 在高级兼容区域安全提取并打开原版 `Panel v1.4.2.exe`。
- 卸载定制插件文件与读取基础诊断信息。
- 启动时与手动检查官网更新，并在系统默认浏览器打开官网、意见页和更新下载地址。
- 每 10 秒轻量刷新 CS2 运行状态；通过“关于与来源”查看上游项目与许可信息。

界面仅提供简体中文。应用不提供 AI 聊天、真人饰品编辑、Demo 管理、比赛管理或第三方品牌展示。

## Panel 融合调查

关于将上游 Panel 功能融合进本助手的源码调查、来源边界和目标功能清单，见 [`docs/panel-integration-investigation.md`](./docs/panel-integration-investigation.md)。面向实际执行 AI 的 0.5.3 文件级实施、验证、回退和发布方案，见 [`docs/panel-integration-plan-0.5.3.md`](./docs/panel-integration-plan-0.5.3.md)。

概览页启动特效、可停止的 CS2 目录推荐扫描，以及取消 0.5.2 新增 `[NadeAudit]` 控制台输出的后续实施方案，见 [`docs/overview-launch-discovery-and-nade-log-plan.md`](./docs/overview-launch-discovery-and-nade-log-plan.md)。

0.5.3 标题栏正式图标、侧栏版本文字以及 BOT 启动前插件版本检查/自动安装门禁方案，见 [`docs/overview-brand-and-bot-plugin-gate-plan-0.5.3.md`](./docs/overview-brand-and-bot-plugin-gate-plan-0.5.3.md)。该方案明确不改“安装与诊断”页的手动安装逻辑。

0.5.3 刀具图片本地化、首次默认值与覆盖安装时保留用户选择的完整方案，见 [`docs/knife-images-and-panel-defaults-plan-0.5.3.md`](./docs/knife-images-and-panel-defaults-plan-0.5.3.md)。

## 构建

```powershell
npm install
npm run verify
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

桌面安装包依赖仓库内的 `src-tauri/resources/CS2BotImprover.zip`。该文件是基于上游 `v1.4.2` 的最小定制包，SHA256：

```text
55DC504BFF8340ABE6AB317661518B512DB6FFA463F10EE3AF84149B1795C3E9
```

## 相对上游的修改

定制 ZIP 相对上游官方 ZIP 替换 `addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll`，新增仅供助手验证版本和 payload 完整性的 `CS2AS05.plugin.json`，并将首次 Panel 默认值固化为 BOT / Low / mixed / normal / Bot 物品全开 / 20 刀全选。资源差异见 [`docs/CS2BotImprover-defaults-diff-0.5.3.json`](./docs/CS2BotImprover-defaults-diff-0.5.3.json)：

- 每个 BOT 每回合由 NadeSystem 调度的闪光最多 2 次，高爆、烟雾、燃烧瓶/燃烧弹各最多 1 次。
- 非紧急投掷使用开局 15 秒预算、同 BOT 5 秒间隔和同队 0.5 秒间隔；开局每队最多 1 颗烟雾和 3 颗进攻道具。
- 普通模式下，每队每回合最多成功执行 1 次计划烟雾；道具候选采用确定性评分和稳定排序，放宽高爆、燃烧瓶及团队闪光的有效选择范围。
- 硬上限、节奏、经济、队伍计数和冷却只在投掷物实体创建成功后提交，创建失败会取消预留。
- 不再生成 `[NadeAudit]` 控制台行；原有 `[NadeSystem]` 加载、命令用法、实体创建和投掷物错误诊断保持不变。

修改后的源码、策略测试、补丁和基线说明位于 [`third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/`](./third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/)。这些限制只覆盖 NadeSystem 调度的投掷，不覆盖 CS2 原生 AI 或其他第三方插件。

安装、卸载和模式切换前请退出 CS2。应用会校验定制资源摘要和必要 ZIP 条目，再以事务方式覆盖程序文件；覆盖前会捕获可识别的 Panel 选择，安装后恢复并回读，失败则回滚。首次迁移状态保存在 `game/csgo/cfg/cs2as05-panel-state.json`，已有 online、中高难度、Aim/Nades、Bot 物品全关和空刀具等合法值均优先保留。原版 Panel 仅提取到应用本地数据目录，不会写入 CS2 游戏目录。

## 授权与来源

本助手以 AGPL-3.0-or-later 发布。内置资源来自 [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)，并保留资源包内的许可证、README 和版权信息。详见 [NOTICE.md](./NOTICE.md)。
