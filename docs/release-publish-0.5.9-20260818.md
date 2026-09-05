# CS2 人机增强助手 0.5.9 发布准备记录

日期：2026-08-18  
范围：本地正式发布候选、签名安装程序与发布前验证

## 已完成

- 开屏鸣谢场景和贡献陈列馆扩大地面、柱廊、展牌环与镜头活动范围。
- 开屏与彩蛋展牌采用独立的大半径环形布局，13 张展牌的相邻弦长均大于牌面宽度，避免折叠遮盖。
- 根据真实 CS2 日志修复 MapRotation 默认关闭导致的自动换图失效，重新编译并打包插件资源。
- `docs/release-notes-0.5.9.md` 已按【新增】、【优化】、【修复】、【其他】整理为正式版本更新日志，不使用开发预览措辞。
- 生成 Windows x64 NSIS 安装程序和 Tauri updater 签名文件。

## 验证结果

- `npm run typecheck`：通过。
- `npm run lint`：通过；已排除 `artifacts/**` 生成的浏览器缓存脚本，业务源码仍纳入检查。
- `npm test`：44 个文件、153 项测试全部通过。
- `npm run build:web`：通过。
- `cargo check`：通过；固定 demoparser 上游保留既有编译警告，无错误。
- Rust 关键引用项目测试：通过；完整 Rust 测试中的引用项目断言已同步到 0.5.9 的 9 个项目。

## 签名产物

安装器：

```text
src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.9_x64-setup.exe
size: 113751959 bytes
sha256: FCDECBCBB1C67CA6E872C2BC9623313E048B7E83FBC717CE503947F840C28C06
```

Tauri updater 签名：

```text
src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.9_x64-setup.exe.sig
size: 436 bytes
sha256: 1A079D5A4322C839F12F5BA2EA4C4E1169492AE53FBC55940AF6CE3A8670A1EC
```

签名构建使用 `C:\Users\GOPtZ\Documents\CS2AS05-release-keys` 中的现有密钥材料。DPAPI 文件内容先去除首尾空白再传入 `ConvertTo-SecureString`；私钥只通过单次进程的 `TAURI_SIGNING_PRIVATE_KEY` 注入，构建结束后已清理 `TAURI_SIGNING_PRIVATE_KEY` 与 `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`。磁盘公钥与 `src-tauri/tauri.conf.json` 内嵌公钥一致。

Windows Authenticode 状态为 `NotSigned`。这与 Tauri updater `.sig` 属于两套独立机制；本记录不把 `.sig` 表述为 Windows 证书签名。

## 正式发布结果

- 2026-08-18 已获用户授权正式发布 0.5.9。
- R2 对象已上传至 `cs2as-r2`：
  `software-updates/cs2-bot-improver/prod/0.5.9/CS2人机增强助手_0.5.9_x64-setup.exe`。
- 用户已完成 R2 独立回读，并确认大小与 SHA-256 校验通过：
  `113751959 bytes`，`FCDECBCBB1C67CA6E872C2BC9623313E048B7E83FBC717CE503947F840C28C06`。
- D1 `software_releases` 已将 0.5.9 设置为 `is_active=1`、`updater_enabled=1`，并绑定最终 R2 key、签名、SHA-256 和大小。
- 官网 API 已回读 0.5.9；旧版 0.5.8 updater feed 返回 HTTP 200，下载 URL、签名、大小和 SHA-256 均完整。
- 夸克回退下载地址：`https://pan.quark.cn/s/8b942772eec7`。

## 发布边界

- 本次未执行 GitHub Release、GitHub 上传、安装覆盖用户环境、提交或推送。
- Windows Authenticode 仍为 `NotSigned`；Tauri updater `.sig` 已正式启用，二者属于独立机制。
- 0.5.9 现已固定为最新正式版本；后续开发基线为 `0.5.10` 开发中版本。
