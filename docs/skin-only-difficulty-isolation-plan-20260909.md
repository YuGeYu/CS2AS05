# 只开换肤模式隔离 BOT 难度方案

日期：2026-09-09

## 根因

当前 `skin_only` 虽然移除了 CS2AS05 BOT 插件，但仍使用 `backup/WithBots/gameinfo.gi`。该文件包含 `csgo/overrides` 搜索路径，所以 CS2 仍会读取 `overrides/botprofile.vpk`，玩家之前选择的 BOT 难度因此继续生效。

## 目标

只开换肤必须保留 MetaMod/CounterStrikeSharp 和 Inventory Simulator，但不加载 BOT 插件，也不让 CS2 读取任何 `overrides/botprofile.vpk`。玩家原有 BOT profile 不得被删除、覆盖或重置，切回 BOT 后仍可恢复。

## 实施方案

1. 资源包新增 `backup/SkinOnly/gameinfo.gi`。
2. 该文件以已验证的 WithBots 变体为基础，仅保留 Inventory Simulator 所需的 MetaMod/CounterStrikeSharp 路径，移除全部 `csgo/overrides`、`botprofile.vpk` 和 BOT profile SearchPath，不修改其它官方路径。
3. `GameInfoSidecar` 增加 `skin_only_sha256`，状态识别增加 `skin_only`。只有活动 gameinfo SHA、状态文件内容和 SearchPath 检查全部通过时才返回该状态。
4. `write_mode_at()` 显式使用三路资源：`online -> backup/Online`、`bots -> backup/WithBots`、`skin_only -> backup/SkinOnly`。写入前校验对应摘要。
5. `set_difficulty()` 在当前模式为 `skin_only` 时后端拒绝，返回 `[SKIN_ONLY_DIFFICULTY_LOCKED]`；前端禁用只是辅助，不能作为唯一保护。
6. 进入 skin-only 时不写 active `overrides/botprofile.vpk`，不改三份 Low/Medium/High 源文件；切回 bots 恢复原 active profile 和 WithBots gameinfo，切回 online 恢复官方 gameinfo。
7. 切换流程必须事务化：gameinfo、插件隔离目录、状态文件和 active profile 任一步失败，都恢复切换前字节。

## 文件范围

预计修改：`src-tauri/src/services/panel.rs`、`src-tauri/src/models/panel.rs`、`src-tauri/src/services/cs2.rs`、`src-tauri/resources/CS2BotImprover.zip`、`scripts/generate-plugin-manifest.ps1`、`tests/gameinfo-upstream-contract.spec.ts`、`tests/installer-contract.spec.ts`。建议新增 `tests/skin-only-gameinfo-contract.spec.ts` 和执行记录。

不修改 Inventory Simulator、玩家难度源 VPK、Online 官方 gameinfo、已发布 0.5.12 安装器、`.sig`、R2 对象和 GitHub Release。

## 测试门禁

- ZIP 存在 `backup/SkinOnly/gameinfo.gi`，摘要与 sidecar、manifest、Rust 常量一致。
- SkinOnly 不包含 `overrides`、`botprofile.vpk` 或 BOT 插件路径，但保留 MetaMod/CounterStrikeSharp 路径。
- `set_mode("skin_only")` 不再使用 WithBots 文件。
- skin-only 下 `set_difficulty()` 返回独立锁定错误。
- 切换前后 active profile SHA 不变，失败时完整回滚。
- 真实 CS2 中确认 Inventory Simulator 皮肤正常、助手选择的 BOT 难度不再生效、地图自带 BOT/关卡逻辑正常，切回 BOT 后原难度可恢复。

## 停止条件

无法同时保留 Inventory Simulator 与移除 overrides、active profile 被覆盖或丢失、摘要不一致、真实地图仍体现助手难度时停止，不得通过强制改成 Low/Medium 规避。

## 玩家口径

> “只开换肤”会保留库存换肤环境，但不会加载 CS2AS05 BOT 插件，也不会加载助手选择的 `botprofile.vpk`；创意工坊地图的 BOT 行为由地图和 CS2 原生逻辑决定，切回 BOT 后原难度仍可使用。
