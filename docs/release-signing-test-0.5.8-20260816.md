# 0.5.8 签名构建与发布前测试记录

日期：2026-08-16

## 产物

- 安装器：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.8_x64-setup.exe`
- Tauri updater 签名：同路径 `.exe.sig`
- 安装器大小：`114638631` bytes
- 安装器 SHA-256：`DD09D5B2617055C22770978B544C83201474C26C5BB7BEE5A16C6EC30AE03129`
- `.sig` 大小：`436` bytes
- `.sig` SHA-256：`6D221DB1AFF4ACBFF0B7252041E686FB1C5DCCBC554007E1A1799DA1EDC1D61D`
- Windows Authenticode：`NotSigned`；这与 Tauri updater `.sig` 是两套独立机制。

## 密钥边界

- 使用本机 `C:\Users\GOPtZ\Documents\CS2AS05-release-keys` 中既有 Tauri updater 密钥材料。
- 私钥和 DPAPI 密码只在单次构建进程环境变量中存在，构建结束后已清理。
- 没有把密钥内容写入仓库、日志、安装器或本记录。

## 测试

- `npm run typecheck`：exit code `0`。
- 相关 Vitest：12 项通过，exit code `0`。
- `npm run build:web`：exit code `0`。
- `cargo test --manifest-path .\\src-tauri\\Cargo.toml performance_radar`：3 passed、1 ignored，exit code `0`。
- 本次修改文件 ESLint：exit code `0`。
- 全量 `npm run lint`：被工作树既有 `artifacts/thank-you-video` 第三方压缩脚本阻断；未修改该目录。

## 发布边界

本次只完成本地签名构建和测试，未上传 GitHub、R2、D1、官网或更新源，未安装覆盖用户环境，未发布 0.5.8。
