# CS2 人机增强助手 0.5.10 发布准备记录

日期：2026-08-28  
范围：本地正式发布候选、签名安装程序、更新日志与发布前验证

## 版本与更新日志

- `package.json`、`package-lock.json`、`src-tauri/Cargo.toml`、`src-tauri/Cargo.lock` 与 `src-tauri/tauri.conf.json` 已统一为 `0.5.10`。
- 正式发布口径更新日志已写入 `docs/release-notes-0.5.10.md`，包含【新增】、【优化】、【修复】、【其他】四类内容。
- 更新内容基于 0.5.9 正式版之后工作树中的实际实现，涵盖 Inventory Simulator、MapRotation、Demo 来源门禁、战报外观同步、LBRating 2.0、表现雷达基础能力及进程/诊断链路。

## 自动化验证

- `npm run verify`：工作区检查、类型检查、Oxlint、ESLint、测试和 Web 构建均通过。
- 测试最终结果：45 个文件、162 项测试全部通过。
- `cargo test --manifest-path src-tauri/Cargo.toml`：86 项通过、4 项按环境忽略、0 项失败。
- `npm run build:web`：通过；仅保留 Vite 动态导入与大 chunk 提示。
- Tauri release 编译：通过；第三方 demoparser 保留既有编译警告，无错误。

## 签名构建产物

安装器：

```text
src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.10_x64-setup.exe
size: 117202183 bytes
sha256: 8F1FC09E573050E6148A0D723C57665503D4086F1CACF03316759214220C69A2
file version: 0.5.10
```

Tauri updater 签名：

```text
src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.10_x64-setup.exe.sig
size: 436 bytes
sha256: 43DC927CF499DBBB2E2C8673B3554B3F97A52629F301AFF826ECCFCF39118464
```

签名构建在单个 PowerShell 进程中完成：DPAPI 文件先执行 `Trim()` 后再传入 `ConvertTo-SecureString`；私钥内容注入 `TAURI_SIGNING_PRIVATE_KEY`，密码注入 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。构建结束后两个变量均确认不存在。未输出、保存或提交密钥内容。

本次重建使用了修订后的 0.5.10 代码和插件资源包。ZIP SHA-256 为 `FC8868ABD46056DA52540EB14F3BA0B1B36005928C44178BC479C3BD2A4AA64E`，内置 marker 版本为 `0.5.10`，payload SHA-256 为 `76D9C84D74368DF6727435EAE0C830C7F75056E1E1F6E0AB14C128CD43DC9EF7`。本轮纳入 Online 官方 `gameinfo.gi` 基线校验、BOT 插件精确版本门禁及安装后 sidecar 回读修复。

生成的本地 updater manifest：

```text
dist-release/cs2-bot-improver/updater-prod.json
```

## 发布边界

- Tauri updater `.sig` 已针对本次 0.5.10 安装器生成；它与 Windows Authenticode 代码签名是两套独立机制。
- Windows Authenticode 当前回读为 `NotSigned`，本记录不将其表述为代码签名。
- 本轮未执行 R2/D1、官网、GitHub、夸克、安装覆盖用户环境、提交、推送或正式发布切换。
- 真实 CS2 对局、实机安装升级与玩家设备验收仍需在发布前按既有流程确认。
