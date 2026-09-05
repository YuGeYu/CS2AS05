# CS2 人机增强助手 0.5.11 开发中

## 基线

- 已发布并固定的正式版本：`0.5.10`
- 本地开发版本：`0.5.11`
- 0.5.10 交付基线：`dist-release/cs2-bot-improver/0.5.10`

## 开发约束

- 不修改 0.5.10 的 R2 对象、线上发布记录、夸克链接、安装器和 `.sig`。
- 改动 `CS2BotImprover.zip` 后，必须运行 `scripts/generate-plugin-manifest.ps1`，使 marker 版本与程序版本一致；随后同步 `CUSTOM_ZIP_SHA256`、资源 fixture 和 ZIP 契约测试。
- 发布前再次由最终签名安装器生成 updater manifest；不得复用历史 `.sig`、哈希或安装器。
