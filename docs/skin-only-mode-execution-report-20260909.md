# 只开换肤模式实现记录

日期：2026-09-09

## 已实现

- 新增运行模式 `skin_only`，界面显示为“只开换肤”。
- `skin_only` 启动仍使用本地 `-insecure` 与带 MetaMod/CounterStrikeSharp 的 `gameinfo.gi`，因此可以加载 Inventory Simulator。
- 启动前检查 `InventorySimulator.dll`；未安装时阻止启动并提示先安装库存换肤。
- 进入 `skin_only` 时，将受本项目管理的 BOT 插件目录临时移动到 `addons/counterstrikesharp/.cs2as-skin-only-plugins`，该目录不属于 CounterStrikeSharp 插件扫描根目录。
- 同时隔离 `addons/BotController` 到 `addons/.cs2as-skin-only/BotController`。
- 切回 `bots` 或 `online` 时恢复上述插件目录，并删除模式状态文件。
- 使用 `cfg/cs2as05-skin-only.state` 识别当前模式；面板回读会显示 `skin_only`。
- 只开换肤时禁用 BOT 难度与自定义强度操作，避免误写 BOT 配置。
- 库存换肤页的启动按钮改为“启动只开换肤”。

受管 BOT 插件目录包括：`BotAI`、`BotAimImprover`、`BotBuy`、`BotControllerImpl`、`BotHiderImpl`、`BotRandomizer`、`BotState`、`MapRotation`、`NadeSystem`、`RayTraceImpl`。

## 已验证

- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过；仅有仓库已有未使用代码警告。
- `cargo fmt` 与 `git diff --check`：通过。

## 尚未确认

- 尚未在真实 CS2 客户端启动 `skin_only`。
- 尚未在真实创意工坊闯关地图确认 BOT 全灭后下一关推进。
- 尚未在真实多人房间确认 Inventory Simulator 皮肤、重生和换队后的显示。

真实游戏验收时应重点检查启动日志和 CounterStrikeSharp 实际插件列表：只允许 Inventory Simulator 及其必要依赖加载，不应出现上述 BOT 插件初始化日志。若地图自身依赖其中某个插件，应将该插件从隔离清单中移除，并单独记录兼容性，不得静默改变地图规则。
