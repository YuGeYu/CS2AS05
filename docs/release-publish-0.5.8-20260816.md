# 0.5.8 正式发布记录

日期：2026-08-16

## 发布结果

- D1 软件发布记录：已写入并启用 `cs2-bot-improver/prod/0.5.8`。
- R2：安装器上传命令返回 `Upload complete`，对象键为 `software-updates/cs2-bot-improver/prod/0.5.8/CS2人机增强助手_0.5.8_x64-setup.exe`。
- Updater feed：`https://cs2as.600318.xyz/api/software-updater/cs2-bot-improver/prod/windows/x86_64/0.0.0` 回读 `200`，版本为 `0.5.8`。
- 官网更新接口：`https://cs2as.600318.xyz/api/software-updates/cs2-bot-improver?channel=prod&currentVersion=0.0.0` 回读 `200`，`hasUpdate=true`，最新记录 active。
- 官网下载渠道：夸克 `https://pan.quark.cn/s/e3314f515421`。

## 产物一致性

- 安装器大小：`114638631` bytes。
- 安装器 SHA-256：`DD09D5B2617055C22770978B544C83201474C26C5BB7BEE5A16C6EC30AE03129`。
- Tauri `.sig`：已随同一次构建生成并写入 D1 发布记录。
- 官网公开响应中的 `size` 和 `sha256` 与本地签名安装器一致。

## 更新日志

官网公开响应中的 `items` 已包含 3 条【新增】、3 条【优化】、3 条【修复】和 1 条【其他】，与 `docs/release-notes-0.5.8.md` 一致。

## 回读边界

- R2 上传已成功。
- 用户已于 `2026-08-17` 完成 R2 对象下载回读并确认通过，0.5.8 发布链路闭合。
- 未执行 GitHub 上传、GitHub Release、官网代码重新部署或群公告发布。

## 版本固定

- `0.5.8` 自 `2026-08-17` 起固定，不再接受后续开发改动。
- 后续源码开发版本为 `0.5.9`；不得覆盖、重签或复用 0.5.8 的安装器、签名、D1 记录和 R2 对象键。
