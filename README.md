# CS2 人机增强助手

当前版本为 `0.5.2`。本仓库是基于 [ed0ard/CS2-Bot-Improver v1.4.2](https://github.com/ed0ard/CS2-Bot-Improver/releases/tag/v1.4.2) 开发的独立下游桌面项目，并非上游官方发行版。

- 本项目：[YuGeYu/CS2AS05](https://github.com/YuGeYu/CS2AS05)
- 上游项目：[ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)
- 上游基线：[v1.4.2 / `97fd57d2`](https://github.com/ed0ard/CS2-Bot-Improver/tree/97fd57d2ee1e14e408ae3ca7b1b0cae596a792cc)

本程序只提供以下操作：

- 自动扫描或手动选择 CS2 游戏目录。
- 安装或覆盖更新内置的定制 `CS2BotImprover.zip`。
- 安全提取并打开官方 `Panel v1.4.2.exe`。
- 卸载定制插件文件与读取基础诊断信息。
- 启动时与手动检查官网更新，并在系统默认浏览器打开官网、意见页和更新下载地址。
- 每 10 秒轻量刷新 CS2 运行状态；通过“关于与来源”查看上游项目与许可信息。

模式、难度、Aim、Nades、队伍、刀具、Bot 外观及命令均由官方 Panel 管理。本程序不再提供 AI 聊天、自定义指令、外观编辑、投掷物参数、Demo 管理或第三方项目展示。

## 构建

```powershell
npm install
npm run verify
cargo test --manifest-path src-tauri/Cargo.toml
npm run bundle:desktop
```

桌面安装包依赖仓库内的 `src-tauri/resources/CS2BotImprover.zip`。该文件是基于上游 `v1.4.2` 的最小定制包，SHA256：

```text
D6BCFF73BBACFEBC0BB3ED91A440A584EFE2C812962655F392F319463DA8D9E0
```

## 相对上游的修改

定制 ZIP 相对上游官方 ZIP 只替换 `addons/counterstrikesharp/plugins/NadeSystem/NadeSystem.dll`，其他内容保持上游 `v1.4.2`：

- 每个 BOT 每回合由 NadeSystem 调度的闪光最多 2 次，高爆、烟雾、燃烧瓶/燃烧弹各最多 1 次。
- 非紧急投掷使用开局 15 秒预算、同 BOT 5 秒间隔和同队 0.5 秒间隔；开局每队最多 1 颗烟雾和 3 颗进攻道具。
- 普通模式下，每队每回合最多成功执行 1 次计划烟雾；道具候选采用确定性评分和稳定排序，放宽高爆、燃烧瓶及团队闪光的有效选择范围。
- 硬上限、节奏、经济、队伍计数和冷却只在投掷物实体创建成功后提交，创建失败会取消预留。
- 成功提交后输出固定格式的 `[NadeAudit]` 审计行，可由随附测试工具严格解析和复算。

修改后的源码、策略测试、审计工具、补丁和基线说明位于 [`third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/`](./third_party/CS2-Bot-Improver-v1.4.2/nades-per-bot-round-limit/)。这些限制只覆盖 NadeSystem 调度的投掷，不覆盖 CS2 原生 AI 或其他第三方插件。

安装前请退出 CS2。应用会校验定制资源摘要和必要 ZIP 条目后，才会删除旧插件文件并开始安装。官方 Panel 仅提取到应用本地数据目录，不会写入 CS2 游戏目录。

## 授权与来源

本助手以 AGPL-3.0-or-later 发布。内置资源来自 [ed0ard/CS2-Bot-Improver](https://github.com/ed0ard/CS2-Bot-Improver)，并保留资源包内的许可证、README 和版权信息。详见 [NOTICE.md](./NOTICE.md)。
