# 安装与诊断维护入口及发布前检查（2026-08-13）

## 本次变更

- 删除安装与诊断页“查看诊断日志”入口、手动展开日志区域和前端刷新诊断状态。
- 保留 `submit_fault_report` 的后端诊断采集、脱敏和随故障工单上传能力；故障提交窗口继续提示“诊断日志将随工单提交”。
- 调整无诊断入口时的通用错误提示，改为引导用户重试或提交故障详情。

## 验证结果

- `npm run typecheck`：通过。
- `npm run lint`：Oxlint 0 warnings / 0 errors，ESLint 通过。
- `npm test -- --run`：48 个测试文件、161 项测试通过。
- `npm run build:web`：通过。
- `cargo check`：通过；仅第三方 demoparser 既有警告。

## 签名密钥检查

- 候选目录：`C:\Users\GOPtZ\Documents\CS2AS05-release-keys`。
- `updater.key`、`updater.key.pub`、`updater-password.dpapi` 均存在。
- 磁盘公钥与 `src-tauri/tauri.conf.json` 内嵌 updater 公钥逐字匹配。
- DPAPI 密码仅在当前构建进程内解密并注入环境变量，构建结束后已清理；私钥和密码未写入仓库、日志或本报告。

## 本次候选产物

- 安装器：`src-tauri/target/release/bundle/nsis/CS2人机增强助手_0.5.7_x64-setup.exe`
- Tauri updater 签名：同路径 `.exe.sig`，非空且与安装器同次构建生成。
- EXE SHA-256：`6083C52631DE8B1598A6FAF3B3ECD0E9C98C357F2266EEDA84783408291438D7`
- EXE 大小：114,527,717 bytes
- Windows Authenticode：`NotSigned`；这不影响 Tauri updater `.sig`，但该安装器不应宣称具有 Authenticode 发行签名。

## 发布边界

本次只完成本地代码、测试和签名构建准备，未上传 GitHub、R2、D1 或官网更新源。上线前仍需按发布流程核对 manifest、远端对象回读和真实客户端更新验签。
