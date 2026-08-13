# 皮肤工坊五类装备三态与 0.5.7 NSIS 执行报告

执行日期：2026-08-12

## 上游能力结论

PlayerSkinMod 只有全局 `useRandom`，没有武器、刀具、手套、角色或音乐盒的分类独立随机字段。因此五类的“每次重生随机”都是同一个全局随机状态的派生显示；本轮没有向上游 JSON 伪造额外 mode 字段。`custom + sentinel` 可以在当前 JSON/adapter 中保持为“未选择”，但真实游戏内如何回退仍需 PlayerSkinMod 日志和 CS2 `-insecure` 画面验证。

## 实现

- 新增 `src/features/skin-forge/loadout-display.ts`，统一派生 `random / custom-value / custom-empty`。
- `ForgeWorkbench.vue` 的武器卡片以及右侧刀具、手套、角色、音乐盒摘要统一使用 formatter。
- 武器 custom 状态区分无记录、`paintKit=-1` 和有效 paint kit；全局 random 优先显示“每次重生随机”。
- 保留现有安全 gate、`chooseCustom()`、取消 sentinel 和 adapter 契约。
- 新增 `tests/skin-forge-three-state-display.spec.ts`。

## 自动化证据

- `npm run typecheck`：通过。
- `npm run lint`：通过，Oxlint 0 warning / 0 error。
- `npm test -- --run`：43 个文件，153 passed，0 failed。
- `cargo fmt --check`：通过。
- `cargo test --manifest-path Cargo.toml --lib`：76 passed，0 failed，3 ignored。
- Rust 构建仅报告 vendored `third_party/demoparser` 的既有 10 条 warning。
- `git diff --check`：通过。

## 版本与 Bundle 配置

- `package.json.version`：`0.5.7`。
- `src-tauri/tauri.conf.json.version`：`0.5.7`。
- `bundle.active=true`，`targets=nsis`，资源包含 `resources/skin-forge`。

## NSIS 候选

- 原始路径：`E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.7_x64-setup.exe`
- 交接路径：`E:\CS2AS05\workspace\player-acceptance\skin-forge-three-state-0.5.7-20260812\CS2人机增强助手_0.5.7_x64-setup.exe`
- 生成时间：`2026-08-12T18:46:33.0285302+08:00`
- 字节数：`113441078`
- SHA-256：`BAD63E44EECEA8DC787417F7CD637900B1AA87F424B6AB44066E4E389B08A2C5`
- Authenticode：`NotSigned`
- updater `.sig`：未生成。

`npm run bundle:desktop` 已完成 Web/Rust/NSIS 生成，但最后因配置存在 updater 公钥且环境没有 `TAURI_SIGNING_PRIVATE_KEY` 而退出码为 1。该文件只能作为本地测试候选，不能作为正式发布包。

## 未验证

- 因签名阶段失败触发方案停止条件，本轮未继续执行隔离安装、启动、进程清理和卸载事务。
- 未回读真实 `player_loadout.json` 或 PlayerSkinMod 日志。
- 未验证真实 CS2 `-insecure` 中五类三态的重生结果。
