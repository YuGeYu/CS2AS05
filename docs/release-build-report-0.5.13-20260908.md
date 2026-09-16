# CS2 人机增强助手 0.5.13 签名构建报告

## 构建状态

0.5.13 已完成本地 Windows x64 NSIS 签名构建。Tauri updater 签名已生成，当前产物可作为发布候选保存；本次没有上传 GitHub、R2、官网或 updater 服务，也没有宣称正式发布。

## 构建方式

- 项目目录：`E:\CS2AS05`
- 版本：`0.5.13`
- 构建命令：`npm run bundle:desktop`
- 目标：Windows x64 NSIS
- 签名方式：Tauri 2 updater signing
- 私钥来源：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys\updater.key`
- 密码来源：`updater-password.dpapi`
- DPAPI 处理：读取后先去除首尾空白，再传入 `ConvertTo-SecureString`
- 注入变量：`TAURI_SIGNING_PRIVATE_KEY`、`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `TAURI_SIGNING_PRIVATE_KEY_PATH` 仅作为兼容环境变量设置，但签名依赖的是私钥内容变量
- 构建结束后已清理私钥、密码和路径环境变量

私钥正文、解密密码和 DPAPI 明文没有写入文件、日志或更新日志。

## 本地产物

| 文件 | 大小 | SHA-256 |
|---|---:|---|
| `src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.13_x64-setup.exe` | 117,914,517 bytes | `9EE085E29E30B2628464E7DF76578C0D69727155DCDD29FC6F69D39D003AF40B` |
| `src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.13_x64-setup.exe.sig` | 436 bytes | `4C7E19F5BE2EABEA99628744459CA53BEF1B1454A1FB3A0B6FAA248BEE7ED59C` |

`.sig` 非空，长度为 436 bytes，内容符合 Tauri updater/minisign 签名文本格式。`dist-release/cs2-bot-improver/updater-dev.json` 已按当前本地候选生成，包含安装器大小、SHA-256 和签名正文；该文件仍是本地 dev manifest，不是公网发布 feed。

## 检查结果

- `npm run bundle:desktop`：通过
- Vite production build：通过
- Rust release build：通过
- Tauri NSIS bundle：通过
- Tauri updater `.sig`：已生成
- 构建后 `TAURI_SIGNING_PRIVATE_KEY`：未设置
- 构建后 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`：未设置
- `git diff --check`：通过
- Windows Authenticode：`NotSigned`

Windows Authenticode 与 Tauri updater `.sig` 是两套不同机制。本次已生成 updater 签名，但没有 Windows Authenticode 证书，因此不能把安装器称为 Authenticode 签名安装器。

## 发布材料

正式口径更新日志位于：`docs/docs/releases/release-notes-0.5.13.md`，已包含【新增】、【优化】、【修复】、【其他】四个类别，并基于 0.5.12 到 0.5.13 的实际改动编写。

## 尚未执行

- 未上传 GitHub Release、R2 或官网资源。
- 未切换公网 updater feed。
- 未执行真实旧版客户端到 0.5.13 的 updater 安装回读。
- 未执行玩家设备上的 CS2、BOT 工坊、预设、概览和 Demo 真实验收。

因此当前状态应表述为：**本地签名发布候选已构建，正式发布尚未执行。**

