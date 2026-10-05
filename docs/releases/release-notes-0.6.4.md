# CS2 人机增强助手 0.6.4 更新日志

【新增】

- 新增“Demo 重玩”核心工作台，集成 DemoTracer GUI v1.5.3 与 CSS Playback v1.5.2，支持 Demo 导入、回合筛选、`.dtr`/manifest 生成、Playback 安装状态检查、上游 GUI 启动和重玩命令查看。
- 新增 DemoTracer Playback 安装事务与 ownership 清单，安装前备份原文件，失败自动回滚，为 Demo 重玩资源提供可追踪的安装和卸载边界。
- 新增 CS2-Bot-Improver v1.4.5 官方核心资源对齐，随包提供 Rush 行为树兼容、真人视角 FOV、贴纸与音乐盒、下包守包、无线电跟随、`bot_aim body` 爆头率调整和 Panel Rules 等上游能力。
- 新增 Inventory Simulator v3.5.0 官方资源同步，包含宠物生成、暖身阶段复活和宠物自由漫游等上游能力。
- 新增桌面端“指令研究所”原生数据通道，通过 Tauri 后端加载官网资料，并提供中文错误提示和重新加载操作。

【优化】

- 优化核心工作台功能边界：联机/机器人模式、难度、瞄准、投掷物、掉落刀具、机器人物品档案、常用指令、规则和开发者信息统一由上游 Panel v1.4.5 提供；助手继续保留“指令研究所”和库存换肤资源部署能力，减少重复配置入口。
- 优化 DemoTracer 与人机增强并行安装策略，保留 Panel、BotController、BotHider 和其他既有上游资源；DemoTracer 仅在实际重玩流程中接管所需组件，避免覆盖现有 BOT 功能。
- 优化 CS2 新版游戏层兼容性，按根 `gameinfo.gi` 实际声明检查依赖，兼容 BuildID `25640462` 的官方合并布局，并分别校验 Online、BOT 和只开换肤模式资源。
- 优化定制资源选择与缓存回退，候选 ZIP 在使用前完整校验，旧缓存缺少 Panel v1.4.5 时自动跳过并从当前版本内嵌官方资源恢复。
- 优化 BOT 插件版本兼容策略，只要已安装 marker 通过身份、结构和关键文件校验即可继续使用，不再因为旧版有效资源的版本号不同而重复安装。
- 优化概览页面，移除助手自动创建本地对局记录的入口和状态卡片；已有 Demo 仍可在“对局复盘”中手动导入和查看。
- 优化关闭确认流程，统一标题栏、Alt+F4 和托盘退出行为，补充返回助手、Esc 取消、焦点管理和可持久化的玩家选择。

【修复】

- 修复旧版“卸载插件”可能误删 Steam 官方 `cfg/gamemode_*.cfg`、`gameinfo.gi` 等文件的问题。现在只按 ownership 清单恢复或移除助手实际写入的文件，玩家或 Steam 修改过的文件会被保留。
- 修复旧 Tauri 资源缓存被优先选中、导致安装时提示 `[PANEL_ASSET_INVALID]` 缺少 Panel v1.4.5 的问题；现在严格校验 Panel 版本、大小、摘要和 ZIP 条目路径。
- 修复 CS2 更新后启动检查无条件要求历史 `csgo_imported` / `csgo_core` 层，导致新版完整游戏被错误拦截的问题。
- 修复“指令研究所”桌面 WebView 因官网接口缺少 CORS 头而固定显示 `Failed to fetch` 的问题，桌面端改由 Rust 后端请求并返回中文错误信息。
- 修复 Windows 原生关闭事件绕过前端确认弹窗的问题，退出程序时会先处理战报窗口和保存的关闭偏好。
- 修复有效的旧版 BOT 插件 marker 被当前程序误判为版本不匹配、重复安装或安装失败的问题。

【其他】

- 正式版本号为 `0.6.4`，本版本同时包含上游 BotImprover v1.4.5、Inventory Simulator v3.5.0 和 DemoTracer v1.5.3/v1.5.2 资源。
- DemoTracer 需要满足上游运行环境要求，包括 Windows x64、Metamod 2.0 build 1469+、CounterStrikeSharp managed API 1.0.371+、启用 KHook 的 CounterStrikeSharp 以及匹配的 BotController ABI。
- DemoTracer、Inventory Simulator 和库存换肤功能面向助手启动的本地 `-insecure` BOT 场景，不适用于官匹、5E、完美或其他受保护环境。
- 本版本保留上游项目许可证、版权和来源信息；Tauri updater 使用独立 `.sig` 验证，Windows NSIS Authenticode 状态单独记录。
- 发布前用户已确认完成本机实际测试并授权正式发布；不同玩家设备仍需满足本日志列出的 CS2、DemoTracer 与本地 `-insecure` 使用条件。
