# CS2 人机增强助手 0.5.10 正式发布记录

发布日期：2026-08-31

## 正式发布状态

- 当前正式版本：`0.5.10`
- 上一正式版本：`0.5.9`
- `release_cs2_bot_improver_prod_0_5_10` 已设为 `is_active=1`、`updater_enabled=1`。
- `0.5.9` 已设为 `is_active=0`、`updater_enabled=0`，避免历史版本继续作为最新发布源。
- 夸克下载链接：`https://pan.quark.cn/s/32355ef669a1`

## 最终产物

- 安装器：`dist-release/cs2-bot-improver/0.5.10/CS2人机增强助手_0.5.10_x64-setup.exe`
  - size：117423074 bytes
  - SHA-256：`34F5921488752F71541F3CC74B97548B47407643D650098D851CCE0ED686883E`
- Tauri updater 签名：`dist-release/cs2-bot-improver/0.5.10/CS2人机增强助手_0.5.10_x64-setup.exe.sig`
  - size：436 bytes
  - SHA-256：`DFA4FC0743F627EF5143F0C0C18FAFC2661C5A8537A6676F120A072C0B83A553`
- updater manifest：`dist-release/cs2-bot-improver/0.5.10/updater-prod.json`

## R2 上传

以下对象已上传到 `cs2as-r2`：

- `software-updates/cs2-bot-improver/prod/0.5.10/CS2-Bot-Improver_0.5.10_x64-setup.exe`
- `software-updates/cs2-bot-improver/prod/0.5.10/CS2-Bot-Improver_0.5.10_x64-setup.exe.sig`
- `software-updates/cs2-bot-improver/prod/0.5.10/release-notes-0.5.10.md`
- `software-updates/cs2-bot-improver/prod/0.5.10/updater-prod.json`

R2 key 使用 ASCII 安装器名，是为绕过 Wrangler 对中文 object key 的 ByteString 缺陷；D1 的 `updater_r2_key` 已指向该实际对象。安装器内容和 updater 签名仍是中文本地文件名的正式构建产物。

## 线上核验

- 公共更新 API 对 `currentVersion=0.5.9` 返回 `hasUpdate=true` 和最新 `0.5.10`。
- API 返回的 updater SHA-256 为最终安装器 SHA-256，大小为 117423074 bytes。
- updater API 返回的 signature 已与本地最终 `.sig` 逐字匹配。
- 已验证 API 可签发下载 URL；按发布分工，本轮未下载或回读 R2 对象，R2 内容回读由用户执行。
- 用户已完成 R2 回读，并确认通过；0.5.10 发布基线自此固定。

## 更新日志

官网更新日志已写入线上 `items_json`，并保留本地正式稿 `docs/release-notes-0.5.10.md`。内容按【新增】、【优化】、【修复】、【其他】分类，使用正式发布口径。

## 签名边界

构建时 DPAPI 内容经过 `Trim()` 后解密密码，私钥内容仅注入构建进程的 `TAURI_SIGNING_PRIVATE_KEY`，构建结束后私钥与密码环境变量均已清理。Tauri updater `.sig` 已验证；Windows Authenticode 状态为 `NotSigned`，两者是不同签名机制。

## 后续开发

发布基线固定后，工作区版本已切换到 `0.5.11` 开发中。该切换不会修改已发布的 0.5.10 R2 对象、线上发布记录或本地 `dist-release/cs2-bot-improver/0.5.10` 交付目录。
