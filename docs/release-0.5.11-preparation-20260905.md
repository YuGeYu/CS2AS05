# 0.5.11 发布准备记录

日期：2026-09-05

## 发布状态

0.5.11 已完成本地签名构建、updater manifest 生成和发布产物封存。当前 Windows 安装器为 Tauri updater 签名版本，可用于发布前安装测试。Windows Authenticode 仍为 `NotSigned`，这不影响 Tauri updater `.sig` 的验证机制。

## 构建与签名

- 私钥来源：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key`
- 密码来源：同目录 `updater-password.dpapi`
- DPAPI 内容在读取后执行首尾空白清理，再解密为构建进程密码。
- 构建进程注入 `TAURI_SIGNING_PRIVATE_KEY` 和 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`；未依赖单独的 `TAURI_SIGNING_PRIVATE_KEY_PATH`。
- 构建结束后已清理签名环境变量，项目文件和 artifacts 未写入私钥、密码或 DPAPI 明文。
- 配置内 updater 公钥与 `updater.key.pub` 一致。

## 发布产物

目录：`E:\CS2AS05\artifacts\release-0.5.11-signed-20260905\`

| 文件 | 大小 | SHA-256 |
| --- | ---: | --- |
| `CS2人机增强助手_0.5.11_x64-setup.exe` | 117,838,251 bytes | `763246C78C92332372767350F5BE6D228BE64E7C9D85646B73DED813FEF0EF5D` |
| `CS2人机增强助手_0.5.11_x64-setup.exe.sig` | 436 bytes | `F47C618344141B62C83B3AAE10E0CD7ADFCA902C1E0AFC33E737A0E1A4579EE1` |
| `updater-prod.json` | 859 bytes | `3DB09368B4FD65B65DC0D1F931916FD132A05E2614D220B57C52BE6807934B77` |

## 自动化门禁

- `cargo check --manifest-path src-tauri/Cargo.toml`：通过；仅存在既有第三方/未使用代码警告。
- `npm run typecheck`：通过。
- `npm run build:web`：通过。
- `npm test -- --run tests/bot-difficulty-workshop-contract.spec.ts`：2/2 通过。
- `npm run bundle:desktop`：成功生成签名安装器与 `.sig`。
- `npm run release:manifest`：成功生成 `updater-prod.json`，manifest 中的安装器大小和 SHA 与封存产物一致。

## 发布前边界

- 尚未执行 GitHub、R2、官网或其他线上发布写入。
- 尚未完成真实 CS2/BOT 行为验收；BOT 强度工坊、NadeSystem 和其他本地 BOT 能力仍应由本机实际对局确认后再对外宣称行为结果。
- 安装器的 Tauri updater 签名已生成；Windows Authenticode 代码签名未配置，系统签名状态为 `NotSigned`。
