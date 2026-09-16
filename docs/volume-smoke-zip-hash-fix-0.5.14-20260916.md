# 0.5.14 体积烟资源摘要修复

日期：2026-09-16

## 故障

玩家切换“体积烟功能”时收到：

```text
[ZIP_HASH_INVALID] 内置定制资源摘要不匹配。
期望：DFB6907BAE020F57FE98FA33DC77A01F5FA05F636EAB1296C472AEB53F597A95
实际：BEE883619EAB4B04AE333DB16007CC56BDF90558F33CC8A8B95F3097079DED59
```

## 根因

2026-09-15 将 `CS2BotLlmChat` 的 DLL、deps、PDB 和配置加入 `CS2BotImprover.zip` 后，资源 ZIP 的完整 SHA-256 已改变，但 `src-tauri/src/services/cs2.rs` 中的 `CUSTOM_ZIP_SHA256` 仍保留旧值。重新启用体积烟需要从内置 ZIP 恢复 `addons/metamod/BotVision.vdf`，因此先被整包摘要校验拦截。

## 修复

- 将 `CUSTOM_ZIP_SHA256` 同步为当前内置 ZIP 的真实 SHA-256：`BEE883619EAB4B04AE333DB16007CC56BDF90558F33CC8A8B95F3097079DED59`。
- 必需条目校验加入 `CS2BotLlmChat.dll`、deps 和玩家可修改的配置文件，避免摘要同步后遗漏新功能结构。
- `generate-plugin-manifest.ps1` 在原子替换 ZIP 后自动计算最终摘要并同步 Rust 常量；找不到唯一常量时立即失败，不再依赖人工复制。
- 安装器契约测试同时校验 ZIP 实际摘要、Rust 常量和生成脚本的自动同步能力。

## 资源检查

- ZIP 大小：70,277,668 bytes。
- ZIP 条目：683 个，无重复路径。
- BotVision `gamedata.json`、DLL、VDF 均存在。
- marker：schema 2，版本 0.5.14，固定 payload 包含 3 个 `CS2BotLlmChat` 插件文件。

## 自动化验证

- `tests/installer-contract.spec.ts`：8 passed，0 failed。
- Rust `services::cs2::tests`：9 passed，0 failed。
- Rust `cargo check --lib`：通过，仅有项目既有警告。
- PowerShell 生成脚本语法解析：通过。
- `git diff --check`：通过。

真实 CS2 中关闭/重新启用体积烟仍由玩家安装新候选后确认。自动化验证不能替代 MetaMod/BotVision 的实机加载结果。

## 本地测试安装包

- 路径：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.14_x64-setup.exe`
- 生成时间：2026-09-16 17:42:27 +08:00
- 大小：116,514,378 bytes
- SHA-256：`A7E671ADC692B3F1EE9A4A6FEF255B904F7A724172160F4AFEC3631C91ADA95B`
- 构建退出码：0
- 打包资源回读：70,277,668 bytes，SHA-256 `BEE883619EAB4B04AE333DB16007CC56BDF90558F33CC8A8B95F3097079DED59`
- Windows Authenticode：`NotSigned`
- 本地候选显式关闭 updater artifact 生成，因此没有 `.sig`，不可作为自动更新发布件。
