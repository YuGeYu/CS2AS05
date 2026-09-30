# CS2 人机增强助手 0.6.3 更新日志

【新增】

- 新增 CounterStrikeSharp 首次初始化兼容处理：检测到基础环境完整但 `core.json` 尚未生成时，助手会自动创建默认配置并继续安装流程。
- 新增安装器资源去重构建方案，保留内嵌资源回退能力，同时移除重复的外置定制资源包。

【优化】

- 优化 Panel 首次启动与库存换肤前置检查，减少因 CounterStrikeSharp 尚未首次启动导致的误报。
- 优化安装包资源组织方式，定制资源包只保留单份有效来源，安装器体积较上一版减少约 67.42 MiB。
- 优化资源回退路径校验，继续验证根级 `gameinfo.manifest.json`，避免使用不完整或不可信的资源包。

【修复】

- 修复部分玩家在 CounterStrikeSharp 已安装、但 `addons/counterstrikesharp/configs/core.json` 不存在时，助手提示“读取 core.json 失败：系统找不到指定的文件”的问题。
- 修复同一个 `CS2BotImprover.zip` 同时嵌入主程序并作为 Tauri 外置资源打包，造成最终安装器重复携带资源的问题。
- 修复安装器契约测试与实际资源回退逻辑不一致的问题，防止后续构建重新引入重复 ZIP。

【其他】

- 正式版本号更新为 `0.6.3`。
- 本版本继续使用 Tauri updater `.sig` 作为更新包签名；Windows NSIS 安装器的 Authenticode 状态与 updater 签名分开记录。
- 已存在但损坏或字段类型错误的 `core.json` 仍会保留严格校验，不会被默认配置覆盖。
- 本版本继续保留 CounterStrikeSharp、Inventory Simulator、定制资源包及相关上游项目的许可证、版权和归属信息。
- 库存换肤仍仅面向助手启动的本地 `-insecure` BOT 对局，不适用于官匹、5E、完美或其他受保护环境。
