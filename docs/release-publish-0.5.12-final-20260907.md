# 0.5.12 最终发布准备记录

日期：2026-09-07

## 状态

- 版本：`0.5.12`
- 上一正式版本：`0.5.11`
- 当前状态：本机最终签名安装器、Tauri updater `.sig`、prod manifest 和官网更新日志已完成，签名已做独立密码学验证。
- 本轮修复了一个会阻断 BOT 模式安装的发布缺陷：内置定制资源 ZIP 的摘要常量未随新 NadeSystem 资源包更新，已同步并新增契约测试防止复发。
- R2、官网 updater、GitHub Release、夸克渠道和公开发布尚未执行。

## 本版本正式更新内容

- 工作区导航分组、标题栏上下文、侧栏收起和无障碍跳转。
- 安装诊断状态反馈和响应式标题栏/浮层层级修复。
- 人机强度工坊阻塞操作移出窗口主响应链路，旧档案读取结果不会覆盖新选择。
- 自定义 BOT 强度档案支持 DB 编辑保存、重命名、删除和可恢复归档。
- 删除活动自定义档案前自动备份并恢复对应 Low、Medium 或 High 内置难度。
- 人机强度工坊提取链路加固：独立错误码、写入探测、有限重试与陈旧工作目录清理。
- CS2-Bot-Improver v1.4.4、Panel v1.4.4、NadeSystem v1.4.4 资源链路和 42 支队伍预设保持一致。

正式日志（官网口径）：`docs/release-notes-0.5.12.md`

## 签名过程

- 私钥目录：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys`
- 私钥内容：`updater.key` 为 base64 包装文本，去首尾空白后作为 `TAURI_SIGNING_PRIVATE_KEY` 注入。
- 密码恢复：`updater-password.dpapi` 为十六进制文本 → 十六进制解码 → DPAPI `Unprotect`（CurrentUser）→ UTF-16LE 解码 → 去除首尾空白，得到 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。未使用 `ConvertTo-SecureString`，避免换行字符导致转换失败。
- 单次构建进程注入两个签名环境变量；未依赖 `TAURI_SIGNING_PRIVATE_KEY_PATH`。
- 构建结束后两个签名环境变量已随进程退出清空；未将私钥、密码或 DPAPI 明文写入仓库、日志或交付目录。

## 签名验证（独立密码学验证）

- 用 Node 按 rust-minisign 格式实现 Ed25519 验证：`.sig` 文件为 base64 包装的四行 minisign 文档。
- keynum（签名关键标识）与 `updater.key.pub` / `tauri.conf.json` 中配置的公钥一致。
- 文件签名 = Ed25519(Blake2b-512(安装器内容))：验证通过。
- 受信注释签名 = Ed25519(文件签名 64B || 受信注释文本)：验证通过。

## 最终产物

交付目录：`E:\CS2AS05\artifacts\release-0.5.12-signed-20260907-final3\`

| 文件 | 大小 | SHA-256 |
| --- | ---: | --- |
| `CS2人机增强助手_0.5.12_x64-setup.exe` | 117,913,180 bytes | `3D227D8B731440DED5E9EFFF2E30C6C8D2FAB958CC02B1CBE106234FF529E5F9` |
| `CS2人机增强助手_0.5.12_x64-setup.exe.sig` | 436 bytes | `04206D500CCC1C08B36F043608DA2B8B4D7A47A5359EC94B3682F82CD1DB616D` |

安装器：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.12_x64-setup.exe`

签名：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.12_x64-setup.exe.sig`

prod manifest：`dist-release/cs2-bot-improver/updater-prod.json`（installer、signature、sha256、size 已对应本次产物）。

Windows Authenticode：`NotSigned`。这与 Tauri updater `.sig` 属于不同签名机制，必须分别报告。

## 自动化验证

- `npm run typecheck`：通过。
- `npm run lint`：通过（已修复 2 个未使用声明，并将 `third_party/**` 加入忽略，避免对上游 React 源码误报）。
- `npm run workspace:check`：通过。
- `npm test`：49 个测试文件、176 项全部通过（新增：`CUSTOM_ZIP_SHA256` 与内置资源 ZIP 实际摘要的一致性契约测试）。
- `cargo fmt --manifest-path src-tauri/Cargo.toml`：通过。
- `cargo check --manifest-path src-tauri/Cargo.toml`：通过，仅有既有警告。
- `npm run bundle:desktop`：通过，生成 0.5.12 NSIS 安装器和 436-byte updater `.sig`。
- `npm run release:manifest`（`RELEASE_CHANNEL=prod`）：通过。

## 发布边界

- 当前只证明本机最终构建、签名与签名验证完成。
- 没有执行 R2 上传或下载回读。
- 没有启用官网 updater。
- 没有上传 GitHub Release 或夸克渠道。
- 没有执行真实 CS2 BOT 环境的自定义档案编辑、重命名、删除和活动档案恢复验收。
