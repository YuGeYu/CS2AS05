# 只开换肤隔离 BOT 难度执行记录

日期：2026-09-09

## 已完成

- 新增并写入 `backup/SkinOnly/gameinfo.gi`。
- SkinOnly 保留 `csgo/addons/metamod`，移除 `csgo/overrides/botprofile.vpk`。
- `GameInfoSidecar` 增加 `skin_only_sha256`，面板状态可识别 `skin_only`。
- `write_mode_at()` 已改为 `online`、`bots`、`skin_only` 三路显式资源选择。
- `set_difficulty()` 在 SkinOnly 状态下返回 `[SKIN_ONLY_DIFFICULTY_LOCKED]`，后端不会写 active `botprofile.vpk`。
- ZIP 必需条目和 gameinfo manifest 已包含 SkinOnly 资源。
- 资源包当前 SHA-256：`D9C277DAA37DEC4DE232E4A49CBF117F3B83C8B83FF075560AA7B32C1AE87117`。
- SkinOnly gameinfo SHA-256：`AEFB44F51339F8422DA9F36EAC87E26E87D92129847BEDF2130227F8A0DFE780`。

## 静态验证

- `cargo check --manifest-path src-tauri/Cargo.toml`：通过，仅有仓库已有警告。
- `npm run typecheck`：通过。
- `cargo fmt --check`：通过。
- `git diff --check`：通过。
- SkinOnly 文件不含 `csgo/overrides` 和 `botprofile.vpk`，保留 `csgo/addons/metamod`。
- ZIP 中存在 `backup/SkinOnly/gameinfo.gi`。

## 尚未完成

- 尚未在真实 CS2 中确认 CounterStrikeSharp/Inventory Simulator 加载。
- 尚未在真实创意工坊地图确认助手选择的 BOT 难度不再生效。
- 尚未由用户验证切回 BOT 后 active profile 恢复。

## 发布边界

本次只更新工作区候选资源和代码，没有覆盖线上 0.5.12 安装器、签名、R2、D1、GitHub Release 或 updater feed。真实 CS2 验收通过前，不称为正式修复发布。
