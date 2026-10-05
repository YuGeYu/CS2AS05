# CS2 人机增强助手

当前版本为 `0.6.4`。本仓库是基于 [ed0ard/CS2-Bot-Improver v1.4.5](https://github.com/ed0ard/CS2-Bot-Improver/releases/tag/v1.4.5) 开发的独立下游桌面项目，并非上游官方发行版。

- 本项目：[YuGeYu/CS2AS05](https://github.com/YuGeYu/CS2AS05)
- 上游项目：[ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)
- 上游基线：`v1.4.5`（Windows 资源包与源码快照均保存在本仓库）

本程序提供以下操作：

- 快速扫描、使用可停止的“猜你想选”深度查找，或手动选择 CS2 游戏目录。
- 安装或覆盖更新内置的上游官方 `CS2BotImprover.zip`，并保留安装事务与失败回滚。
- 通过上游 `Panel v1.4.5` 使用 Online/BOT 模式、难度、Aim、Nades、Bot 物品、丢刀、规则和开发者信息；助手不再维护一套平行控制面板。
- 保留本项目的“指令研究所”，用于检索公开指令资料和复制命令；上游 Panel 的常用指令页仍是游戏配置的权威入口。
- 提供本地 Demo 目录扫描、手动导入、SQLite 录像库，以及基于可靠事件字段的回合、击杀、炸弹时间线、对局战报和个人表现雷达。
- 可为助手启动的 BOT/本地托管对局写入 `tv_enable` / `tv_autorecord`；官方匹配是否提供 Demo 仍由服务器或平台决定。
- 在“安装与诊断”和概览页安全提取并打开原版 `Panel v1.4.5.exe`，它是模式、规则和 BOT 预设的主要操作入口。
- 将官方 Inventory Simulator v3.5.0 资源按 `addons/counterstrikesharp/plugins/InventorySimulator` 与 `gamedata/inventory-simulator.json` 目录安装、校验和移除；它只服务助手启动的 `-insecure` 本地 BOT 场景，并包含武器、刀、手套、角色、贴纸、挂件、音乐盒、涂鸦、StatTrak、磨损和宠物能力。
- 卸载定制插件文件、读取基础诊断信息，并提供可随时停止的快快客服会话；会话保存在应用数据目录，不使用浏览器存储。
- 启动时与手动检查官网更新，并在系统默认浏览器打开官网、意见页和更新下载地址。
- 每 2 秒轻量刷新 CS2 运行状态；通过“关于与来源”查看上游项目与许可信息。
- 内置包并行安装 BotVision `0.2.2` MetaMod 组件；它与 CounterStrikeSharp/NadeSystem 分开校验。

界面仅提供简体中文。应用不提供真人饰品编辑、比赛管理、视频剪辑或云端 Demo 上传。快快客服仅在用户配置或内置额度可用时请求指定的 API；自定义连接密钥不会显示在页面或诊断日志中。

Demo 录像库数据库位于应用数据目录的 `demo-review/demo-review-v1.sqlite3`。程序只读用户明确添加或手动选择的 `.dem` 文件，不移动、不重命名、不删除原录像；解析结果缺失时显示 `--`，不会推测玩家或比分数据。

## Panel 融合调查

关于将上游 Panel 功能融合进本助手的源码调查、来源边界和目标功能清单，见 [`docs/panel-integration-investigation.md`](./docs/panel-integration-investigation.md)。面向实际执行 AI 的 0.5.3 文件级实施、验证、回退和发布方案，见 [`docs/panel-integration-plan-0.5.3.md`](./docs/panel-integration-plan-0.5.3.md)。

概览页启动特效、可停止的 CS2 目录推荐扫描，以及取消 0.5.2 新增 `[NadeAudit]` 控制台输出的后续实施方案，见 [`docs/overview-launch-discovery-and-nade-log-plan.md`](./docs/overview-launch-discovery-and-nade-log-plan.md)。

0.5.3 标题栏正式图标、侧栏版本文字以及 BOT 启动前插件版本检查/自动安装门禁方案，见 [`docs/overview-brand-and-bot-plugin-gate-plan-0.5.3.md`](./docs/overview-brand-and-bot-plugin-gate-plan-0.5.3.md)。该方案明确不改“安装与诊断”页的手动安装逻辑。

0.5.3 刀具图片本地化、首次默认值与覆盖安装时保留用户选择的完整方案，见 [`docs/knife-images-and-panel-defaults-plan-0.5.3.md`](./docs/knife-images-and-panel-defaults-plan-0.5.3.md)。

0.5.3 主导航、分段控件、Bot 物品、刀具、命令复制反馈及全局科技感 UI 优化方案，见 [`docs/technology-motion-ui-plan-0.5.3.md`](./docs/technology-motion-ui-plan-0.5.3.md)。

历史版本的上游迁移方案仍保留在文档目录；当前 0.6.4 的核心资源与 Panel 已统一切换到上游 `v1.4.5`，详见 [`docs/release-upstream-v1.4.5-0.6.4.md`](./docs/release-upstream-v1.4.5-0.6.4.md)。

库存换肤已同步到 `ianlucas/cs2-css-inventory-simulator` Release `3.5.0`，包括宠物、暖身阶段复活和自由漫游能力；来源、哈希和兼容边界见 [`docs/inventory-simulator-3.5.0-upgrade-20261005.md`](./docs/inventory-simulator-3.5.0-upgrade-20261005.md)。

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

桌面安装包依赖仓库内的 `src-tauri/resources/CS2BotImprover.zip`。0.6.4 使用上游 `v1.4.5` Windows 资源包原样内容，不在资源层做下游修改。完整源码快照位于 `third_party/CS2-Bot-Improver-v1.4.5-source.zip`。资源包 SHA256：

```text
AE37B86533ABFE0547C5AD4346D478CD846727509FC092842A81240EB0130140
```

## 相对上游的修改

0.6.4 的 ZIP 原样保留上游许可证、来源文件和 v1.4.5 运行资产，不加入下游 marker 或 manifest。助手仍会在安装前校验 ZIP 可读性、Panel 摘要和上游核心文件，覆盖升级会保留用户已明确选择的合法面板值；仅缺失或未初始化字段使用新默认。资源变体与验收边界见 [`docs/release-upstream-v1.4.5-0.6.4.md`](./docs/release-upstream-v1.4.5-0.6.4.md)。

本版本不再打包或注册自动换图 `MapRotation`、`CS2BotLlmChat` 等下游实验组件。安装官方包时会把旧版本遗留的这两类目录暂存到事务备份并清理，失败则恢复；库存换肤资源单独按官方目录部署，卸载 BOT 包也不会删除 Inventory Simulator。

0.6.4 不改写上游 NadeSystem、Rush 行为树、无线电跟随、下包守包或饰品数据；这些能力直接随 v1.4.5 资源包安装。

安装、卸载和模式切换前请退出 CS2。应用会校验上游 v1.4.5 ZIP 可读性和必要条目，再以事务方式把资源写入 `game/csgo`；覆盖前会捕获可识别的 Panel 选择，安装后恢复并回读，失败则回滚。BOT 与 Online 的 `gameinfo.gi` 使用各自静态资源，Online 路径不会携带 `-insecure`。Windows 安装按上游教程将除 Panel 外的文件复制到 `game/csgo`；Linux 资源包仍按上游教程复制到 `game/csgo` 并在 Steam 启动项加入 `-insecure`。原版 Panel 仅提取到应用本地数据目录，不会写入 CS2 游戏目录。当前契约见 [`docs/release-upstream-v1.4.5-0.6.4.md`](./docs/release-upstream-v1.4.5-0.6.4.md)。

兼容性提示：部分用户的 CS2/插件环境可能无法在游戏内应用“探员模型”和“丢刀/刀具”开关。官方 Panel 在相同环境也可能出现相同限制。助手仍会按契约写入并回读配置；本提示不代表每台电脑都能得到对应的游戏内效果。

## 授权与来源

本助手以 AGPL-3.0-or-later 发布。内置资源来自 [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)，并保留资源包内的许可证、README 和版权信息。详见 [NOTICE.md](./NOTICE.md)。
