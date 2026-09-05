# CS2 人机增强助手 0.5.10 最终签名构建记录

日期：2026-08-31  
范围：0.5.10 本地正式发布候选、Tauri updater 签名、发布前自动化验证

## 版本与更新日志

- `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock`、`src-tauri/tauri.conf.json` 均为 `0.5.10`。
- 正式版本更新日志：`docs/release-notes-0.5.10.md`。内容以 0.5.9 正式版为基线，按【新增】、【优化】、【修复】、【其他】分类，不使用开发预览口径。
- 本轮修复并锁定当前资源 ZIP 哈希：契约测试与 Rust `CUSTOM_ZIP_SHA256` 均为 `B5C101D44C799000BDE6A0E5ECE1A1291663CA666DB4017D562702FF4401E579`。
- `CS2BotImprover.zip` marker 版本为 `0.5.10`，固定 payload 摘要为 `6880C768E3C1BA42E47E78B57D38AB8C3AF761FBEA2B3E8F1D54D110B357FFB0`；MapRotation JSON 仅列入 `mutableConfigEntries`。

## 自动化验证

- `npm run verify`：通过。
- 工作区检查：通过。
- `vue-tsc --build --force`：通过。
- Oxlint：0 warnings / 0 errors；ESLint：通过。
- Vitest：45 个文件、162 项测试全部通过。
- `npm run build:web`：通过。Vite 仅输出既有动态导入和大 chunk 提示。
- `npm run bundle:desktop`：通过并生成 NSIS 安装程序及 updater `.sig`。

## 最终签名产物

安装器：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe
size: 117414162 bytes
sha256: 3C6334AD0DC385F6402E5D8FC656DA7A453E9EBDEE0DA2121D1315C3B3974F2A
```

Tauri updater 签名：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.10_x64-setup.exe.sig
size: 436 bytes
sha256: 13315E5A1CE4D4F364C26AD83EB61E5F3E60B0E88E776466132DC214FE8C8A9F
```

签名过程在单个 PowerShell 进程完成：DPAPI 密文先执行 `Trim()`，再传入 `ConvertTo-SecureString`；私钥内容注入 `TAURI_SIGNING_PRIVATE_KEY`，解密密码注入 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。构建结束后两个环境变量均确认不存在，未输出或写入密钥内容。

本地 updater manifest：

```text
E:\CS2AS05\dist-release\cs2-bot-improver\updater-prod.json
```

manifest 已回读并包含上述安装器文件名、SHA-256 与大小。

## 发布边界

- Tauri updater `.sig` 已生成；它与 Windows Authenticode 代码签名是两套独立机制。
- Windows Authenticode 回读为 `NotSigned`，本记录不将其表述为代码签名。
- 本轮未执行 R2/D1、官网、GitHub、夸克上传或正式发布切换；未安装覆盖用户环境，也未提交或推送。
- 真实 CS2 Online/BOT 对局、隔离安装升级和玩家设备验收仍需在发布前按既有验收流程完成；在此之前状态为“本地最终签名候选”。
