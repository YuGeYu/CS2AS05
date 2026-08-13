# 皮肤工坊安全门槛、图片加载与来源归并修复实施报告（2026-08-12）

## 1. 实施结论

已按 `docs/skin-forge-safety-image-sources-fix-execution-plan-20260812.md` 完成代码实现和自动化验证，未升级版本号、未新开分支、未提交、未推送、未打包安装程序。

- 每次挂载皮肤工坊都会先显示阻塞式风险确认；确认前不加载工坊配置、不部署、不修改、不重置、不应用。取消、Escape 和遮罩关闭统一通过 `cs2as:navigate` 返回概览。
- PlayerSkinMod 仅在 `allPresent === true` 且 `hashMismatches.length === 0` 时视为就绪。未就绪时部署按钮持续高亮并显示锚定说明，模式、阵营、目录选择、编辑、重置和应用均锁定。
- 配置写入具备按钮、Pinia store、Rust 命令三层保护。Rust 在 app draft 和 `player_loadout.json` 的任何写入前执行 `require_plugin_ready`，失败返回 `[PLAYER_SKIN_MOD_REQUIRED]`。
- 快速上手使用固定键 `cs2as:skin-forge:quick-start-highlighted:v1`，仅在风险确认和插件复检通过后高亮一次；部署提醒优先。storage 不可用不会阻塞工坊。
- Tauri asset protocol 已启用，scope 精确为 `$APPLOCALDATA/skin-forge/cache/images/**`；Cargo 同步启用 `protocol-asset` feature。
- 图片加载改为共享 URL 去重、最大 4 并发 FIFO、仅 `[IMAGE_BUSY]` 退避 120/300 ms 两次、视口附近加载和旧请求结果丢弃。
- 工坊内“关于与来源”入口已删除。安装与诊断中的来源页按核心上游、Demo 参考、皮肤参考展示五个项目；外链只接受 Rust 固定项目 ID 白名单。

## 2. 主要文件

- `src/views/SkinForgeView.vue`
- `src/features/skin-forge/components/ForgeSafetyDialog.vue`
- `src/features/skin-forge/components/ForgeWorkbench.vue`
- `src/features/skin-forge/components/CatalogGrid.vue`
- `src/features/skin-forge/components/ForgeImage.vue`
- `src/features/skin-forge/image-cache.ts`
- `src/features/skin-forge/quick-start-cue.ts`
- `src/stores/skinForge.ts`
- `src-tauri/src/commands/skin_forge.rs`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `src/components/AboutSourcesModal.vue`
- `src/services/tauri/support.ts`
- `src-tauri/src/commands/support.rs`
- `src-tauri/src/services/support.rs`
- `src/styles/main.css`

## 3. 自动化证据

- `npm run typecheck`：通过。
- `npm run lint`：通过；oxlint 0 warnings / 0 errors，ESLint 通过。
- 皮肤工坊定向前端测试：7 files / 21 tests passed。
- `npm test`：41 files / 148 tests passed。
- `npm run build:web`：通过；1771 modules transformed。仅有既有的大 chunk 提示。
- `cargo fmt --check`：通过。
- `cargo test --lib commands::skin_forge`：13 passed / 0 failed。
- `cargo test --lib services::support`：2 passed / 0 failed。
- `cargo test --lib`：76 passed / 0 failed / 3 real-environment tests ignored。
- `git diff --check`：通过。
- `npm run build:desktop`：通过，生成 `src-tauri/target/release/CS2BotImproverAssistant.exe`。

桌面二进制：

- 大小：44,943,360 bytes
- SHA-256：`6AA1EB2B44AE2A0A88E6BADB9E52502F4AE7B2E82C6E6246374E0BFE805913BF`
- 说明：这是 `tauri build --no-bundle` 的 release 主程序，不是安装包，也未执行签名检查。

Rust 输出仍有 `third_party/demoparser` 的既有 unused/lifetime warnings，本轮未修改第三方解析代码。

## 4. 用户负责的真实验收

按用户要求，本轮未启动或操作真实桌面窗口。以下项目尚未验证：

- Tauri 主窗口中首次和缓存后二次图片显示。
- 武器、刀具、手套、角色、音乐盒以及编辑器皮肤、贴纸、挂件的分类抽查。
- 1440x900、1100x700、reduced-motion 下的视觉布局与动画。
- 真实 CS2 本地/离线 `-insecure` 游戏内效果、配置 JSON 回读和 PlayerSkinMod 日志。
- 安装包构建、安装签名与安装事务。

在上述证据完成前，只能称为“代码、自动化与无界面桌面构建通过”，不能称为“真实桌面图片或游戏内修复完成”。

## 5. 最短验收步骤

1. 安装新候选，进入皮肤工坊，确认每次进入都有风险确认。
2. 保持 PlayerSkinMod 未部署，确认部署按钮和气泡持续出现，任何装备选择和“应用装备”都不可写入。
3. 退出 CS2，点击部署；部署成功后确认提醒消失、装备选择开放。
4. 依次查看武器、刀具、手套、角色、音乐盒以及编辑器内皮肤、贴纸、挂件图片。
5. 完成一套 CT/T 装备并应用，在本地/离线 `-insecure` BOT 对局重生查看；不要连接 VAC 保护服务器。
6. 关闭再打开工坊：风险确认再次出现；快速上手高亮不再出现。
