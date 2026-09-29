# 库存换肤上游 3.2.0 合入记录

- 上游：`ianlucas/cs2-css-inventory-simulator` tag `3.2.0`，提交 `c9b8acd57cea173d62605a666d34771a070ff482`。
- 发布说明：缓存经济物品视图；仅在对应功能启用时安装 Hook，并在卸载时解除 Hook。
- 固定下载：源码 tar.gz SHA-256 `233CD51A0E887F97DD7336BE181D2C9842338FFC24C88DC33F74E8153CD76915`；官方发布 ZIP SHA-256 `AED5379AFE5E1E24FDB099A8D8AD427FEDF1014A312D7167653BA384C1313AAC`。
- 本地来源：`workspace/inventory-3.2.0-input/` 保留下载输入；`third_party/cs2-css-inventory-simulator/upstream/` 固定源码快照。
- 正式应用资源直接取自官方 ZIP 的插件 DLL、PDB、依赖清单、语言文件和 gamedata；没有下游编译补丁。上游 `invsim_ws_enabled` 默认 `false`，使用 `!ws` 前需在本地服务端执行 `invsim_ws_enabled true`。
- Release 依赖 `.NET 10` 和 `CounterStrikeSharp.API 1.0.375`。当前定制核心包基于上游 `CS2-Bot-Improver v1.4.4`，包内 API 为 `1.0.373`，尚不能证明兼容。须等待核心上游更新后统一验收。
- 应用资源位于 `src-tauri/resources/inventory-simulator/`；部署时按 `src-tauri/src/commands/inventory_simulator.rs` 的 SHA-256 逐项校验。完整哈希见 `third_party/cs2-css-inventory-simulator/SHA256SUMS.txt`。

此记录只证明固定来源与静态资源替换，不代表游戏内可用。按照本次决定，不进行测试、安装包构建、部署或上传。待核心上游更新时，再检查 API 版本、插件加载、`!ws` 刷新、饰品实际装备与喷漆开关切换后的 Hook 行为。应用主版本号保持不变。
