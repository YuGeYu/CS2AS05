# CS2 人机增强助手 0.5.14 正式发布记录

发布日期：2026-09-16

## 发布状态

- 当前正式版本：`0.5.14`；上一正式版本：`0.5.13`。
- 用户已完成 R2 对象回读并确认通过。
- 夸克公开永久下载链接：`https://pan.quark.cn/s/cb786f2605f3`，无提取码。
- 官网已部署；生产 D1 中 `0.5.14` 为 `is_active=1`、`updater_enabled=1`。

## 最终产物

| 文件 | 大小 | SHA-256 |
| --- | ---: | --- |
| `CS2人机增强助手_0.5.14_x64-setup.exe` | 116,516,704 bytes | `CBF5E6B0DAD3B984B68AE39A8407285DD5A1931C74327C3DF37868558493761F` |
| `CS2人机增强助手_0.5.14_x64-setup.exe.sig` | 436 bytes | `1026840E3A2160765B999D5D7776300BB32DDF9046DB8955AFB3FABAC6396F27` |

R2 安装器对象键：`software-updates/cs2-bot-improver/prod/0.5.14/CS2-Bot-Improver_0.5.14_x64-setup.exe`。

## 签名边界

构建时将去除首尾空白后的 DPAPI 内容解密，并将私钥内容仅注入单次构建进程的 `TAURI_SIGNING_PRIVATE_KEY`。Tauri updater `.sig` 已生成；Windows Authenticode 状态为 `NotSigned`，两者是不同签名机制。

## 更新日志

正式稿位于 `docs/releases/release-notes-0.5.14.md`，并已同步至官网静态更新日志与生产软件发布记录，包含【新增】、【优化】、【修复】、【其他】四个分类。
