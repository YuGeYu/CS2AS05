# CS2 人机增强助手

当前候选版本为 `0.5.5`。本仓库是基于 [ed0ard/CS2-Bot-Improver v1.4.3](https://github.com/ed0ard/CS2-Bot-Improver/releases/tag/v1.4.3) 开发的独立下游桌面项目，并非上游官方发行版。

- 本项目：[YuGeYu/CS2AS05](https://github.com/YuGeYu/CS2AS05)
- 上游项目：[ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)
- 上游基线：[v1.4.3 / `d1d83982`](https://github.com/ed0ard/CS2-Bot-Improver/tree/d1d83982db88fbdb686b2bf13aa8c6f9d65a4604)

本程序提供以下操作：

- 快速扫描、使用可停止的“猜你想选”深度查找，或手动选择 CS2 游戏目录。
- 安装或覆盖更新内置的定制 `CS2BotImprover.zip`。
- 原生管理在线/BOT 模式、难度、Aim、Nades、Bot 物品和丢刀配置，并按当前模式启动 CS2；启动期间显示最长 30 秒的可关闭状态特效。
- 提供 40 支队伍预设、完整命令搜索与可靠剪贴板复制。
- 提供本地 Demo 目录扫描、手动导入、SQLite 录像库，以及基于可靠事件字段的回合、击杀和炸弹时间线。
- 可为助手启动的 BOT/本地托管对局写入 `tv_enable` / `tv_autorecord`；官方匹配是否提供 Demo 仍由服务器或平台决定。
- 在高级兼容区域安全提取并打开原版 `Panel v1.4.3.exe`。
- 卸载定制插件文件与读取基础诊断信息。
- 启动时与手动检查官网更新，并在系统默认浏览器打开官网、意见页和更新下载地址。
- 每 10 秒轻量刷新 CS2 运行状态；通过“关于与来源”查看上游项目与许可信息。

界面仅提供简体中文。应用不提供 AI 聊天、真人饰品编辑、比赛管理、视频剪辑、云端 Demo 上传或第三方品牌展示。

Demo 录像库数据库位于应用数据目录的 `demo-review/demo-review-v1.sqlite3`。程序只读用户明确添加或手动选择的 `.dem` 文件，不移动、不重命名、不删除原录像；解析结果缺失时显示 `--`，不会推测玩家或比分数据。

## Panel 融合调查

关于将上游 Panel 功能融合进本助手的源码调查、来源边界和目标功能清单，见 [`docs/panel-integration-investigation.md`](./docs/panel-integration-investigation.md)。面向实际执行 AI 的 0.5.3 文件级实施、验证、回退和发布方案，见 [`docs/panel-integration-plan-0.5.3.md`](./docs/panel-integration-plan-0.5.3.md)。

概览页启动特效、可停止的 CS2 目录推荐扫描，以及取消 0.5.2 新增 `[NadeAudit]` 控制台输出的后续实施方案，见 [`docs/overview-launch-discovery-and-nade-log-plan.md`](./docs/overview-launch-discovery-and-nade-log-plan.md)。

0.5.3 标题栏正式图标、侧栏版本文字以及 BOT 启动前插件版本检查/自动安装门禁方案，见 [`docs/overview-brand-and-bot-plugin-gate-plan-0.5.3.md`](./docs/overview-brand-and-bot-plugin-gate-plan-0.5.3.md)。该方案明确不改“安装与诊断”页的手动安装逻辑。

0.5.3 刀具图片本地化、首次默认值与覆盖安装时保留用户选择的完整方案，见 [`docs/knife-images-and-panel-defaults-plan-0.5.3.md`](./docs/knife-images-and-panel-defaults-plan-0.5.3.md)。

0.5.3 主导航、分段控件、Bot 物品、刀具、命令复制反馈及全局科技感 UI 优化方案，见 [`docs/technology-motion-ui-plan-0.5.3.md`](./docs/technology-motion-ui-plan-0.5.3.md)。

基于上游 `v1.4.3` Windows 整包开发 `0.5.4`、对齐八项 Bot Items、加入 `bot_nades less`、重放道具投掷节奏并保留现有安装/签名/关闭链路的完整交接方案，见 [`docs/upstream-v1.4.3-release-plan-0.5.4.md`](./docs/upstream-v1.4.3-release-plan-0.5.4.md)。

0.5.4 发布收尾、首次默认值调整、Panel 恢复源码评估和探员模型/丢刀兼容性限制说明，见 [`docs/release-readiness-plan-0.5.4-defaults-recovery-20260727.md`](./docs/release-readiness-plan-0.5.4-defaults-recovery-20260727.md)。

最终签名构建、自动化重跑、真实 CS2 验收、D1/R2/GitHub 发布顺序和停止条件，见 [`docs/final-release-execution-plan-0.5.4-20260727.md`](./docs/final-release-execution-plan-0.5.4-20260727.md)。

0.5.5 自定义 Steam 安装目录启动修复、“安装与诊断”页进页即检查更新、版本/插件 marker 一致性及签名发布的完整交接方案，见 [`docs/version-0.5.5-steam-launch-and-update-entry-plan-20260727.md`](./docs/version-0.5.5-steam-launch-and-update-entry-plan-20260727.md)。

## 构建

```powershell
npm install
npm run verify
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

桌面安装包依赖仓库内的 `src-tauri/resources/CS2BotImprover.zip`。该文件是基于上游 `v1.4.3` 的最小定制包，SHA256：

```text
ACC5E0B73626A86F3C07ECDAE04B164F806F7D5A30DDC692C3C8C864FF73F4AB
```

## 相对上游的修改

定制 ZIP 相对上游官方 ZIP 仅修改两个 BOT cfg、替换 `addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll`，并新增供助手验证版本和 payload 完整性的 `CS2AS05.plugin.json`。首次 Panel 默认值为 BOT / Low / mixed / less / 八项 Bot 物品全开 / 刀具 `507、508、515、519、525`。覆盖升级会保留用户已经明确选择的合法值，包括 `normal` Nades、全关 Bot Items 和空刀具；仅缺失或未初始化字段使用新默认。资源差异见 [`docs/CS2BotImprover-defaults-diff-0.5.4.json`](./docs/CS2BotImprover-defaults-diff-0.5.4.json)，0.5.5 marker 证据见 [`docs/CS2BotImprover-marker-diff-0.5.5.json`](./docs/CS2BotImprover-marker-diff-0.5.5.json)：

- 每个 BOT 每回合由 NadeSystem 调度的闪光最多 2 次，高爆、烟雾、燃烧瓶/燃烧弹各最多 1 次。
- 非紧急投掷使用开局 15 秒预算、同 BOT 5 秒间隔和同队 0.5 秒间隔；开局每队最多 1 颗烟雾和 3 颗进攻道具。
- 普通模式下，每队每回合最多成功执行 1 次计划烟雾；道具候选采用确定性评分和稳定排序，放宽高爆、燃烧瓶及团队闪光的有效选择范围。
- 硬上限、节奏、经济、队伍计数和冷却只在投掷物实体创建成功后提交，创建失败会取消预留。
- 不再生成 `[NadeAudit]` 控制台行；原有 `[NadeSystem]` 加载、命令用法、实体创建和投掷物错误诊断保持不变。

修改后的源码、策略测试和基线说明位于 [`third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/`](./third_party/CS2-Bot-Improver-v1.4.3/nades-pacing/)。`less` 保留上游 `1.1.7` 的四次总上限，并与计划投掷、紧急投掷和反击投掷共用事务式硬上限。这些限制只覆盖 NadeSystem 调度的投掷，不覆盖 CS2 原生 AI 或其他第三方插件。

安装、卸载和模式切换前请退出 CS2。应用会校验定制资源摘要和必要 ZIP 条目，再以事务方式覆盖程序文件；覆盖前会捕获可识别的 Panel 选择，安装后恢复并回读，失败则回滚。首次迁移状态保存在 `game/csgo/cfg/cs2as05-panel-state.json`，八项 Bot 物品读写上游 `addons/counterstrikesharp/configs/core.json` 并保留未知字段；旧 `bot_randomizer_options.json` 可残留但不再参与状态。已有 online、中高难度、Aim/Nades、Bot 物品全关和空刀具等合法值均优先保留。原版 Panel 仅提取到应用本地数据目录，不会写入 CS2 游戏目录。当前契约见 [`docs/panel-v1.4.3-contract.md`](./docs/panel-v1.4.3-contract.md)。

兼容性提示：部分用户的 CS2/插件环境可能无法在游戏内应用“探员模型”和“丢刀/刀具”开关。官方 Panel 在相同环境也可能出现相同限制。助手仍会按契约写入并回读配置；本提示不代表每台电脑都能得到对应的游戏内效果。

## 授权与来源

本助手以 AGPL-3.0-or-later 发布。内置资源来自 [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)，并保留资源包内的许可证、README 和版权信息。详见 [NOTICE.md](./NOTICE.md)。
