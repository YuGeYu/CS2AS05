# 0.5.13 签名发布候选构建记录

日期：2026-09-11
工作区：`E:\CS2AS05`
版本：`0.5.13`

## 构建结论

已使用项目签名流程完成 Windows x64 NSIS 安装器和 Tauri updater `.sig` 生成。DPAPI 文件先执行 `Trim()` 去除首尾空白，再进行十六进制解码和当前用户 DPAPI 解密；私钥内容注入 `TAURI_SIGNING_PRIVATE_KEY`，未使用仅设置路径的方式。

构建结束后已确认以下环境变量未残留：

```text
TAURI_SIGNING_PRIVATE_KEY=False
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=False
```

## 产物

```text
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.13_x64-setup.exe
E:\CS2AS05\src-tauri\target\release\bundle\nsis\CS2人机增强助手_0.5.13_x64-setup.exe.sig
```

| 文件 | 大小 | SHA-256 |
|---|---:|---|
| `CS2人机增强助手_0.5.13_x64-setup.exe` | 116064746 bytes | `9930410F50293C3B598FDED8DEF144C5B75549FCE100D1F64F1650B4A82C8E70` |
| `CS2人机增强助手_0.5.13_x64-setup.exe.sig` | 436 bytes | `3ECADB48593771D86428CDC8A4572D357546DE8B5B1B24D1AE71976F8CE6B6BB` |

## 候选交付目录

```text
E:\CS2AS05\artifacts\release-0.5.13-signed-20260911\
```

包含：

- 签名 NSIS 安装器；
- 436-byte updater `.sig`；
- 官网格式更新日志；
- `updater-prod.json`；
- `SHA256SUMS.txt`。

## 已通过检查

- `npm run typecheck`：通过；
- `npm run build:web`：通过；
- `npm run bundle:desktop`：通过；
- Tauri 已明确输出 updater signature；
- `npm run release:manifest`：通过；
- 生产 manifest 的版本、安装器文件名、签名、大小和 SHA-256 已生成；
- 安装器为 Tauri updater 签名，不等同于 Windows Authenticode 代码签名。

## 发布状态

当前仅完成本地签名发布候选和官网更新日志准备。未执行 GitHub 上传、R2 上传、D1 写入、官网 updater 启用或正式发布。真实 CS2、真实安装、在线下载与用户设备验收仍需独立完成。
