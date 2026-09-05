# gameinfo 更新兼容修复执行报告（2026-08-26）

## 执行结果

已完成代码与静态资源改造，未宣称 VAC 实机通过。开发基线来自 `D:\SteamLibrary\steamapps\common\Counter-Strike Global Offensive\game\csgo\gameinfo.gi`。

- size：9433 bytes
- mtime：2026-08-26 19:46:21（本地时间）
- SHA-256：`B1391E73DBEC2E078BDBAF7279C2B955084CF2B38A47E3D8181662E7948679B8`

## 修改范围

- `src-tauri/src/services/cs2.rs`：校验 `gameinfo.manifest.json` 三条资源摘要；事务安装写入官方原始备份与 sidecar；更新 ZIP 摘要。
- `src-tauri/src/services/panel.rs`：静态摘要门禁、结构化 `gameinfo` 状态、Online 基线检查、事务保护范围；BOT 启动仅保留 `-insecure`。
- `src-tauri/src/models/panel.rs`、`src/features/panel/types.ts`：增加 gameinfo 状态及启动结果摘要字段。
- `src-tauri/resources/CS2BotImprover.zip`：三条 gameinfo 静态资产和 `gameinfo.manifest.json` 已更新。
- `docs/panel-v1.4.3-contract.md`：增加动态基线契约章节。

未修改与本修复无关的插件 DLL、Steam 客户端、用户未知插件和 VAC 配置。

## 资源差异

Online 与根 `gameinfo.gi` 原样等同开发基线；BOT 变体仅增加 `Game csgo/addons/metamod` SearchPath。安装不再跳过三条 gameinfo，写入前由 ZIP hash、manifest 和事务回滚保护。

## 验证

- `cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check`：检测到工作区既有 Rust 文件格式差异，未执行全仓格式化。
- `cargo check --manifest-path src-tauri/Cargo.toml -q`：通过（仅保留 third_party demoparser 既有 warning）。
- `cargo test ... bundled_custom_zip_verifies_and_extracts_into_fake_cs2_root`：本轮在长时间编译阶段因清理残留构建进程而中止，未将其记为通过。
- `npm test -- --run tests/installer-contract.spec.ts tests/launch-experience.spec.ts`：Vitest worker 启动超时，未将其记为通过；这属于当前工作区测试基础设施状态，不替代 Rust 编译结果。
- 真实游戏树 BOT -> Online、Steam 默认入口、Steam 文件验证及 VAC secure 对局：待用户在备份后的真实目录执行。当前不能把大厅可启动、Steam spawn 成功或本地测试替代 VAC 验收。

## 实机验收步骤（待执行）

1. 退出 CS2，备份 `game/csgo/gameinfo.gi`，记录 size/mtime/SHA-256。
2. 安装助手并启动 BOT，确认 sidecar 摘要与活动 BOT 摘要一致，命令仅含 `-insecure`。
3. 退出 CS2，切换 Online，确认活动摘要恢复为上述官方基线。
4. 关闭助手，从 Steam 默认入口启动，确认无 `-insecure`、`-allow_third_party_software`、`-console`、`-condebug`。
5. Steam 验证游戏文件后再次记录摘要，并尝试加入 VAC secure 在线对局。

在第 5 步成功前，结论保持为“代码与静态资源修复完成，VAC 实机验收未完成”。
