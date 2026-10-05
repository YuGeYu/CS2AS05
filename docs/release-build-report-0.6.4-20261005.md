# CS2 人机增强助手 0.6.4 签名发布准备报告

日期：2026-10-05

## 发布范围

本次发布候选基于 `v0.6.3` 之后的实际代码和资源变更，包含 DemoTracer Demo 重玩、CS2-Bot-Improver v1.4.5、Inventory Simulator v3.5.0、卸载 ownership 安全、Panel v1.4.5 资源选择、CS2 BuildID 25640462 兼容、指令研究所桌面请求、关闭流程和概览自动录制移除等内容。

正式官网口径更新日志位于：

`docs/releases/release-notes-0.6.4.md`

## 自动化验证

- `npm run verify`：通过。
- `workspace:check`：通过，工作区登记有效。
- `vue-tsc`：通过。
- Oxlint：0 warning、0 error。
- ESLint：通过。
- Vitest：56 个测试文件、200 个测试全部通过。
- Vite Web 构建：通过。
- DemoTracer Rust 定向测试：3 passed、0 failed。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过。
- `cargo fmt --manifest-path src-tauri/Cargo.toml`：通过。
- `git diff --check`：通过。
- 使用本机 `minisign-verify` 以 `src-tauri/tauri.conf.json` 对应公钥离线验签：通过；trusted comment 文件名为 `CS2人机增强助手_0.6.4_x64-setup.exe`。

测试期间的 Node `localStorage`、jsdom canvas 和资源解包提示均为既有非阻断警告，没有失败测试。

## 签名流程

签名使用项目既有密钥目录：

`C:\Users\GOPtZ\Documents\CS2AS05-release-keys`

本次读取 `updater-password.dpapi` 时先执行首尾空白清理，再调用 `ConvertTo-SecureString` 解密。解密后的密码和 `updater.key` 修剪内容只注入当前构建进程使用的 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` 与 `TAURI_SIGNING_PRIVATE_KEY` 环境变量；构建结束后两个环境变量已清除，明文没有写入项目、manifest、日志或更新日志。

Tauri 已输出与最终 NSIS 文件配对的 `.sig`。磁盘公钥与 `src-tauri/tauri.conf.json` 的 updater 公钥逐字匹配。

## 最终产物

安装器：

`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe`

```text
大小：131,239,656 bytes
SHA-256：4270C550034087842EEEC6CA46C843024D3EFC5E6706433E4354082493140E7B
```

Tauri updater 签名：

`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.6.4_x64-setup.exe.sig`

```text
大小：436 bytes
SHA-256：ED22FD5743DD585CD07BFC8C896E35B5B2719985F9D4B8E3735A9BAC937A2F8F
```

Tauri 主程序：

`src-tauri/target/release/CS2BotImproverAssistant.exe`

```text
大小：121,259,520 bytes
SHA-256：B8A18C22F1B3CA08ED014C330B00951DE6DB439617C8DC83C8F113DED63A8E5D
```

生产 manifest：

`dist-release/cs2-bot-improver/updater-prod.json`

已核对 `version=0.6.4`、`channel=prod`、`projectId=cs2-bot-improver`，manifest 中的安装器大小、SHA-256 和签名正文与同一次构建产物完全一致。manifest 本身 SHA-256 为 `17C978A3826F170428EB3FB430FBA7317373AA0985D425C68E37DE3E54AF1327`。

离线发布证据目录：

`artifacts/release-0.6.4-signed-20261005/`

目录内保存最终安装器、`.sig`、生产 manifest、正式更新日志和 `SHA256SUMS.txt`。签名前的未签名候选另存于 `artifacts/release-0.6.4-unsigned-pre-signing-20261005/`，没有删除原文件。

Windows Authenticode 回读结果为 `NotSigned`。这不影响 Tauri updater `.sig` 的存在，但两者不能混为同一种签名；如果需要 Windows 发布者证书签名，仍需另行提供 Authenticode 证书和签名流程。

## 目标设备验收边界

本报告确认本地源码、资源、自动化测试、Rust 编译、生产 manifest、NSIS 和 Tauri updater `.sig` 已准备完成。用户于 2026-10-05 确认本机实际测试完成并授权正式发布；该确认作为本次发布门槛，不延伸为对所有玩家设备、游戏版本或第三方插件环境的兼容性保证。

## 外部发布状态（2026-10-05）

- 夸克分享：`https://pan.quark.cn/s/86aa81c9a274`，永久公开且无需提取码；上传文件名和大小已在夸克侧确认与签名安装包匹配。
- R2 上传：对象键为 `software-updates/cs2-bot-improver/prod/0.6.4/CS2人机增强助手_0.6.4_x64-setup.exe`，Wrangler 返回 `Upload complete.`。本地文件为 `131239656` bytes，SHA-256 为 `4270C550034087842EEEC6CA46C843024D3EFC5E6706433E4354082493140E7B`。用户于 2026-10-05 独立回读并确认通过；本次助手未读取 R2 对象内容。
- 官网 D1：已创建并启用 `cs2-bot-improver/prod/0.6.4` 正式记录，下载地址使用上述夸克链接；签名、对象键、本地 SHA-256 和大小已登记。`updater_enabled=1`，全局 `r2_push_enabled=1`。
- 官网部署：Cloudflare Worker `cs2as-official` 部署成功，最新生产版本 ID 为 `9cc5b4f4-166b-491f-ab5d-f0fd38126121`；公开更新日志路由 `/rizhi` 返回 HTTP 200。部署前官网 `npm run build`、`npm test` 通过（10 个测试文件、33 个测试）。
