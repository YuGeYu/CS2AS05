# 0.5.13 发布准备记录

日期：2026-09-09  
版本：`0.5.13`  
上一正式版本：`0.5.12`

## 发布内容

- 新增“只开换肤”模式：保留 Inventory Simulator，隔离 CS2AS05 管理的 BOT 插件，地图与 CS2 原生 BOT 逻辑不再由本项目接管。
- 恢复上游 CS2-Bot-Improver v1.4.4 NadeSystem 投掷行为，移除 CS2AS05 的 `NadePacingPolicy` 节奏、开局预算、每回合数量、队伍烟雾和投掷间隔限制。
- 保留 `bot_nades off|less|normal|more|max` 五档及上游经济、地图数据、视线、特殊投掷和投掷通知行为。
- 纳入当前工作区已有的手动确认 CS2 已关闭、BOT 强度工坊、Demo 战术回放和启动台界面改进。

正式官网更新日志：[docs/releases/release-notes-0.5.13.md](./docs/releases/release-notes-0.5.13.md)。

## 签名构建

- 私钥目录：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys`。
- 私钥：读取 `updater.key` 后去除首尾空白，作为 `TAURI_SIGNING_PRIVATE_KEY` 内容变量注入。
- DPAPI 密码：读取 `updater-password.dpapi`，先去除首尾空白、十六进制解码、使用 CurrentUser DPAPI 解密、UTF-16LE 解码并再次去除首尾空白。
- 未使用 `ConvertTo-SecureString` 处理原始带换行 DPAPI 文本。
- 只在一个 `npm run bundle:desktop` 构建进程中注入 `TAURI_SIGNING_PRIVATE_KEY` 与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`；未使用 `TAURI_SIGNING_PRIVATE_KEY_PATH`。
- 构建结束后检查：两个签名环境变量均不存在。

## 最终本地签名产物

| 文件 | 大小 | SHA-256 |
| --- | ---: | --- |
| `CS2人机增强助手_0.5.13_x64-setup.exe` | 115,900,999 bytes | `98F11A5FADA2B6F840C5235204B804663D057CCC3BE0FC1D791A8F7E92A08104` |
| `CS2人机增强助手_0.5.13_x64-setup.exe.sig` | 436 bytes | `37FD688F2B83A85A124682DA8D2680343C844E3D2D9A94039C3F9F6A9878709E` |

路径：

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.13_x64-setup.exe
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.13_x64-setup.exe.sig
```

Tauri 已在本次构建输出中明确报告 updater signature 生成成功。Windows Authenticode 状态为 `NotSigned`；这不代表 Tauri updater `.sig` 缺失，两者是不同签名机制。

## 资源候选

- 内置 `CS2BotImprover.zip` SHA-256：`D9C277DAA37DEC4DE232E4A49CBF117F3B83C8B83FF075560AA7B32C1AE87117`。
- 上游 NadeSystem DLL SHA-256：`F9BBE17D1CA4729144A4F7CC37E1CD47B76729BC476E741987D0AC77F142F907`。
- SkinOnly `gameinfo.gi` SHA-256：`AEFB44F51339F8422DA9F36EAC87E26E87D92129847BEDF2130227F8A0DFE780`。
- 候选前资源备份与摘要：`artifacts/nades-pacing-removal-0.5.12-candidate-20260909-014623/`。

## 已完成验证

- `npm run workspace:check`：通过。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- 上游 NadeSystem `dotnet build -c Release --nologo`：通过，0 错误；有 `RayTraceApi` 已知引用警告。
- Rust 内置 ZIP 解包、marker 与安装器资源契约测试：通过。
- `cargo fmt --check` 与 `git diff --check`：通过。
- `npm run bundle:desktop`：通过，生成包含 Online 基线和 native BOT 隔离修复的 NSIS 安装程序与 436-byte Tauri `.sig`。

## 发布前仍需完成

- 在真实 CS2 中对 `bot_nades max|more|normal|less|off` 做同地图、同 BOT 数量对照，确认投掷频率与上游语义一致。
- 在创意工坊闯关地图中验收“只开换肤”：BOT 全灭后的下一关推进、地图自带清理 BOT 指令、多人库存刷新与重生外观。
- 本地 updater manifest 已生成并核对 `version`、安装器文件名、签名、大小和 SHA-256；线上 endpoint 仍未启用。
- 未创建 Git commit/tag/release，未上传 GitHub/R2，未写 D1 或启用官网 updater feed。
- 发布到 R2 后仍需由用户执行对象内容回读、哈希/大小比对与下载验证；未通过前保持 updater 未启用。

## 签名候选交付目录

```text
E:\CS2AS05\artifacts\release-0.5.13-signed-final-20260910\
```

目录包含：本次最新代码与资源生成的签名 NSIS 安装器、Tauri `.sig`、官网更新日志、`updater-prod.json` 和 UTF-8 `SHA256SUMS.txt`。本目录只作为发布候选，不代表已经上传或正式上线。

本次本地 updater manifest 已生成，Windows x86_64 安装器字段与签名、大小、SHA-256 一致；manifest 位于 `dist-release/cs2-bot-improver/updater-prod.json`，同时复制到候选目录。

## 结果口径

当前结果是：**0.5.13 最新代码与资源的本地签名发布候选已生成，待真实 CS2 验收和线上发布流程。**

