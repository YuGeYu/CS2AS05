# 0.5.12 最终发布准备记录

日期：2026-09-07

## 状态

- 版本：`0.5.12`
- 上一正式版本：`0.5.11`
- 当前状态：**发布收尾完成，0.5.12 发布基线已固定。** GitHub Release、官网 D1 发布记录、R2 对象、官网 updater feed 与下载路由均已上线；用户已完成 R2 下载回读并确认通过；旧版本 R2 对象已清理，bucket 中仅保留 0.5.12 对象。
- 本版本修复了一个会阻断 BOT 模式安装的发布缺陷：内置定制资源 ZIP 的摘要常量未随新 NadeSystem 资源包更新，已同步并新增契约测试防止复发。

## 线上发布记录

### GitHub

- 发布 commit：`85d8111`（`release: publish CS2 assistant 0.5.12`），tag `v0.5.12` 已推送。
- Release：https://github.com/YuGeYu/CS2AS05/releases/tag/v0.5.12
- 资产：`CS2._0.5.12_x64-setup.exe`（117,913,180 bytes）、同名 `.sig`（436 bytes）、`SHA256SUMS.txt`。
- 更新日志：release body 使用 `docs/release-notes-0.5.12.md` 全文（【新增】【优化】【修复】【其他】四分类）。

### 官网（D1 + R2 + Worker API）

- R2 bucket：`cs2as-r2`，新对象：
  - `software-updates/cs2-bot-improver/prod/0.5.12/CS2-Bot-Improver_0.5.12_x64-setup.exe`
  - `software-updates/cs2-bot-improver/prod/0.5.12/CS2-Bot-Improver_0.5.12_x64-setup.exe.sig`
  - `software-updates/cs2-bot-improver/prod/0.5.12/release-notes-0.5.12.md`
  - `software-updates/cs2-bot-improver/prod/0.5.12/updater-prod.json`
- D1 `software_releases`：新增/更新 `release_cs2_bot_improver_prod_0_5_12`，`is_active=1`、`updater_enabled=1`，artifact 字段（key/signature/sha256=3D227D8B…/size=117913180）完整；`0.5.11` 已设 `is_active=0`、`updater_enabled=0`。
- 官网更新日志：`items_json` 已写入 0.5.12 的 34 条【新增】【优化】【修复】【其他】条目。
- 全局开关 `software_update_settings.r2_push_enabled=1` 保持开启。
- 夸克渠道：`https://pan.quark.cn/s/c3b94db940d9`（已写入 `download_url`，未使用带尾字符的错误地址）。

### 线上核验

- `GET /api/software-updates/cs2-bot-improver?currentVersion=0.5.11&channel=prod`：`hasUpdate=true`，latest `0.5.12`，夸克地址正确，selfUpdate available，size/sha256 与本地一致。
- `GET /api/software-updater/…/windows/x86_64/0.5.11`：`200`，`version=0.5.12`，`signature` 与本地最终 `.sig` 逐字一致。
- `GET /api/software-updater/…/0.5.12`：`204`（当前已是最新）。
- 下载路由 HEAD：`200`，`Content-Length=117913180`，`application/octet-stream`，ASCII 文件名。未下载对象内容（R2 回读由用户执行）。

## 发布边界

- R2 对象内容回读与 SHA-256 比对：**已由用户执行并确认通过**。
- 旧对象清理：**已完成**。已删除 `software-updates/cs2-bot-improver/prod/` 下 0.5.1–0.5.11 的全部已知对象（66 个），删除后重新核验：0.5.12 的 4 个对象与本地产物 SHA-256 逐一一致（installer=3D227D8B…，sig=04206D50…，release-notes、updater-prod.json 均一致）。wrangler 无 `r2 object list` 子命令，对象清单以 D1 引用的 key 集合与既有命名约定枚举。
- Windows Authenticode 仍为 `NotSigned`，与 Tauri updater `.sig` 为不同签名机制。
- 未执行真实 CS2 BOT 环境的自定义档案编辑、重命名、删除和活动档案恢复验收。

## 固定口径

0.5.12 是当前正式版本。后续如需修复，必须以新版本号发布，不覆盖已公开的 0.5.12 安装器、`.sig`、R2 对象与 GitHub Release 资产。旧版本回滚素材（0.5.10/0.5.11 安装器与 `.sig`）仍保留在本机 `artifacts/` 交付目录。

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
